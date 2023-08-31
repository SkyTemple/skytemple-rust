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
use crate::st_at_common::CompressionContainer;
use bytes::{BufMut, Bytes, BytesMut};

#[pyclass(module = "skytemple_rust.st_at4pn")]
#[derive(Clone)]
pub struct At4pn {
    data: Bytes,
}
impl CompressionContainer for At4pn {}

impl At4pn {
    pub fn compress(data: &[u8]) -> PyResult<Self> {
        Self::new(data, true)
    }
    pub fn matches(data: &[u8]) -> bool {
        &data[0..5] == Self::MAGIC
    }
}

#[pymethods]
impl At4pn {
    const DATA_START: usize = 7;
    const MAGIC: &'static [u8; 5] = b"AT4PN";

    #[new]
    pub fn new(data: &[u8], new: bool) -> PyResult<Self> {
        if new {
            Ok(Self {
                data: data.to_vec().into(),
            })
        } else {
            if Self::cont_size(&mut Bytes::copy_from_slice(data), 0)
                != (data.len() - Self::DATA_START) as u16
            {
                return Err(exceptions::PyValueError::new_err("Invalid data size."));
            }
            let (_, content) = data.split_at(Self::DATA_START);
            Ok(Self {
                data: content.to_vec().into(),
            })
        }
    }
    pub fn decompress(&self) -> PyResult<StBytesMut> {
        Ok(self.data.clone().into())
    }
    pub fn to_bytes(&self) -> StBytesMut {
        let len = self.data.len();
        let mut res = BytesMut::with_capacity(Self::DATA_START + len);
        res.put(Bytes::from_static(Self::MAGIC));
        res.put_u16_le(len as u16);
        res.put(self.data.clone());
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
pub(crate) fn create_st_at4pn_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_at4pn";
    let m = PyModule::new(py, name)?;
    m.add_class::<At4pn>()?;

    Ok((name, m))
}
