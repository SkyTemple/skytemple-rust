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
use crate::compression::generic::nrl::{
    compression_step, decompression_step, NrlCompRead, NrlCompWrite, NrlDecompRead, NrlDecompWrite,
    NullablePrimitive,
};
use crate::python::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Cursor;

///

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TwoU16([u8; 4]);

impl TwoU16 {
    pub fn new(v: [u8; 4]) -> Self {
        Self(v)
    }
    pub fn raw(&self) -> &[u8; 4] {
        &self.0
    }
}

impl NullablePrimitive for TwoU16 {
    fn is_null(&self) -> bool {
        self.0 == [0, 0, 0, 0]
    }

    fn null() -> Self {
        Self([0, 0, 0, 0])
    }
}

///

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Pair24([u8; 3]);

impl Pair24 {
    pub fn new(v: [u8; 3]) -> Self {
        Self(v)
    }
    pub fn raw(&self) -> &[u8; 3] {
        &self.0
    }
}

impl NullablePrimitive for Pair24 {
    fn is_null(&self) -> bool {
        self.0 == [0, 0, 0]
    }

    fn null() -> Self {
        Self([0, 0, 0])
    }
}

///

impl From<Pair24> for TwoU16 {
    /// Writes the two u16 integers to the output as 2 16 bit integers
    ///
    /// ```txt
    /// Pair-24 packing:
    /// 1111 1111 2222 3333 4444 4444
    /// 1- The lowest 8 bits of the first value
    /// 2- The lowest 4 bits of the second value
    /// 3- The highest 4 bits of the first value
    /// 4- The highest 8 bits of the second value
    /// pattern_to_write = pattern_to_write >> 0xf
    /// 01 20 00 -> 00 10 02 -> 001 002
    /// 11 20 01 -> 01 10 12 -> 011 012
    /// ```
    fn from(other_in: Pair24) -> Self {
        let other: u32 =
            ((other_in.0[0] as u32) << 16) + ((other_in.0[1] as u32) << 8) + other_in.0[2] as u32;
        let v1 = (((0xff0000 & other) >> 16) + (0x000f00 & other)) as u16;
        let v2 = (((0x0000ff & other) << 4) + ((0x00f000 & other) >> 12)) as u16;
        Self(
            v1.to_le_bytes()
                .iter()
                .chain(v2.to_le_bytes().iter())
                .copied()
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl From<TwoU16> for Pair24 {
    /// Writes 4 bytes of 2 16 bit LE integers in pair24 encoding.
    fn from(other_in: TwoU16) -> Self {
        let mut other = &other_in.0[..];
        let int1 = other.get_u16_le() as u32;
        let int2 = other.get_u16_le() as u32;
        debug_assert!(int1 < 0x1000);
        debug_assert!(int2 < 0x1000);
        let pair24 =
            ((int1 & 0xff) << 16) + ((int2 & 0xf) << 12) + (int1 & 0xf00) + ((int2 & 0xff0) >> 4);
        let out = Self([
            ((pair24 >> 16) & 0xff) as u8,
            ((pair24 >> 8) & 0xff) as u8,
            (pair24 & 0xff) as u8,
        ]);
        debug_assert_eq!(other_in, TwoU16::from(out));
        out
    }
}

///

struct CompWrite(BytesMut);
#[derive(Clone)]
struct CompRead(Cursor<Bytes>);
struct DecompRead<'a, T>(&'a mut T)
where
    T: Buf;
struct DecompWrite(BytesMut);

impl NrlCompWrite<TwoU16> for CompWrite {
    fn nrl_put_u8(&mut self, val: u8) {
        self.0.put_u8(val)
    }

    fn nrl_put(&mut self, val: TwoU16) {
        self.0.put(&Pair24::from(val).raw()[..])
    }

    fn nrl_put_seq(&mut self, val: Bytes) {
        for seq in val.chunks_exact(4) {
            self.0
                .put(&Pair24::from(TwoU16::new(seq.try_into().unwrap())).raw()[..])
        }
    }

    fn nrl_unit_size() -> usize {
        4
    }

    fn nrl_put_into(target: &mut BytesMut, val: TwoU16) {
        target.put(&val.raw()[..])
    }
}

impl NrlCompRead<TwoU16> for CompRead {
    fn nrl_get(&mut self) -> TwoU16 {
        let mut out = [0, 0, 0, 0];
        self.0.copy_to_slice(&mut out[..]);
        TwoU16::new(out)
    }

    fn nrl_advance(&mut self, n: usize) {
        self.0.advance(n * 4);
    }

    fn nrl_has_remaining(&self) -> bool {
        self.0.remaining() > 3
    }
}

impl NrlDecompWrite<Pair24> for DecompWrite {
    fn nrl_put(&mut self, val: Pair24) {
        self.0.put(&TwoU16::from(val).raw()[..])
    }
}

impl<'a, T> NrlDecompRead<Pair24> for DecompRead<'a, T>
where
    T: Buf,
{
    fn nrl_get(&mut self) -> Pair24 {
        let mut out = [0, 0, 0];
        self.0.copy_to_slice(&mut out[..]);
        Pair24::new(out)
    }

    fn nrl_get_u8(&mut self) -> u8 {
        self.0.get_u8()
    }
}

///

pub struct BmaLayerNrlCompressor;

impl BmaLayerNrlCompressor {
    pub fn run(decompressed_data: Bytes) -> PyResult<Bytes> {
        if decompressed_data.is_empty() {
            return Ok(Bytes::new());
        }
        let mut compressed_data = CompWrite(BytesMut::with_capacity(decompressed_data.len() * 2));

        let mut cursor = CompRead(Cursor::new(decompressed_data));

        while cursor.0.has_remaining() {
            compression_step(&mut cursor, &mut compressed_data);
        }

        Ok(compressed_data.0.freeze())
    }
}

/////////////////////////////////////////
/////////////////////////////////////////

pub struct BmaLayerNrlDecompressor;

impl BmaLayerNrlDecompressor {
    pub fn run<T>(compressed_data: &mut Cursor<T>, stop_when_size: usize) -> PyResult<Bytes>
    where
        T: AsRef<[u8]>,
    {
        let mut compressed_data = DecompRead(compressed_data);
        let mut decompressed_data = DecompWrite(BytesMut::with_capacity(stop_when_size));
        while decompressed_data.0.len() < stop_when_size {
            if !compressed_data.0.has_remaining() {
                return Err(exceptions::PyValueError::new_err(format!(
                    "BMA Layer NRL Decompressor: Phase1: End result length unexpected. \
                    Should be {}, is {}.",
                    stop_when_size,
                    decompressed_data.0.len()
                )));
            }

            decompression_step(&mut compressed_data, &mut decompressed_data);
        }

        Ok(decompressed_data.0.freeze())
    }
}

// "Private" container for compressed data for use with tests written in Python (skytemple-files):
#[pyclass(module = "skytemple_rust._st_bma_layer_nrl_compression")]
#[derive(Clone)]
pub(crate) struct BmaLayerNrlCompressionContainer {
    compressed_data: Bytes,
    length_decompressed: u16,
}

impl BmaLayerNrlCompressionContainer {
    pub fn compress(data: &[u8]) -> PyResult<Self> {
        let compressed_data = BmaLayerNrlCompressor::run(Bytes::copy_from_slice(data))?;
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
impl BmaLayerNrlCompressionContainer {
    const DATA_START: usize = 8;
    const MAGIC: &'static [u8; 6] = b"BMANRL";

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
        let result =
            Ok(BmaLayerNrlDecompressor::run(&mut cur, self.length_decompressed as usize)?.into());
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
pub(crate) fn create_st_bma_layer_nrl_compression_module(
    py: Python,
) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust._st_bma_layer_nrl_compression";
    let m = PyModule::new(py, name)?;
    m.add_class::<BmaLayerNrlCompressionContainer>()?;

    Ok((name, m))
}
