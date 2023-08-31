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

use crate::bytes::{StBytes, StBytesMut};
use crate::gettext::gettext;
use crate::image::tiled::TiledImage;
use crate::image::{In16ColSolidIndexedImage, InIndexedImage, IndexedImage, PixelGenerator};
use crate::python::*;
use crate::st_at_common::{CommonAt, COMMON_AT_MUST_COMPRESS_3};
use arr_macro::arr;
use bytes::{Buf, BufMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Cursor;
use std::mem::swap;
use std::vec;

#[pyclass(module = "skytemple_rust.st_kao")]
#[derive(Clone)]
pub struct KaoImage {
    pal_data: StBytesMut,
    compressed_img_data: StBytesMut,
}

impl KaoImage {
    const KAO_IMG_PAL_B_SIZE: usize = 48; // Size of KaoImage palette block in bytes (16*3)
    const TILE_DIM: usize = 8;
    const IMG_DIM: usize = 40;

    pub fn new(raw_data: &[u8]) -> PyResult<Self> {
        let cont_len: usize =
            if let Some(x) = CommonAt::cont_size(&raw_data[Self::KAO_IMG_PAL_B_SIZE..], 0) {
                x as usize
            } else {
                return Err(exceptions::PyValueError::new_err(
                    "Invalid Kao image data; image not an AT container.",
                ));
            };
        // palette size + at container size
        Ok(Self {
            pal_data: StBytesMut::from(&raw_data[..Self::KAO_IMG_PAL_B_SIZE]),
            compressed_img_data: StBytesMut::from(
                &raw_data[Self::KAO_IMG_PAL_B_SIZE..Self::KAO_IMG_PAL_B_SIZE + cont_len],
            ),
        })
    }
    /// Create a new KaoImage from image data.
    pub fn new_from_img(source: IndexedImage) -> PyResult<Self> {
        let (pal, img) = Self::bitmap_to_kao(source)?;
        debug_assert_eq!(Self::KAO_IMG_PAL_B_SIZE, pal.len());
        Ok(Self {
            compressed_img_data: img,
            pal_data: pal,
        })
    }
    /// Create from raw compressed image and palette data.
    pub fn create_from_raw(cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        Ok(Self {
            pal_data: StBytesMut::from(pal),
            compressed_img_data: StBytesMut::from(cimg),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = u8> {
        self.pal_data
            .0
            .clone()
            .into_iter()
            .chain(self.compressed_img_data.0.clone())
    }

    fn bitmap_to_kao(source: IndexedImage) -> PyResult<(StBytesMut, StBytesMut)> {
        let (img, pal) =
            TiledImage::native_to_tiled_seq(source, Self::TILE_DIM, Self::IMG_DIM, Self::IMG_DIM)?;
        let (img, pal) = Self::reorder_palette(TiledImage::unpack_tiles::<StBytesMut>(img), pal);
        let compressed_img = CommonAt::compress(&img, COMMON_AT_MUST_COMPRESS_3.iter())?;
        // Check image size
        if compressed_img.len() > 800 {
            return Err(exceptions::PyValueError::new_err(gettext!(
                "This portrait does not compress well, the result size is greater than 800 bytes ({} bytes total).\n If you haven't done already, try applying the 'ProvideATUPXSupport' to install an optimized compression algorithm, which might be able to better compress this image.",
                compressed_img.len()
            )));
        }
        Ok((pal, compressed_img))
    }

    /// Tries to reorder the palette to have a more favorable data
    /// configuration for the PX algorithm
    fn reorder_palette(in_img: StBytesMut, in_pal: StBytesMut) -> (StBytesMut, StBytesMut) {
        let mut pairs: HashMap<(u8, u8), usize> = HashMap::with_capacity(in_img.len());
        for x in 0..in_img.len() - 1 {
            let l = [
                in_img[x] % 16,
                in_img[x] / 16,
                in_img[x + 1] % 16,
                in_img[x + 1] / 16,
            ];
            let count_l0 = l.iter().filter(|&x| *x == l[0]).count();
            if count_l0 == 3 || (count_l0 == 1 || l.iter().filter(|&x| *x == l[1]).count() == 3) {
                let mut a = l[0];
                let mut b = l[0];
                for v in l {
                    if v != a {
                        b = v;
                        break;
                    }
                }
                if a >= b {
                    swap(&mut a, &mut b);
                }
                match pairs.entry((a, b)) {
                    Entry::Occupied(mut e) => e.insert(e.get() + 1),
                    Entry::Vacant(e) => *e.insert(1),
                };
            }
        }
        let mut new_order: Vec<i16> = Vec::with_capacity(pairs.len() * 4);
        new_order.push(0);
        let mut sorted_pairs = pairs.into_iter().collect::<Vec<((u8, u8), usize)>>();
        sorted_pairs.sort_by_key(|((_k1, _k2), v)| *v);
        for ((k0, k1), _) in sorted_pairs.into_iter().rev() {
            let k0_in_no = new_order.contains(&(k0 as i16));
            let k1_in_no = new_order.contains(&(k1 as i16));
            if k0_in_no && k1_in_no {
                continue;
            }
            if k0_in_no || k1_in_no {
                let to_check: i16;
                let to_add: i16;
                if k0_in_no {
                    to_check = k0 as i16;
                    to_add = k1 as i16;
                } else {
                    to_check = k1 as i16;
                    to_add = k0 as i16;
                }
                let i = new_order.iter().position(|&r| r == to_check).unwrap();
                if i > 0 {
                    if new_order[i - 1] == -1 {
                        new_order.insert(i, to_add)
                    }
                    if new_order.len() == i + 1 || new_order[i + 1] == -1 {
                        new_order.insert(i + 1, to_add)
                    }
                }
            } else {
                new_order.push(-1);
                new_order.push(k0 as i16);
                new_order.push(k1 as i16);
            }
        }
        new_order.retain(|x| *x != -1);
        for x in 0..16 {
            if !new_order.contains(&x) {
                new_order.push(x);
            }
        }
        (
            in_img
                .into_iter()
                .map(|v| {
                    ((new_order.iter().position(|x| *x as u8 == v % 16).unwrap())
                        + (new_order.iter().position(|x| *x as u8 == v / 16).unwrap()) * 16)
                        as u8
                })
                .collect::<StBytesMut>(),
            new_order
                .into_iter()
                .flat_map(|v| &in_pal[(v * 3) as usize..(v * 3 + 3) as usize])
                .copied()
                .collect::<StBytesMut>(),
        )
    }
}

#[pymethods]
impl KaoImage {
    #[cfg(feature = "python")]
    #[pyo3(name = "clone")]
    fn _clone(&self) -> Self {
        self.clone()
    }
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "create_from_raw")]
    fn _create_from_raw(_cls: &PyType, cimg: &[u8], pal: &[u8]) -> PyResult<Self> {
        Self::create_from_raw(cimg, pal)
    }
    /// Returns the portrait as a PIL image with a 16-color color palette.
    pub fn get(&self) -> PyResult<IndexedImage> {
        Ok(TiledImage::tiled_to_native_seq(
            PixelGenerator::pack4bpp(
                &CommonAt::decompress(&self.compressed_img_data)?,
                Self::TILE_DIM,
            ),
            self.pal_data.iter().copied(),
            Self::TILE_DIM,
            Self::IMG_DIM,
            Self::IMG_DIM,
        ))
    }
    pub fn size(&self) -> PyResult<usize> {
        Ok(Self::KAO_IMG_PAL_B_SIZE + self.compressed_img_data.len())
    }
    /// Sets the portrait using image data with 16-bit color palette as input.
    pub fn set(&mut self, py: Python, source: In16ColSolidIndexedImage) -> PyResult<()> {
        let (pal, img) = Self::bitmap_to_kao(source.extract(py)?)?;
        debug_assert_eq!(Self::KAO_IMG_PAL_B_SIZE, pal.len());
        self.pal_data = pal;
        self.compressed_img_data = img;
        Ok(())
    }
    /// Returns raw image data and palettes.
    pub fn raw(&self) -> PyResult<(&[u8], &[u8])> {
        Ok((&self.compressed_img_data[..], &self.pal_data[..]))
    }
}

#[pyclass(module = "skytemple_rust.st_kao")]
#[derive(Clone)]
/// A container for portrait images.
pub struct Kao {
    portraits: Vec<[Option<Py<KaoImage>>; Self::PORTRAIT_SLOTS]>,
}

#[pymethods]
impl Kao {
    const PORTRAIT_SLOTS: usize = 40;
    const TOC_PADDING: usize = 160;

    #[new]
    #[allow(clippy::needless_range_loop)]
    /// Reads a container from the binary KAO format.
    pub fn new(raw_data: &[u8], py: Python) -> PyResult<Self> {
        let mut data = Cursor::new(raw_data);
        let mut portraits: Vec<[Option<Py<KaoImage>>; Self::PORTRAIT_SLOTS]> =
            Vec::with_capacity(1600);
        // First 160 bytes are padding
        data.advance(Self::TOC_PADDING);
        let mut first_pointer = 0;
        while first_pointer == 0 || data.position() < first_pointer {
            let mut species: [Option<Py<KaoImage>>; Self::PORTRAIT_SLOTS] = arr![None; 40];
            for i in 0..Self::PORTRAIT_SLOTS {
                let pointer = data.get_i32_le();
                if pointer > 0 {
                    if first_pointer == 0 {
                        first_pointer = pointer as u64;
                    }
                    species[i] = Some(Py::new(py, KaoImage::new(&raw_data[pointer as usize..])?)?);
                }
            }
            portraits.push(species);
        }
        if data.position() > first_pointer {
            return Err(exceptions::PyValueError::new_err("Corrupt KAO TOC."));
        }
        Ok(Self { portraits })
    }
    /// Creates a new empty KAO with the specified number of entries.
    #[classmethod]
    pub fn create_new(_cls: &PyType, number_entries: usize) -> Self {
        Self {
            portraits: vec![arr![None; 40]; number_entries],
        }
    }
    /// Returns the number of entries.
    pub fn n_entries(&self) -> usize {
        self.portraits.len()
    }
    /// Enlarges the table of contents of the Kao to the new size.
    pub fn expand(&mut self, new_size: usize) -> PyResult<()> {
        if new_size < self.portraits.len() {
            return Err(exceptions::PyValueError::new_err(format!(
                "Can't reduce size from {} to {}",
                self.portraits.len(),
                new_size
            )));
        }
        for _ in self.portraits.len()..new_size {
            self.portraits.push(arr![None; 40]);
        }
        Ok(())
    }
    /// Gets an image from the Kao catalog.
    pub fn get(&self, index: usize, subindex: usize) -> PyResult<Option<Py<KaoImage>>> {
        if index < self.portraits.len() {
            if subindex < Self::PORTRAIT_SLOTS {
                return Ok(self.portraits[index][subindex].clone());
            }
            return Err(exceptions::PyValueError::new_err(format!(
                "The subindex requested must be between 0 and {}",
                Self::PORTRAIT_SLOTS
            )));
        }
        Err(exceptions::PyValueError::new_err(format!(
            "The index requested must be between 0 and {}",
            self.portraits.len()
        )))
    }
    /// Set the KaoImage at the specified location.
    pub fn set(&mut self, index: usize, subindex: usize, img: Py<KaoImage>) -> PyResult<()> {
        if index <= self.portraits.len() {
            if subindex < Self::PORTRAIT_SLOTS {
                self.portraits[index][subindex] = Some(img);
                return Ok(());
            }
            return Err(exceptions::PyValueError::new_err(format!(
                "The subindex requested must be between 0 and {}",
                Self::PORTRAIT_SLOTS
            )));
        }
        Err(exceptions::PyValueError::new_err(format!(
            "The index requested must be between 0 and {}",
            self.portraits.len()
        )))
    }
    /// Creates a new KaoImage at the specified location from image data.
    pub fn set_from_img(
        &mut self,
        py: Python,
        index: usize,
        subindex: usize,
        img: In16ColSolidIndexedImage,
    ) -> PyResult<()> {
        if index <= self.portraits.len() {
            if subindex < Self::PORTRAIT_SLOTS {
                self.portraits[index][subindex] =
                    Some(Py::new(py, KaoImage::new_from_img(img.extract(py)?)?)?);
                return Ok(());
            }
            return Err(exceptions::PyValueError::new_err(format!(
                "The subindex requested must be between 0 and {}",
                Self::PORTRAIT_SLOTS
            )));
        }
        Err(exceptions::PyValueError::new_err(format!(
            "The index requested must be between 0 and {}",
            self.portraits.len()
        )))
    }
    /// Removes a KaoImage, if it exists.
    pub fn delete(&mut self, index: usize, subindex: usize) -> PyResult<()> {
        if index <= self.portraits.len() && subindex < Self::PORTRAIT_SLOTS {
            self.portraits[index][subindex] = None
        }
        Ok(())
    }
    #[cfg(not(feature = "python"))]
    /// Iterates over all KaoImages.
    pub fn iter(&self) -> PyResult<KaoIterator> {
        let mut reference = Box::new(
            self.portraits
                .clone()
                .into_iter()
                .map(|s| s.to_vec().into_iter()),
        );
        let iter_outer = reference.next();
        Ok(KaoIterator {
            reference,
            iter_outer,
            i_outer: 0,
            i_inner: -1,
        })
    }

    #[cfg(feature = "python")]
    /// Iterates over all KaoImages.
    #[allow(clippy::unnecessary_to_owned)]
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<KaoIterator>> {
        let mut reference = Box::new(
            slf.portraits
                .clone()
                .into_iter()
                .map(|s| s.to_vec().into_iter()),
        );
        let iter_outer = reference.next();
        Py::new(
            slf.py(),
            KaoIterator {
                reference,
                iter_outer,
                i_outer: 0,
                i_inner: -1,
            },
        )
    }
}

#[pyclass(module = "skytemple_rust.st_kao", unsendable)]
pub struct KaoIterator {
    reference: Box<dyn Iterator<Item = std::vec::IntoIter<Option<Py<KaoImage>>>>>,
    iter_outer: Option<vec::IntoIter<Option<Py<KaoImage>>>>,
    i_outer: u32,
    i_inner: i32,
}

impl Iterator for KaoIterator {
    type Item = (u32, u32, Option<Py<KaoImage>>);

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

#[pymethods]
#[cfg(feature = "python")]
impl KaoIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __next__(
        mut slf: PyRefMut<Self>,
    ) -> pyo3::iter::IterNextOutput<(u32, u32, Option<Py<KaoImage>>), &'static str> {
        match slf.next() {
            Some(x) => pyo3::iter::IterNextOutput::Yield(x),
            None => pyo3::iter::IterNextOutput::Return(""),
        }
    }
}

#[pyclass(module = "skytemple_rust.st_kao")]
#[derive(Clone, Default)]
pub struct KaoWriter; // No fields.

#[pymethods]
impl KaoWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<Kao>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        let toc_len = Kao::TOC_PADDING + (model.portraits.len() * Kao::PORTRAIT_SLOTS * 4);
        let mut toc: Vec<u8> = Vec::with_capacity(toc_len);
        toc.put_slice(&[0; Kao::TOC_PADDING]);
        let mut current_image_end = toc_len as i32;
        let mut portrait_data = model
            .portraits
            .iter()
            .flatten()
            .filter_map(|opt| {
                match opt {
                    None => {
                        // Write TOC
                        toc.put_i32_le(-current_image_end);
                        None
                    }
                    Some(v) => {
                        // Write TOC
                        toc.put_i32_le(current_image_end);
                        let data: Vec<u8> = v.borrow(py).iter().collect();
                        current_image_end += data.len() as i32;
                        Some(data)
                    }
                }
            })
            .flatten()
            .collect::<Vec<u8>>();
        toc.append(&mut portrait_data);
        Ok(StBytes::from(toc))
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
