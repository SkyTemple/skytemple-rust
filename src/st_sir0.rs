use crate::bytes::{StBytes, StBytesMut};
use crate::python::PyErr;
use crate::python::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::convert::TryFrom;
use std::vec;
use thiserror::Error;

pub type Sir0Result<T> = Result<T, Sir0Error>;

#[derive(Error, Debug)]
pub enum Sir0Error {
    #[error("Serialization failed: {0}")]
    SerializeFailed(anyhow::Error),
    #[error("Unwrap failed: {0}")]
    UnwrapFailed(anyhow::Error),
    #[error("Serialization failed: {0}")]
    SerializeFailedPy(PyErr),
    #[error("Unwrap failed: {0}")]
    UnwrapFailedPy(PyErr),
    #[error("Sir0 file is too short. Length: {0}")]
    Sir0TooShort(usize),
    #[error("File is not a valid Sir0 file (wrong magic value).")]
    HeaderInvalid,
    #[error(
        "Pointer offset refers to a pointer that is out of bounds (0x{0:0x}). File length: {0}"
    )]
    PointerOffsetOutOfBounds(u32, usize),
    #[error(
        "Pointer at offset 0x{0:0x} is invalid; it points inside the Sir0 file header (0x{0:0x})."
    )]
    InvalidPointer(u32, u32),
    #[error("Pointer offset 0x{0:0x} is invalid; it refers to a location inside the Sir0 file header (0x{0:0x}).")]
    InvalidPointerOffset(u32),
    #[error("Pointer offset 0x{0:0x} is invalid; it would overflow a 32-bit unsigned integer when encoded.")]
    PointerOffsetOverflow(u32),
    #[error("Pointer at offset 0x{0:0x} (0x{0:0x}) is invalid; it would overflow a 32-bit unsigned integer when encoded.")]
    PointerOverflow(u32),
    #[error("The Sir0 file is too big. It's length must fit in a 32-bit unsigned integer. Current length: 0x{0:0x}.")]
    Sir0TooBig(usize),
    #[error("Data pointer (0x{0:0x}) is invalid; it would overflow a 32-bit unsigned integer when encoded.")]
    DataPointerOob(u32),
}

impl From<Sir0Error> for PyErr {
    fn from(source: Sir0Error) -> Self {
        match source {
            Sir0Error::SerializeFailedPy(e) => e,
            Sir0Error::UnwrapFailedPy(e) => e,
            _ => exceptions::PyValueError::new_err(format!(
                "Error trying to process Sir0 data: {}",
                source
            )),
        }
    }
}

pub trait Sir0Serializable
where
    Self: Sized,
{
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<u32>, Option<u32>)>;

    fn sir0_unwrap(content_data: StBytes, data_pointer: u32) -> Sir0Result<Self>;

    fn wrap(&self) -> Sir0Result<Sir0> {
        let (content, pointer_offsets, data_pointer) = self.sir0_serialize_parts()?;
        Ok(Sir0::new(content, pointer_offsets, data_pointer))
    }

    fn unwrap(self_as_sir0: Sir0) -> Sir0Result<Self> {
        Self::sir0_unwrap(self_as_sir0.content, self_as_sir0.data_pointer)
    }
}

// Based on C++ algorithm by psy_commando from
// https://projectpokemon.org/docs/mystery-dungeon-nds/sir0siro-format-r46/
pub(crate) fn decode_sir0_pointer_offsets(
    data: StBytes,
    pointer_offset_list_pointer: u32,
    relative: bool,
) -> Vec<u32> {
    let mut decoded: Vec<u32> = Vec::with_capacity(data.len());
    // This is used to sum up all offsets and obtain the offset relative to the file, and not the last offset
    let mut offsetsum = 0;
    // temp buffer to assemble longer offsets
    let mut buffer = 0;
    // This contains whether the byte read on the previous turn of the loop had the bit flag
    // indicating to append the next byte!
    let mut last_had_bit_flag = false;
    for curbyte in &data[(pointer_offset_list_pointer as usize)..data.len()] {
        if !last_had_bit_flag && *curbyte == 0 {
            break;
        }

        // Ignore the first bit, using the 0x7F bitmask, as its reserved.
        // And append or assign the next byte's value to the buffer.
        buffer |= (*curbyte as u32) & 0x7F;

        if (0x80 & curbyte) != 0 {
            last_had_bit_flag = true;
            // If first bit is 1, bitshift left the current buffer, to append the next byte.
            buffer <<= 7;
        } else {
            last_had_bit_flag = false;
            // If we don't need to append, add the value of the current buffer to the offset sum this far,
            // and add that value to the output vector. Then clear the buffer.
            if relative {
                offsetsum += buffer;
                decoded.push(offsetsum);
            } else {
                decoded.push(buffer);
            }
            buffer = 0;
        }
    }
    decoded
}

// Based on C++ algorithm by psy_commando from
// https://projectpokemon.org/docs/mystery-dungeon-nds/sir0siro-format-r46/
pub(crate) fn encode_sir0_pointer_offsets<S, I>(
    pointer_offsets: S,
    relative: bool,
) -> Sir0Result<StBytes>
where
    S: IntoIterator<Item = u32, IntoIter = I>,
    I: Iterator<Item = u32> + ExactSizeIterator,
{
    let pointer_offsets_iter = pointer_offsets.into_iter();
    let mut buffer = StBytesMut::from(vec![0; 4 * pointer_offsets_iter.len()]);
    let mut cursor = 0;
    // used to add up the sum of all the offsets up to the current one
    let mut offset_so_far = 0;
    for offset in pointer_offsets_iter {
        let offset_to_encode = if relative {
            offset - offset_so_far
        } else {
            // If we are not working relative, we can just use the offset directly.
            offset
        };

        // This tells the loop whether it needs to encode null bytes, if at least one higher byte was non-zero
        let mut has_higher_non_zero = false;
        // Set the value to the latest offset, so we can properly subtract it from the next offset.
        offset_so_far = offset;

        // Encode every bytes of the 4 bytes integer we have to
        for i in [4, 3, 2, 1] {
            let currentbyte = ((offset_to_encode >> (7 * (i - 1))) & 0x7F) as u8;
            // the lowest byte to encode is special
            if i == 1 {
                // If its the last byte to append, leave the highest bit to 0 !
                buffer[cursor] = currentbyte;
                cursor += 1;
            } else if currentbyte != 0 || has_higher_non_zero {
                // if any bytes but the lowest one! If not null OR if we have encoded a higher non-null byte before!
                buffer[cursor] = currentbyte | 0x80;
                cursor += 1;
                has_higher_non_zero = true;
            }
        }
    }

    buffer.truncate(cursor + 1);
    Ok(buffer.freeze())
}

#[pyclass(module = "skytemple_rust.st_sir0")]
#[derive(Clone)]
pub struct Sir0 {
    #[pyo3(get, set)]
    pub data_pointer: u32,
    #[pyo3(get, set)]
    pub content: StBytes,
    #[pyo3(get, set)]
    pub content_pointer_offsets: Vec<u32>,
}

#[pymethods]
impl Sir0 {
    const HEADER_LEN: u32 = 16;

    #[new]
    #[pyo3(signature = (content, pointer_offsets, data_pointer = None))]
    pub fn new(content: StBytes, pointer_offsets: Vec<u32>, data_pointer: Option<u32>) -> Self {
        Self {
            data_pointer: data_pointer.unwrap_or_default(),
            content,
            content_pointer_offsets: pointer_offsets,
        }
    }

    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "from_bin")]
    pub fn _py_from_bin(_cls: &PyType, data: StBytes) -> PyResult<Self> {
        <Self as TryFrom<_>>::try_from(data).map_err(PyErr::from)
    }
}

impl TryFrom<StBytes> for Sir0 {
    type Error = Sir0Error;

    fn try_from(buffer: StBytes) -> Result<Self, Self::Error> {
        let mut header = buffer.clone();
        if header.len() < Self::HEADER_LEN as usize {
            return Err(Sir0Error::Sir0TooShort(header.len()));
        }
        // Skip header
        if b"SIR0"[..] != header[0..4][..] {
            return Err(Sir0Error::HeaderInvalid);
        }
        header.advance(4);
        let data_pointer = header.get_u32_le();
        let pointer_offset_list = header.get_u32_le();

        let pointer_offsets =
            decode_sir0_pointer_offsets(buffer.clone(), pointer_offset_list, true);
        let mut buffer = StBytesMut::from(buffer.0);

        // Correct pointers by subtracting the header
        for pnt_off in pointer_offsets.iter().copied() {
            if (pnt_off + 4) as usize > buffer.len() {
                return Err(Sir0Error::PointerOffsetOutOfBounds(pnt_off, buffer.len()));
            }
            let val = (&buffer[(pnt_off as usize)..]).get_u32_le();
            let new_val = val.checked_sub(Self::HEADER_LEN);
            if let Some(new_val) = new_val {
                (&mut buffer[(pnt_off as usize)..]).put_u32_le(new_val);
            } else {
                return Err(Sir0Error::InvalidPointer(pnt_off, val));
            }
        }

        let content_pointer_offsets = pointer_offsets
            .into_iter()
            // The first two are for the pointers in the header, we remove them now, they are not
            // part of the content pointers
            .skip(2)
            .map(|pnt_off| {
                pnt_off
                    .checked_sub(Self::HEADER_LEN)
                    .ok_or(Sir0Error::InvalidPointerOffset(pnt_off))
            })
            .collect::<Sir0Result<Vec<u32>>>()?;

        Ok(Self {
            data_pointer: data_pointer - Self::HEADER_LEN,
            content: StBytes::from(
                &buffer[(Self::HEADER_LEN as usize)..(pointer_offset_list as usize)],
            ),
            content_pointer_offsets,
        })
    }
}

#[pyclass(module = "skytemple_rust.st_sir0")]
#[derive(Clone, Default)]
pub struct Sir0Writer;

#[pymethods]
impl Sir0Writer {
    #[new]
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "python")]
    #[pyo3(name = "write")]
    pub fn _py_write(&self, model: Py<Sir0>, py: Python) -> PyResult<StBytes> {
        self.write(model, py).map_err(PyErr::from)
    }
}

impl Sir0Writer {
    pub fn write(&self, model: Py<Sir0>, py: Python) -> Sir0Result<StBytes> {
        let brwd = model.borrow(py);
        let (mut content, mut content_pointer_offsets, data_pointer) = (
            StBytesMut::from(brwd.content.to_vec()),
            brwd.content_pointer_offsets.clone(),
            brwd.data_pointer,
        );

        // Correct all pointers in content by HEADER_LEN
        for pnt_off in content_pointer_offsets.iter_mut() {
            if (*pnt_off + 4) as usize > content.len() {
                return Err(Sir0Error::PointerOffsetOutOfBounds(*pnt_off, content.len()));
            }

            let added_pntr = (&content[(*pnt_off as usize)..])
                .get_u32_le()
                .checked_add(Sir0::HEADER_LEN);
            if let Some(added_pntr) = added_pntr {
                (&mut content[(*pnt_off as usize)..]).put_u32_le(added_pntr);
            } else {
                return Err(Sir0Error::PointerOverflow(*pnt_off));
            }

            let added = pnt_off.checked_add(Sir0::HEADER_LEN);
            if let Some(added) = added {
                *pnt_off = added;
            } else {
                return Err(Sir0Error::PointerOffsetOverflow(*pnt_off));
            }
        }

        // Also add the two header pointers
        let pointer_offsets = [4, 8]
            .into_iter()
            .chain(content_pointer_offsets)
            .collect::<Vec<u32>>();

        // Pointer offsets list
        let pol = encode_sir0_pointer_offsets(pointer_offsets, true)?;

        let len_content_padding = Self::len_pad(content.len());
        let len_eof_padding = Self::len_pad(pol.len());

        let pointer_pol = Sir0::HEADER_LEN
            .checked_add(content.len() as u32)
            .and_then(|v| v.checked_add(len_content_padding as u32))
            .ok_or_else(|| {
                Sir0Error::Sir0TooBig(
                    (Sir0::HEADER_LEN as usize) + content.len() + len_content_padding,
                )
            })?;

        // Header
        let mut header = BytesMut::with_capacity(Sir0::HEADER_LEN as usize);
        header.put(&b"SIR0"[..]);
        let data_pointer = data_pointer
            .checked_add(Sir0::HEADER_LEN)
            .ok_or(Sir0Error::DataPointerOob(data_pointer))?;
        header.put_u32_le(data_pointer);
        header.put_u32_le(pointer_pol);
        header.put_u32_le(0);

        Ok(StBytes(
            header
                .into_iter()
                .chain(content)
                .chain(Self::pad(len_content_padding))
                .chain(pol)
                .chain(Self::pad(len_eof_padding))
                .collect::<Bytes>(),
        ))
    }

    #[inline(always)]
    fn len_pad(cur_len: usize) -> usize {
        if cur_len % 16 == 0 {
            0
        } else {
            16 - (cur_len % 16)
        }
    }

    #[inline(always)]
    fn pad(padding_length: usize) -> vec::IntoIter<u8> {
        vec![0xAA; padding_length].into_iter()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_sir0_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_sir0";
    let m = PyModule::new(py, name)?;
    m.add_class::<Sir0>()?;
    m.add_class::<Sir0Writer>()?;

    Ok((name, m))
}
