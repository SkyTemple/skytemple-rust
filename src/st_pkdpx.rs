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
use crate::compression::px::{PxCompLevel, PxCompressor, PxDecompressor};
use crate::python::*;
use crate::st_at_common::CompressionContainer;
use crate::util::slice_to_array;

#[pyclass(module = "st_pkdpx")]
#[derive(Clone)]
pub struct Pkdpx {
    data: Bytes,
    flags: [u8; 9],
    len_comp: u16,
    len_decomp: u32
}
impl CompressionContainer for Pkdpx {}

impl Pkdpx {
    fn compress(data: &[u8]) -> PyResult<Self> {
        let (px, flags) = PxCompressor::run(
            Bytes::copy_from_slice(data), PxCompLevel::Level3, true
        )?;
        Ok(Self {
            len_comp: px.len() as u16 + Self::DATA_START, data: px, flags, len_decomp: data.len() as u32
        })
    }
}

#[pymethods]
impl Pkdpx {
    const DATA_START: u16 = 0x14;
    const MAGIC: &'static [u8; 5] = b"PKDPX";

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
            len_decomp: data.get_u32_le(),
            data: data.to_vec().into(),
        })
    }
    pub fn decompress(&self) -> PyResult<StBytes> {
        Ok(PxDecompressor::run(&self.data, self.flags.as_ref(), self.len_comp)?.into())
    }
    pub fn to_bytes(&self) -> PyResult<StBytes> {
        let mut res = BytesMut::with_capacity(self.len_comp as usize);
        res.put(Bytes::from_static(Self::MAGIC));
        res.put_u16_le(self.len_comp);
        res.put(&self.flags[..]);
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
fn st_pkdpx(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pkdpx>()?;
    Ok(())
}
