/*
 * Copyright 2021-2022 Capypara and the SkyTemple Contributors
 *
 * This file is part of SkyTemple.
 *
 * SkyTemple is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SkyTemple is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with SkyTemple.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::bytes::StBytesMut;
use crate::python::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Cursor;

const RLE_MAX_LOOKAHEAD_SIZE: u8 = 127;

pub struct BmaCollisionRleCompressor {
    decompressed_data: Bytes,
    compressed_data: BytesMut,
}

impl BmaCollisionRleCompressor {
    pub fn run(decompressed_data: Bytes) -> PyResult<Bytes> {
        let mut slf = Self {
            compressed_data: BytesMut::with_capacity(decompressed_data.len() * 2),
            decompressed_data,
        };

        while slf.decompressed_data.has_remaining() {
            slf.process()
        }

        Ok(slf.compressed_data.freeze())
    }

    fn process(&mut self) {
        let next = Self::read(&mut self.decompressed_data);
        let mut repeats = 0;
        let mut nc = Cursor::new(self.decompressed_data.clone());
        while nc.has_remaining() && Self::read(&mut nc) == next && repeats < RLE_MAX_LOOKAHEAD_SIZE
        {
            repeats += 1;
        }
        self.decompressed_data.advance(repeats as usize);
        let w = if next > 0 {
            // Write 1
            0x80 + repeats
        } else {
            // Write 0
            repeats
        };
        self.compressed_data.put_u8(w);
    }

    #[inline]
    fn read<T>(from: &mut T) -> u8
    where
        T: Buf,
    {
        let n = from.get_u8();
        debug_assert!(n < 2);
        n
    }
}

/////////////////////////////////////////
/////////////////////////////////////////

pub struct BmaCollisionRleDecompressor<'a, T>
where
    T: 'a + AsRef<[u8]>,
{
    compressed_data: &'a mut Cursor<T>,
    decompressed_data: BytesMut,
    stop_when_size: usize,
}

impl<'a, T> BmaCollisionRleDecompressor<'a, T>
where
    T: 'a + AsRef<[u8]>,
{
    pub fn run(compressed_data: &'a mut Cursor<T>, stop_when_size: usize) -> PyResult<Bytes> {
        let mut slf = Self {
            decompressed_data: BytesMut::with_capacity(stop_when_size),
            compressed_data,
            stop_when_size,
        };

        while slf.decompressed_data.len() < slf.stop_when_size {
            if !slf.compressed_data.has_remaining() {
                return Err(exceptions::PyValueError::new_err(format!(
                    "BMA Collision RLE Decompressor: End result length unexpected. \
                    Should be {}, is {}.",
                    slf.stop_when_size,
                    slf.decompressed_data.len()
                )));
            }

            slf.process()
        }

        Ok(slf.decompressed_data.freeze())
    }

    fn process(&mut self) {
        let cmd = self.compressed_data.get_u8();
        let byte_to_write = cmd >> 7;
        let times_to_write = cmd & 0x7F;
        for _ in 0..times_to_write + 1 {
            self.decompressed_data.put_u8(byte_to_write);
        }
    }
}

// "Private" container for compressed data for use with tests written in Python (skytemple-files):
#[pyclass(module = "skytemple_rust._st_bma_collision_rle_compression")]
#[derive(Clone)]
pub(crate) struct BmaCollisionRleCompressionContainer {
    compressed_data: Bytes,
    length_decompressed: u16,
}

impl BmaCollisionRleCompressionContainer {
    pub fn compress(data: &[u8]) -> PyResult<Self> {
        let compressed_data = BmaCollisionRleCompressor::run(Bytes::copy_from_slice(data))?;
        Ok(Self {
            length_decompressed: data.len() as u16,
            compressed_data,
        })
    }
    fn cont_size(data: Bytes, byte_offset: usize) -> u16 {
        (data.len() - byte_offset) as u16
    }
}

#[pymethods]
impl BmaCollisionRleCompressionContainer {
    const DATA_START: usize = 8;
    const MAGIC: &'static [u8; 6] = b"BMARLE";

    #[new]
    pub fn new(data: &[u8]) -> PyResult<Self> {
        let mut data = Bytes::from(data.to_vec());
        data.advance(6);
        let length_decompressed = data.get_u16_le();
        Ok(Self {
            compressed_data: data,
            length_decompressed,
        })
    }
    pub fn decompress(&self) -> PyResult<StBytesMut> {
        let mut cur = Cursor::new(self.compressed_data.clone());
        let result = Ok(BmaCollisionRleDecompressor::run(
            &mut cur,
            self.length_decompressed as usize,
        )?
        .into());
        debug_assert!(!cur.has_remaining());
        result
    }
    pub fn to_bytes(&self) -> StBytesMut {
        let mut res = BytesMut::with_capacity(self.compressed_data.len() + Self::DATA_START);
        res.put(Bytes::from_static(Self::MAGIC));
        res.put_u16_le(self.length_decompressed);
        res.put(self.compressed_data.clone());
        res.into()
    }
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(signature = (data, byte_offset = 0))]
    #[pyo3(name = "cont_size")]
    fn _cont_size(_cls: &PyType, data: crate::bytes::StBytes, byte_offset: usize) -> u16 {
        Self::cont_size(data.0, byte_offset)
    }
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "compress")]
    fn _compress(_cls: &PyType, data: &[u8]) -> PyResult<Self> {
        Self::compress(data)
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bma_collision_rle_compression_module(
    py: Python,
) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust._st_bma_collision_rle_compression";
    let m = PyModule::new(py, name)?;
    m.add_class::<BmaCollisionRleCompressionContainer>()?;

    Ok((name, m))
}
