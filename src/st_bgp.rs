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
use crate::bytes::StBytes;
use crate::image::{In256ColIndexedImage, IndexedImage};
use crate::image::tilemap_entry::TilemapEntry;
use crate::python::*;

#[pyclass(module = "skytemple_rust.st_bgp")]
#[derive(Clone)]
pub struct Bgp {
    #[pyo3(get, set)]
    palettes: Vec<Vec<u8>>,
    #[pyo3(get, set)]
    pub tiles: Vec<StBytes>,
    #[pyo3(get, set)]
    tilemap: Vec<Py<TilemapEntry>>
}

#[pymethods]
impl Bgp {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        todo!()
    }

    #[args(ignore_flip_bits = "false")]
    // #[allow(unused_variables)]
    /// Convert all tiles of the BGP to one big image.
    /// The resulting image has one large palette with 256 colors.
    /// If ignore_flip_bits is set, tiles are not flipped. TODO
    ///
    /// The image returned will have the size 256x192.
    pub fn to_pil(&self, ignore_flip_bits: bool) -> PyResult<IndexedImage> {
        todo!()
    }

    #[args(ignore_flip_bits = "false")]
    // #[allow(unused_variables)]
    /// Convert all tiles of the BGP into separate images.
    /// Each image has one palette with 16 colors.
    /// If ignore_flip_bits is set, tiles are not flipped. TODO
    ///
    /// 768 tiles are returned.
    pub fn to_pil_tiled(&self, ignore_flip_bits: bool) -> PyResult<Vec<IndexedImage>> {
        todo!()
    }

    #[args(force_import = "false")]
    #[allow(unused_variables)]
    /// Modify the image data in the BGP by importing the passed image.
    /// The passed image will be split into separate tiles and the tile's palette index
    /// is determined by the first pixel value of each tile in the image. The image
    /// must have a palette containing the 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range the color is replaced with
    /// 0 of the palette (transparent). The "force_import" parameter is ignored.
    ///
    /// The image must have the size 256x192.
    pub fn from_pil(&mut self, pil: In256ColIndexedImage, force_import: bool) {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_bgp")]
#[derive(Clone, Default)]
pub struct BgpWriter;

#[pymethods]
impl BgpWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<Bgp>, py: Python) -> PyResult<StBytes> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bgp_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bgp";
    let m = PyModule::new(py, name)?;
    m.add_class::<Bgp>()?;
    m.add_class::<BgpWriter>()?;

    Ok((name, m))
}
