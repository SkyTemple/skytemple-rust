/*
 * Copyright 2021-2021 Parakoopa and the SkyTemple Contributors
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

use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::compression::custom_999::{Custom999Compressor, Custom999Decompressor};
use crate::python::*;
use crate::st_at_common::CompressionContainer;

#[pyclass(module = "st_atupx")]
#[derive(Clone)]
pub struct Atupx {
    data: Bytes,
    len_comp: u16,
    len_decomp: u32
}
impl CompressionContainer for Atupx {}

impl Atupx {
    fn compress(data: &[u8]) -> PyResult<Self> {
        let nine = Custom999Compressor::run(&mut Bytes::copy_from_slice(data));
        Ok(Self {
            len_comp: nine.len() as u16 + Self::DATA_START,
            data: nine.into(),
            len_decomp: data.len() as u32
        })
    }
}

#[pymethods]
impl Atupx {
    const DATA_START: u16 = 0x14;
    const MAGIC: &'static [u8; 5] = b"ATUPX";

    #[new]
    pub fn new(data: &[u8]) -> PyResult<Self> {
        let mut data = <&[u8]>::clone(&data);
        data.advance(5);
        Ok(Self {
            len_comp: data.get_u16_le(),
            len_decomp: data.get_u32_le(),
            data: data.to_vec().into(),
        })
    }
    pub fn decompress(&self) -> PyResult<StBytes> {
        Ok(Custom999Decompressor::run(&self.data, self.len_comp).into())
    }
    pub fn to_bytes(&self) -> PyResult<StBytes> {
        let mut res = BytesMut::with_capacity(self.len_comp as usize);
        res.put(Bytes::from_static(Self::MAGIC));
        res.put_u16_le(self.len_comp);
        res.put_u32_le(self.len_decomp);
        res.put(self.data.clone());
        assert_eq!(self.len_comp as usize, res.len());
        Ok(res.into())
    }
    #[cfg(not(feature = "no-python"))]
    #[classmethod]
    #[args(byte_offset = 0)]
    #[pyo3(name = "cont_size")]
    fn _cont_size(_cls: &PyType, data: &[u8], byte_offset: usize) -> u16 {
        Self::cont_size(&mut Bytes::copy_from_slice(data), byte_offset)
    }
    #[cfg(not(feature = "no-python"))]
    #[classmethod]
    #[pyo3(name = "compress")]
    fn _compress(_cls: &PyType, data: &[u8]) -> PyResult<Self> {
        Self::compress(data)
    }
}

#[cfg(not(feature = "no-python"))]
#[pymodule]
fn st_atupx(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Atupx>()?;
    Ok(())
}
