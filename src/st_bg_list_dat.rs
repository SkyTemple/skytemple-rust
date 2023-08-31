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
use crate::encoding::{BufEncoding, BufMutEncoding};
use crate::err::convert_io_err;
use crate::python::*;
use crate::rom_source::{RomFileProvider, RomSource};
use crate::st_bma::Bma;
use crate::st_bpa::Bpa;
use crate::st_bpc::Bpc;
use crate::st_bpl::Bpl;
use bytes::BytesMut;
use encoding::codec::ascii::ASCIIEncoding;
use encoding::{DecoderTrap, EncoderTrap};
use std::fs;
use std::path::Path;

const DIR: &str = "MAP_BG/";
const BPC_EXT: &str = ".bpc";
const BPL_EXT: &str = ".bpl";
const BMA_EXT: &str = ".bma";
const BPA_EXT: &str = ".bpa";

#[pyclass(module = "skytemple_rust.st_bg_list_dat")]
#[derive(Clone)]
pub struct BgListEntry {
    #[pyo3(get, set)]
    bpl_name: String,
    #[pyo3(get, set)]
    bpc_name: String,
    #[pyo3(get, set)]
    bma_name: String,
    #[pyo3(get, set)]
    bpa_names: [Option<String>; 8],
}

impl BgListEntry {
    pub fn get_bpl<T: RomFileProvider + Sized>(
        &self,
        rom_or_directory_root: RomSource<T>,
        py: Python,
    ) -> PyResult<Bpl> {
        Bpl::new(
            self.get_file(
                &rom_or_directory_root,
                &format!("{}{}{}", DIR, self.bpl_name.to_lowercase(), BPL_EXT),
            )?,
            py,
        )
    }
    pub fn get_bpc<T: RomFileProvider + Sized>(
        &self,
        rom_or_directory_root: RomSource<T>,
        bpc_tiling_width: u16,
        bpc_tiling_height: u16,
        py: Python,
    ) -> PyResult<Bpc> {
        Bpc::new(
            self.get_file(
                &rom_or_directory_root,
                &format!("{}{}{}", DIR, self.bpc_name.to_lowercase(), BPC_EXT),
            )?,
            bpc_tiling_width,
            bpc_tiling_height,
            py,
        )
    }
    pub fn get_bma<T: RomFileProvider + Sized>(
        &self,
        rom_or_directory_root: RomSource<T>,
    ) -> PyResult<Bma> {
        Bma::new(self.get_file(
            &rom_or_directory_root,
            &format!("{}{}{}", DIR, self.bma_name.to_lowercase(), BMA_EXT),
        )?)
    }
    pub fn get_bpas<T: RomFileProvider + Sized>(
        &self,
        rom_or_directory_root: RomSource<T>,
        py: Python,
    ) -> PyResult<Vec<Option<Bpa>>> {
        let mut v = Vec::with_capacity(self.bpa_names.len());
        for name in &self.bpa_names {
            v.push(match name {
                None => None,
                Some(name) => Some(Bpa::new(
                    self.get_file(
                        &rom_or_directory_root,
                        &format!("{}{}{}", DIR, name.to_lowercase(), BPA_EXT),
                    )?,
                    py,
                )?),
            });
        }
        Ok(v)
    }
    fn get_file<T: RomFileProvider + Sized>(
        &self,
        rom_or_directory_root: &RomSource<T>,
        path: &str,
    ) -> PyResult<StBytes> {
        match rom_or_directory_root {
            RomSource::Folder(f) => fs::read(Path::new(f).join(path))
                .map(StBytes::from)
                .map_err(convert_io_err),
            RomSource::Rom(r) => r.get_file_by_name(path).map(StBytes::from),
        }
    }
}

#[pymethods]
impl BgListEntry {
    #[new]
    pub fn new(
        bpl_name: String,
        bpc_name: String,
        bma_name: String,
        bpa_names: [Option<String>; 8],
    ) -> Self {
        Self {
            bpl_name,
            bpc_name,
            bma_name,
            bpa_names,
        }
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bpl")]
    pub fn _get_bpl(&self, rom_or_directory_root: RomSource<&PyAny>, py: Python) -> PyResult<Bpl> {
        self.get_bpl(rom_or_directory_root, py)
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bpc")]
    #[pyo3(signature = (rom_or_directory_root, bpc_tiling_width = 3, bpc_tiling_height = 3))]
    pub fn _get_bpc(
        &self,
        rom_or_directory_root: RomSource<&PyAny>,
        bpc_tiling_width: u16,
        bpc_tiling_height: u16,
        py: Python,
    ) -> PyResult<Bpc> {
        self.get_bpc(
            rom_or_directory_root,
            bpc_tiling_width,
            bpc_tiling_height,
            py,
        )
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bma")]
    pub fn _get_bma(&self, rom_or_directory_root: RomSource<&PyAny>) -> PyResult<Bma> {
        self.get_bma(rom_or_directory_root)
    }
    #[cfg(feature = "python")]
    #[pyo3(name = "get_bpas")]
    pub fn _get_bpas(
        &self,
        rom_or_directory_root: RomSource<&PyAny>,
        py: Python,
    ) -> PyResult<Vec<Option<Bpa>>> {
        self.get_bpas(rom_or_directory_root, py)
    }
}

#[pyclass(module = "skytemple_rust.st_bg_list_dat")]
#[derive(Clone)]
pub struct BgList {
    #[pyo3(get)]
    level: Vec<Py<BgListEntry>>,
}

#[pymethods]
impl BgList {
    #[new]
    pub fn new(data: Vec<u8>, py: Python) -> PyResult<Self> {
        Ok(Self {
            level: data
                .chunks(11 * 8)
                .map(|mut chunk| {
                    Py::new(
                        py,
                        BgListEntry::new(
                            chunk.get_fixed_string(ASCIIEncoding, 8, DecoderTrap::Strict)?,
                            chunk.get_fixed_string(ASCIIEncoding, 8, DecoderTrap::Strict)?,
                            chunk.get_fixed_string(ASCIIEncoding, 8, DecoderTrap::Strict)?,
                            [
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                                chunk.get_fixed_string_or_null(
                                    ASCIIEncoding,
                                    8,
                                    DecoderTrap::Strict,
                                )?,
                            ],
                        ),
                    )
                })
                .collect::<PyResult<Vec<Py<BgListEntry>>>>()?,
        })
    }

    #[cfg(feature = "python")]
    #[setter(level)]
    fn set_level_attr(&mut self, value: Vec<Py<BgListEntry>>) -> PyResult<()> {
        self.level = value;
        Ok(())
    }

    /// Count all occurrences of this BMA in the list.
    pub fn find_bma(&self, name: &str, py: Python) -> usize {
        self.level
            .iter()
            .fold(0, |acc, pl| acc + (pl.borrow(py).bma_name == name) as usize)
    }

    /// Count all occurrences of this BPL in the list.
    pub fn find_bpl(&self, name: &str, py: Python) -> usize {
        self.level
            .iter()
            .fold(0, |acc, pl| acc + (pl.borrow(py).bpl_name == name) as usize)
    }

    /// Count all occurrences of this BPC in the list.
    pub fn find_bpc(&self, name: &str, py: Python) -> usize {
        self.level
            .iter()
            .fold(0, |acc, pl| acc + (pl.borrow(py).bpc_name == name) as usize)
    }

    /// Count all occurrences of this BPA in the list.
    pub fn find_bpa(&self, name: &str, py: Python) -> usize {
        self.level.iter().fold(0, |acc, pl| {
            acc + pl.borrow(py).bpa_names.iter().fold(0, |iacc, opt_in_name| {
                iacc + match opt_in_name {
                    None => 0,
                    Some(n) => (n == name) as usize,
                }
            })
        })
    }

    /// Adds a level to the level list.
    pub fn add_level(&mut self, level: Py<BgListEntry>) {
        self.level.push(level)
    }

    /// Overwrites a level in the level list.
    pub fn set_level(&mut self, level_id: usize, level: Py<BgListEntry>) {
        self.level[level_id] = level
    }

    /// Overwrites an entry in a level's BPA list.
    pub fn set_level_bpa(
        &mut self,
        level_id: usize,
        bpa_id: usize,
        bpa_name: Option<&str>,
        py: Python,
    ) {
        self.level[level_id].borrow_mut(py).bpa_names[bpa_id] = bpa_name.map(ToString::to_string)
    }
}

#[pyclass(module = "skytemple_rust.st_bg_list_dat")]
#[derive(Clone, Default)]
pub struct BgListWriter;

#[pymethods]
impl BgListWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<BgList>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        // We will need 11 8 character cstrings for each entry:
        let mut data = BytesMut::with_capacity(model.level.len() * 11 * 9);
        for l in &model.level {
            let l = l.borrow(py);
            data.put_fixed_string(&l.bpl_name, ASCIIEncoding, 8, EncoderTrap::Strict)?;
            data.put_fixed_string(&l.bpc_name, ASCIIEncoding, 8, EncoderTrap::Strict)?;
            data.put_fixed_string(&l.bma_name, ASCIIEncoding, 8, EncoderTrap::Strict)?;
            for name in &l.bpa_names {
                match name {
                    None => data.put_fixed_string("", ASCIIEncoding, 8, EncoderTrap::Strict)?,
                    Some(name) => {
                        data.put_fixed_string(name, ASCIIEncoding, 8, EncoderTrap::Strict)?
                    }
                }
            }
        }
        Ok(data.into())
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
