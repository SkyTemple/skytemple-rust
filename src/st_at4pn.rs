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

use bytes::{BufMut, Bytes, BytesMut};
use crate::python::*;
use crate::st_at_common::CompressionContainer;

#[pyclass(module = "st_at4pn")]
#[derive(Clone)]
pub struct At4pn {
    data: Bytes
}
impl CompressionContainer for At4pn {}

impl At4pn {
    fn compress(data: &[u8]) -> PyResult<Self> {
        Self::new(data, true)
    }
}

#[pymethods]
impl At4pn {
    const DATA_START: usize = 7;
    const MAGIC: &'static [u8; 5] = b"AT4PN";

    #[new]
    pub fn new(data: &[u8], new: bool) -> PyResult<Self> {
        if new {
            Ok(Self { data: data.to_vec().into() })
        } else {
            if Self::cont_size(&mut Bytes::copy_from_slice(data), 0) != (data.len() - Self::DATA_START) as u16 {
                return Err(exceptions::PyValueError::new_err("Invalid data size."));
            }
            let (_, content) = data.split_at(Self::DATA_START);
            Ok(Self { data: content.to_vec().into() })
        }
    }
    pub fn decompress(&self) -> PyResult<StBytes> {
        Ok(self.data.clone().into())
    }
    pub fn to_bytes(&self) -> PyResult<StBytes> {
        let len = self.data.len();
        let mut res = BytesMut::with_capacity(Self::DATA_START + len);
        res.put(Bytes::from_static(Self::MAGIC));
        res.put_u16_le(len as u16);
        res.put(self.data.clone());
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
fn st_at4pn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<At4pn>()?;
    Ok(())
}