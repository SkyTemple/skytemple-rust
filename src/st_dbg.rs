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
use crate::python::*;
use crate::st_dpc::input::InputDpc;
use crate::st_dpci::input::InputDpci;
use crate::st_dpl::input::InputDpl;

#[pyclass(module = "skytemple_rust.st_dbg")]
#[derive(Clone)]
pub struct Dbg {
    #[pyo3(get, set)]
    pub mappings: Vec<u16>,
}

#[pymethods]
impl Dbg {
    #[allow(unused_variables)]
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        todo!()
    }

    #[allow(unused_variables)]
    pub fn to_pil(
        &self,
        dpc: InputDpc,
        dpci: InputDpci,
        palettes: Vec<Vec<u8>>,
    ) -> PyResult<IndexedImage> {
        todo!()
    }

    #[args(force_import = "false")]
    #[allow(unused_variables)]
    /// Import an entire background from an image.
    /// Changes all tiles, tile mappings and chunks in the DPC/DPCI and re-writes the mappings of the DBG.
    /// Imports the palettes of the image to the DPL.
    ///
    /// The passed image will be split into separate tiles and the tile's palette index in the tile mapping for this
    /// coordinate is determined by the first pixel value of each tile in the image. The image
    /// must have a palette containing up to 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range the color is replaced with
    /// 0 of the palette (transparent). The "force_import" parameter is ignored.
    ///
    /// The input images must have the same dimensions as the DBG (same dimensions as to_pil_single_layer would export).
    pub fn from_pil(
        &mut self,
        dpc: InputDpc,
        dpci: InputDpci,
        dpl: InputDpl,
        img: In256ColIndexedImage,
        force_import: bool,
    ) {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_dbg")]
#[derive(Clone, Default)]
pub struct DbgWriter;

#[pymethods]
impl DbgWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    #[allow(unused_variables)]
    pub fn write(&self, model: Py<Dbg>, py: Python) -> PyResult<StBytes> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_dbg_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_dbg";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dbg>()?;
    m.add_class::<DbgWriter>()?;

    Ok((name, m))
}
