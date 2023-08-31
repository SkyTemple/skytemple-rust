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
use crate::compression::px::{PxCompLevel, PxCompressor, PxDecompressor};
use crate::python::*;
use crate::st_at_common::CompressionContainer;
use crate::util::slice_to_array;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[pyclass(module = "skytemple_rust.st_at3px")]
#[derive(Clone)]
pub struct At3px {
    data: Bytes,
    flags: [u8; 9],
    len_comp: u16,
}
impl CompressionContainer for At3px {}

impl At3px {
    pub fn compress(data: &[u8]) -> PyResult<Self> {
        let (px, flags) =
            PxCompressor::run(Bytes::copy_from_slice(data), PxCompLevel::Level3, true)?;
        Ok(Self {
            len_comp: px.len() as u16 + Self::DATA_START,
            data: px,
            flags,
        })
    }
    pub fn matches(data: &[u8]) -> bool {
        &data[0..5] == Self::MAGIC
    }
}

#[pymethods]
impl At3px {
    const DATA_START: u16 = 0x10;
    const MAGIC: &'static [u8; 5] = b"AT3PX";

    #[new]
    pub fn new(data: &[u8]) -> PyResult<Self> {
        let mut data = <&[u8]>::clone(&data);
        data.advance(5);
        let len_comp = data.get_u16_le();
        let flags = slice_to_array(&data[..9]);
        data.advance(9);
        Ok(Self {
            len_comp,
            flags,
            data: data.to_vec().into(),
        })
    }
    pub fn decompress(&self) -> PyResult<StBytesMut> {
        Ok(PxDecompressor::run(
            &self.data[..(self.len_comp - Self::DATA_START) as usize],
            self.flags.as_ref(),
            self.len_comp,
        )?
        .into())
    }
    pub fn to_bytes(&self) -> StBytesMut {
        let mut res = BytesMut::with_capacity(self.len_comp as usize);
        res.put(Bytes::from_static(Self::MAGIC));
        res.put_u16_le(self.len_comp);
        res.put(&self.flags[..]);
        res.put(self.data.clone());
        debug_assert_eq!(self.len_comp as usize, res.len());
        res.into()
    }
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(signature = (data, byte_offset = 0))]
    #[pyo3(name = "cont_size")]
    fn _cont_size(_cls: &PyType, data: &[u8], byte_offset: usize) -> u16 {
        Self::cont_size(&mut <&[u8]>::clone(&data), byte_offset)
    }
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "compress")]
    fn _compress(_cls: &PyType, data: &[u8]) -> PyResult<Self> {
        Self::compress(data)
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_at3px_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_at3px";
    let m = PyModule::new(py, name)?;
    m.add_class::<At3px>()?;

    Ok((name, m))
}
