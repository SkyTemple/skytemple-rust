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

use arr_macro::arr;
use std::io::Cursor;
use std::vec;
use bytes::Buf;
use crate::image::{IndexedImage, InWrappedImage, Tile, TiledImage};
use crate::python::*;
#[cfg(feature = "python")]
use pyo3::PyIterProtocol;
#[cfg(feature = "python")]
use pyo3::iter::IterNextOutput;
use crate::st_at_common::{COMMON_AT_MUST_COMPRESS_3, CommonAt};

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct KaoImage {
    pal_data: Vec<u8>,
    compressed_img_data: Vec<u8>
}

impl KaoImage {
    const KAO_IMG_PAL_B_SIZE: usize = 48;  // Size of KaoImage palette block in bytes (16*3)
    const TILE_DIM: usize = 8;
    const IMG_DIM: usize = 40;

    pub fn new(raw_data: &[u8]) -> PyResult<Self> {
        let cont_len: usize;
        if let Some(x) = CommonAt::cont_size(&raw_data[Self::KAO_IMG_PAL_B_SIZE..], 0) {
            cont_len = x as usize;
        } else {
            return Err(exceptions::PyValueError::new_err("Invalid Kao image data; image not an AT container."));
        }
        // palette size + at container size
        Ok(Self {
            pal_data: Vec::from(&raw_data[..Self::KAO_IMG_PAL_B_SIZE]),
            compressed_img_data: Vec::from(&raw_data[Self::KAO_IMG_PAL_B_SIZE..Self::KAO_IMG_PAL_B_SIZE + cont_len])
        })
    }
    pub fn new_from_img(source: IndexedImage) -> PyResult<Self> {
        let (pal, img) = Self::bitmap_to_kao(source)?;
        Ok(Self {
            compressed_img_data: img,
            pal_data: pal
        })
    }
    pub fn create_from_raw(cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        Ok(Self {
            pal_data: Vec::from(pal),
            compressed_img_data: Vec::from(cimg)
        })
    }

    fn bitmap_to_kao(source: IndexedImage) -> PyResult<(Vec<u8>, Vec<u8>)> {
        let (pal, img) = TiledImage::native_to_tiled_seq(source, Self::TILE_DIM, Self::IMG_DIM)?;
        Ok((Tile::unpack(pal), CommonAt::compress(&*img, COMMON_AT_MUST_COMPRESS_3.iter())?))
    }
}

#[pymethods]
impl KaoImage {
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "create_from_raw")]
    fn _create_from_raw(_cls: &PyType, cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        Self::create_from_raw(cimg, pal)
    }
    pub fn get(&self) -> PyResult<IndexedImage> {
        TiledImage::tiled_to_native_seq(
            Tile::pack4bpp(CommonAt::decompress(&*self.compressed_img_data)?.into_iter()),
            &self.pal_data, Self::TILE_DIM, Self::IMG_DIM
        )
    }
    pub fn size(&self) -> PyResult<usize> {
        Ok(Self::KAO_IMG_PAL_B_SIZE + self.compressed_img_data.len())
    }
    pub fn set(&mut self, py: Python, source: InWrappedImage) -> PyResult<()> {
        let (pal, img) = Self::bitmap_to_kao(source.extract(py)?)?;
        self.pal_data = pal;
        self.compressed_img_data = img;
        Ok(())
    }
    pub fn raw(&self) -> PyResult<(&[u8], &[u8])> {
        Ok((&self.compressed_img_data[..], &self.pal_data[..]))
    }
}

#[pyclass(module = "st_kao")]
#[derive(Clone)]
pub struct Kao {
    portraits: Vec<[Option<KaoImage>; Self::PORTRAIT_SLOTS]>
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
    #[allow(clippy::needless_range_loop)]
    pub fn new(raw_data: &[u8]) -> PyResult<Self> {
        let mut data = Cursor::new(raw_data);
        let mut portraits: Vec<[Option<KaoImage>; Self::PORTRAIT_SLOTS]> = Vec::with_capacity(1600);
        // First 160 bytes are padding
        data.advance(160);
        let mut first_pointer = 0;
        while first_pointer == 0 || data.position() < first_pointer {
            let mut species: [Option<KaoImage>; Self::PORTRAIT_SLOTS] = arr![None; 40];
            for i in 0..Self::PORTRAIT_SLOTS {
                let pointer = data.get_i32_le();
                if pointer > 0 {
                    if first_pointer == 0 {
                        first_pointer = pointer as u64;
                    }
                    species[i] = Some(KaoImage::new(&raw_data[pointer as usize..])?);
                }
            }
            portraits.push(species);
        }
        if data.position() > first_pointer {
            return Err(exceptions::PyValueError::new_err("Corrupt KAO TOC."));
        }
        Ok(Self { portraits })
    }
    pub fn expand(&mut self, new_size: usize) -> PyResult<()> {
        if new_size < self.portraits.len() {
            return Err(exceptions::PyValueError::new_err(format!(
                "Can't reduce size from {} to {}", self.portraits.len(), new_size
            )));
        }
        for _ in self.portraits.len()..new_size {
            self.portraits.push(arr![None; 40]);
        }
        Ok(())
    }
    #[cfg(feature = "python")]
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
    pub fn set_from_img(&mut self, py: Python, index: usize, subindex: usize, img: InWrappedImage) -> PyResult<()> {
        if index <= self.portraits.len() {
            if subindex < Self::PORTRAIT_SLOTS as usize {
                match self.portraits[index][subindex].as_mut() {
                    Some(x) => x.set(py, img)?,
                    None => self.portraits[index][subindex] = Some(KaoImage::new_from_img(img.extract(py)?)?)
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
    #[cfg(not(feature = "python"))]
    pub fn iter(&self, index: u32, subindex: u32) -> PyResult<KaoIterator> {
        let mut reference = Box::new(self.portraits.clone().into_iter()
            .map(
                |s| s.to_vec().into_iter()
            ));
        let iter_outer = reference.next();
        Ok(KaoIterator {
            reference,
            iter_outer,
            i_outer: 0,
            i_inner: -1
        })
    }
}

#[pyproto]
#[cfg(feature = "python")]
impl PyIterProtocol for Kao {
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<KaoIterator>> {
        let mut reference = Box::new(slf.portraits.clone().into_iter()
            .map(
                |s| s.to_vec().into_iter()
            ));
        let iter_outer = reference.next();
        Py::new(slf.py(), KaoIterator {
            reference,
            iter_outer,
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
#[cfg(feature = "python")]
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

#[cfg(feature = "python")]
pub(crate) fn create_st_kao_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_kao";
    let m = PyModule::new(py, name)?;
    m.add_class::<KaoImage>()?;
    m.add_class::<Kao>()?;
    m.add_class::<KaoWriter>()?;
    m.add_class::<KaoIterator>()?;

    Ok((name, m))
}
