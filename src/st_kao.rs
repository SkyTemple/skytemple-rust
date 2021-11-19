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

use std::vec;
use crate::image::InWrappedImage;
use crate::image::OutWrappedImage;
use crate::python::*;
#[cfg(not(feature = "no-python"))]
use pyo3::PyIterProtocol;
#[cfg(not(feature = "no-python"))]
use pyo3::iter::IterNextOutput;

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct KaoImage {
}

impl KaoImage {
    fn new(source: InWrappedImage) -> PyResult<Self> {
        todo!()
    }
    fn create_from_raw(cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        todo!()
    }
}

#[pymethods]
impl KaoImage {
    #[cfg(not(feature = "no-python"))]
    #[classmethod]
    #[pyo3(name = "create_from_raw")]
    fn _create_from_raw(_cls: &PyType, cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        Self::create_from_raw(cimg, pal)
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
    portraits: Vec<Vec<Option<KaoImage>>>
}

impl Kao {
    pub fn get(&self, index: usize, subindex: usize) -> PyResult<&Option<KaoImage>> {
        if index <= self.portraits.len() {
            if subindex < Self::PORTRAIT_SLOTS {
                return Ok(&self.portraits[index][subindex])
            }
            return Err(exceptions::PyValueError::new_err(
                format!("The subindex requested must be between 0 and {}", Self::PORTRAIT_SLOTS)
            ))
        }
        Err(exceptions::PyValueError::new_err(
            format!("The index requested must be between 0 and {}", self.portraits.len())
        ))
    }
}

#[pymethods]
impl Kao {
    const PORTRAIT_SLOTS: usize = 40;

    #[new]
    pub fn new(data: &[u8]) -> PyResult<Self> {
        todo!()
    }
    pub fn expand(&mut self, new_size: u32) -> PyResult<()> {
        todo!()
    }
    #[cfg(not(feature = "no-python"))]
    #[pyo3(name = "get")]
    pub fn _get(slf: PyRef<Self>, index: usize, subindex: usize) -> PyResult<PyObject> {
        Ok(slf.get(index, subindex)?.to_owned().into_py(slf.py()))
    }
    pub fn set(&mut self, index: usize, subindex: usize, img: KaoImage) -> PyResult<()> {
        if index <= self.portraits.len() {
            if subindex < Self::PORTRAIT_SLOTS as usize {
                self.portraits[index][subindex] = Some(img);
                return Ok(())
            }
            return Err(exceptions::PyValueError::new_err(
                format!("The subindex requested must be between 0 and {}", Self::PORTRAIT_SLOTS)
            ))
        }
        Err(exceptions::PyValueError::new_err(
            format!("The index requested must be between 0 and {}", self.portraits.len())
        ))
    }
    pub fn set_from_img(&mut self, index: usize, subindex: usize, img: InWrappedImage) -> PyResult<()> {
        if index <= self.portraits.len() {
            if subindex < Self::PORTRAIT_SLOTS as usize {
                match self.portraits[index][subindex].as_mut() {
                    Some(x) => x.set(img)?,
                    None => self.portraits[index][subindex] = Some(KaoImage::new(img)?)
                }
                return Ok(())
            }
            return Err(exceptions::PyValueError::new_err(
                format!("The subindex requested must be between 0 and {}", Self::PORTRAIT_SLOTS)
            ))
        }
        Err(exceptions::PyValueError::new_err(
            format!("The index requested must be between 0 and {}", self.portraits.len())
        ))
    }
    pub fn delete(&mut self, index: usize, subindex: usize) -> PyResult<()> {
        if index <= self.portraits.len() && subindex < Self::PORTRAIT_SLOTS {
            self.portraits[index][subindex] = None
        }
        Ok(())
    }
    #[cfg(feature = "no-python")]
    pub fn iter(&self, index: u32, subindex: u32) -> PyResult<KaoIterator> {
        let mut reff = Box::new(self.portraits.clone().into_iter()
            .map(
                |s| s.into_iter()
            ));
        let first = reff.next();
        Ok(KaoIterator {
            reference: reff,
            iter_outer: first,
            i_outer: 0,
            i_inner: -1
        })
    }
}

#[pyproto]
#[cfg(not(feature = "no-python"))]
impl PyIterProtocol for Kao {
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<KaoIterator>> {
        let mut reff = Box::new(slf.portraits.clone().into_iter()
            .map(
                |s| s.into_iter()
            ));
        let first = reff.next();
        Py::new(slf.py(), KaoIterator {
            reference: reff,
            iter_outer: first,
            i_outer: 0,
            i_inner: -1
        })
    }
}

#[pyclass(module = "st_kao", unsendable)]
pub struct KaoIterator {
    reference: Box<dyn Iterator<Item=std::vec::IntoIter<Option<KaoImage>>>>,
    iter_outer: Option<vec::IntoIter<Option<KaoImage>>>,
    i_outer: u32,
    i_inner: i32
}

impl Iterator for KaoIterator {
    type Item = (u32, u32, Option<KaoImage>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_outer.as_ref()?;
        self.i_inner += 1;
        match self.iter_outer.as_mut().unwrap().next() {
            Some(x) => Some((self.i_outer, self.i_inner as u32, x)),
            None => {
                self.i_outer += 1;
                self.iter_outer = self.reference.next();
                self.iter_outer.as_ref()?;
                self.i_inner = -1;
                self.next()
            }
        }
    }
}

#[pyproto]
#[cfg(not(feature = "no-python"))]
impl PyIterProtocol for KaoIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<(u32, u32, Option<KaoImage>), &'static str> {
        match slf.next() {
            Some(x) => IterNextOutput::Yield(x),
            None => IterNextOutput::Return("")
        }
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
