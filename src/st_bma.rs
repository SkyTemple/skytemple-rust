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
use crate::st_bpa::Bpa;
use crate::st_bpc::Bpc;
use crate::st_bpl::Bpl;

#[pyclass(module = "skytemple_rust.st_bma")]
#[derive(Clone)]
pub struct Bma {
    #[pyo3(get, set)]
    map_width_camera: u16,
    #[pyo3(get, set)]
    map_height_camera: u16,
    #[pyo3(get, set)]
    tiling_width: u8,
    #[pyo3(get, set)]
    tiling_height: u8,
    #[pyo3(get, set)]
    map_width_chunks: u16,
    #[pyo3(get, set)]
    map_height_chunks: u16,
    #[pyo3(get, set)]
    number_of_layers: u8,
    #[pyo3(get, set)]
    unk6: u8,
    #[pyo3(get, set)]
    number_of_collision_layers: u8,

    #[pyo3(get, set)]
    layer0: Vec<u8>,
    #[pyo3(get, set)]
    layer1: Option<Vec<u8>>,

    // if unk6:
    #[pyo3(get, set)]
    unknown_data_block: Option<Vec<u8>>,
    // if number_of_collision_layers > 0:
    #[pyo3(get, set)]
    collision: Option<Vec<bool>>,
    // if number_of_collision_layers > 1:
    #[pyo3(get, set)]
    collision2: Option<Vec<bool>>
}

#[pymethods]
impl Bma {
    #[new]
    pub fn new(data: Vec<u8>) -> Self {
        todo!()
    }
    pub fn to_pil_single_layer(&self, bpc: Bpc, palettes: Vec<Vec<u8>>, bpas: Vec<Option<Bpa>>, layer: u8) -> IndexedImage {
        todo!()
    }
    #[allow(clippy::too_many_arguments)]
    #[args(include_collision = "true", include_unknown_data_block = "true", pal_ani = "true", single_frame = "false")]
    pub fn to_pil(
        &self, bpc: Bpc, bpl: Bpl, bpas: Vec<Option<Bpa>>, include_collision: bool,
        include_unknown_data_block: bool, pal_ani: bool, single_frame: bool
    ) -> Vec<IndexedImage> {
        todo!()
    }
    #[allow(clippy::too_many_arguments)]
    #[args(lower_img = "None", upper_img = "None", force_import = "true", how_many_palettes_lower_layer = "16")]
    pub fn from_pil(
        &self, bpc: Bpc, bpl: Bpl, lower_img: Option<In256ColIndexedImage>,
        upper_img: Option<In256ColIndexedImage>, force_import: bool,
        how_many_palettes_lower_layer: u16
    ) -> PyResult<()> {
        todo!()
    }
    pub fn remove_upper_layer(&self) -> PyResult<()> {
        todo!()
    }
    pub fn add_upper_layer(&self) -> PyResult<()> {
        todo!()
    }
    pub fn resize(&self, new_width_chunks: u16, new_height_chunks: u16, new_width_camera: u16, new_height_camera: u16) -> PyResult<()> {
        todo!()
    }
    pub fn place_chunk(&self, layer_id: u8, x: u16, y: u16, chunk_index: u16) -> PyResult<()> {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_bma")]
#[derive(Clone)]
pub struct BmaWriter;

#[pymethods]
impl BmaWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Bma, py: Python) -> PyResult<StBytes> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bma_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bma";
    let m = PyModule::new(py, name)?;
    m.add_class::<Bma>()?;
    m.add_class::<BmaWriter>()?;

    Ok((name, m))
}
