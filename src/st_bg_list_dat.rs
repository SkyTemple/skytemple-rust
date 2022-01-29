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
use crate::python::*;
use crate::rom_source::{RomFileProvider, RomSource};
use crate::st_bma::Bma;
use crate::st_bpa::Bpa;
use crate::st_bpc::Bpc;
use crate::st_bpl::Bpl;

#[pyclass(module = "st_bg_list_dat")]
#[derive(Clone)]
pub struct BgListEntry {
    #[pyo3(get, set)]
    bpl_name: String,
    #[pyo3(get, set)]
    bpc_name: String,
    #[pyo3(get, set)]
    bma_name: String,
    #[pyo3(get, set)]
    bpa_names: Vec<Option<String>>
}

impl BgListEntry {
    pub fn get_bpl<T: RomFileProvider + Sized>(&self, rom_or_directory_root: RomSource<T>) -> Bpl {
        todo!()
    }
    pub fn get_bpc<T: RomFileProvider + Sized>(&self, rom_or_directory_root: RomSource<T>, bpc_tiling_width: u8, bpc_tiling_height: u8) -> Bpc {
        todo!()
    }
    pub fn get_bma<T: RomFileProvider + Sized>(&self, rom_or_directory_root: RomSource<T>) -> Bma {
        todo!()
    }
    pub fn get_bpas<T: RomFileProvider + Sized>(&self, rom_or_directory_root: RomSource<T>) -> Vec<Option<Bpa>> {
        todo!()
    }
}

#[pymethods]
impl BgListEntry {
    #[new]
    pub fn new(bpl_name: String, bpc_name: String, bma_name: String, bpa_names: Vec<Option<String>>) -> Self {
        todo!()
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bpl")]
    pub fn _get_bpl(&self, rom_or_directory_root: RomSource<&PyAny>) -> Bpl {
        self.get_bpl(rom_or_directory_root)
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bpc")]
    #[args(bpc_tiling_width = "3", bpc_tiling_height = "3")]
    pub fn _get_bpc(&self, rom_or_directory_root: RomSource<&PyAny>, bpc_tiling_width: u8, bpc_tiling_height: u8) -> Bpc {
        self.get_bpc(rom_or_directory_root, bpc_tiling_width, bpc_tiling_height)
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bma")]
    pub fn _get_bma(&self, rom_or_directory_root: RomSource<&PyAny>) -> Bma {
        self.get_bma(rom_or_directory_root)
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bpas")]
    pub fn _get_bpas(&self, rom_or_directory_root: RomSource<&PyAny>) -> Vec<Option<Bpa>> {
        self.get_bpas(rom_or_directory_root)
    }
}

#[pyclass(module = "st_bg_list_dat")]
#[derive(Clone)]
pub struct BgList {
    #[pyo3(get, set)]
    level: Vec<Py<BgListEntry>>
}

#[pymethods]
impl BgList {
    #[new]
    pub fn new(data: Vec<u8>) -> Self {
        todo!()
    }
    pub fn find_bma(&self, name: &str) -> usize {
        todo!()
    }
    pub fn find_bpl(&self, name: &str) -> usize {
        todo!()
    }
    pub fn find_bpc(&self, name: &str) -> usize {
        todo!()
    }
    pub fn find_bpa(&self, name: &str) -> usize {
        todo!()
    }
}

#[pyclass(module = "st_bg_list_dat")]
#[derive(Clone)]
pub struct BgListWriter;

#[pymethods]
impl BgListWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: BgList, py: Python) -> PyResult<StBytes> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bg_list_dat_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bg_list_dat";
    let m = PyModule::new(py, name)?;
    m.add_class::<BgListEntry>()?;
    m.add_class::<BgList>()?;
    m.add_class::<BgListWriter>()?;

    Ok((name, m))
}
