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

use crate::image::InWrappedImage;
use crate::image::OutWrappedImage;
use crate::python::*;

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct KaoImage {
}

impl KaoImage {
    fn create_from_raw(cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        todo!()
    }
}

#[pymethods]
impl KaoImage {
    #[cfg(not(feature = "no-python"))]
    #[classmethod]
    #[pyo3(name = "create_from_raw")]
    fn _create_from_raw(cls: &PyType, cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        todo!()
    }
    fn get(&self) -> PyResult<OutWrappedImage> {
        todo!()
    }
    fn size(&self) -> PyResult<u32> {
        todo!()
    }
    fn set(&mut self, img: InWrappedImage) -> PyResult<()> {
        todo!()
    }
    fn raw(&self) -> PyResult<(&[u8], &[u8])> {
        todo!()
    }
}

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct Kao {
}

#[pymethods]
impl Kao {
    #[new]
    pub fn new(data: &[u8]) -> PyResult<Self> {
        todo!()
    }
    pub fn expand(&mut self, new_size: u32) -> PyResult<()> {
        todo!()
    }
    pub fn get(&self, index: u32, subindex: u32) -> PyResult<Option<KaoImage>> {
        todo!()
    }
    pub fn set(&mut self, index: u32, subindex: u32, img: KaoImage) -> PyResult<()> {
        todo!()
    }
    pub fn set_from_img(&mut self, index: u32, subindex: u32, img: InWrappedImage) -> PyResult<()> {
        todo!()
    }
    pub fn delete(&mut self, index: u32, subindex: u32) -> PyResult<()> {
        todo!()
    }
    #[pyo3(name = "__iter__")]
    pub fn iter(&self, index: u32, subindex: u32) -> PyResult<KaoIterator> {
        todo!()
    }
}

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct KaoIterator {

}

impl Iterator for KaoIterator {
    type Item = (u32, u32, Option<KaoImage>);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[pymethods]
impl KaoIterator {
    fn __next__(&mut self) -> PyResult<(u32, u32, Option<KaoImage>)> {
        todo!()
    }
}

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct KaoWriter {
}

#[pymethods]
impl KaoWriter {
    #[new]
    pub fn new() -> PyResult<Self> {
        todo!()
    }
    pub fn write(&self, model: Kao) -> PyResult<&[u8]> {
        todo!()
    }
}

#[cfg(not(feature = "no-python"))]
#[pymodule]
fn st_kao(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<KaoImage>()?;
    m.add_class::<Kao>()?;
    m.add_class::<KaoWriter>()?;
    m.add_class::<KaoIterator>()?;
    Ok(())
}

