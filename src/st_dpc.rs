/*
 * Copyright 2021-2024 Capypara and the SkyTemple Contributors
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
use std::iter::once;

use bytes::{Buf, BufMut, BytesMut};
use itertools::Itertools;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::bytes::{StBytes, StU8List};
use crate::gettext::gettext;
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::{InputTilemapEntry, ProvidesTilemapEntry, TilemapEntry};
use crate::image::{In256ColIndexedImage, InIndexedImage, IndexedImage, PixelGenerator};
use crate::st_dpci::input::{DpciProvider, InputDpci};
use crate::st_dpci::DPCI_TILE_DIM;
use crate::st_dpl::DPL_MAX_PAL;

pub const DPC_TILING_DIM: usize = 3;
pub const DPC_TILING_DIM_SQUARED: usize = DPC_TILING_DIM * DPC_TILING_DIM;

#[pyclass(module = "skytemple_rust.st_dpc")]
pub struct Dpc {
    #[pyo3(get)]
    pub chunks: Vec<Vec<Py<TilemapEntry>>>,
}

#[pymethods]
impl Dpc {
    #[new]
    pub fn new(data: StBytes, py: Python) -> PyResult<Self> {
        let mut data = data.0;
        let mut chunks = Vec::with_capacity(data.len() / 2 / DPC_TILING_DIM_SQUARED);
        let mut i = 0;
        let mut current_tilemaps = Vec::with_capacity(DPC_TILING_DIM_SQUARED);
        while data.remaining() >= 2 {
            current_tilemaps.push(Py::new(py, TilemapEntry::from(data.get_u16_le() as usize))?);
            i += 1;
            if i % DPC_TILING_DIM_SQUARED == 0 {
                debug_assert_eq!(DPC_TILING_DIM_SQUARED, current_tilemaps.len());
                chunks.push(current_tilemaps);
                current_tilemaps = Vec::with_capacity(DPC_TILING_DIM_SQUARED);
            }
        }
        if !current_tilemaps.is_empty() {
            chunks.push(current_tilemaps);
        }
        Ok(Self { chunks })
    }

    #[setter]
    pub fn set_chunks(&mut self, value: Vec<Vec<InputTilemapEntry>>) -> PyResult<()> {
        self.chunks = value
            .into_iter()
            .map(|v| v.into_iter().map(Into::into).collect())
            .collect();
        Ok(())
    }

    #[pyo3(signature = (dpci, palettes, width_in_mtiles = 16))]
    /// Convert all chunks of the DPC to one big image.
    /// The chunks are all placed next to each other.
    /// The resulting image has one large palette with all palettes merged together.
    ///
    /// To be used with the DPCI file for this dungeon.
    /// To get the palettes, use the data from the DPL file for this dungeon.
    pub fn chunks_to_pil(
        &self,
        dpci: InputDpci,
        palettes: Vec<StU8List>,
        width_in_mtiles: usize,
        py: Python,
    ) -> PyResult<IndexedImage> {
        let width = width_in_mtiles * DPC_TILING_DIM * DPCI_TILE_DIM;
        let height = (self.chunks.len() as f32 / width_in_mtiles as f32).ceil() as usize
            * DPC_TILING_DIM
            * DPCI_TILE_DIM;
        Ok(TiledImage::tiled_to_native(
            self.chunks.iter().flatten().map(|x| x.borrow(py)),
            PixelGenerator::tiled4bpp(dpci.get_tiles(py)?.as_slice()),
            palettes.iter().flat_map(|x| x.iter().copied()),
            DPCI_TILE_DIM,
            width,
            height,
            DPC_TILING_DIM,
        ))
    }

    /// Convert a single chunk of the DPC into a image. For general notes, see chunks_to_pil.
    pub fn single_chunk_to_pil(
        &self,
        chunk_idx: usize,
        dpci: InputDpci,
        palettes: Vec<StU8List>,
        py: Python,
    ) -> PyResult<IndexedImage> {
        Ok(TiledImage::tiled_to_native(
            self.chunks[chunk_idx].iter().map(|x| x.borrow(py)),
            PixelGenerator::tiled4bpp(dpci.get_tiles(py)?.as_slice()),
            palettes.iter().flat_map(|x| x.iter().copied()),
            DPCI_TILE_DIM,
            DPCI_TILE_DIM * DPC_TILING_DIM,
            DPCI_TILE_DIM * DPC_TILING_DIM,
            DPC_TILING_DIM,
        ))
    }

    #[pyo3(signature = (img, force_import = true))]
    #[allow(unused_variables)]
    /// Imports chunks. Format same as for chunks_to_pil.
    /// Replaces tile mappings and returns the new tiles for storing them in a DPCI and the palettes
    /// for storing in a DPL.
    //
    /// The image must have a palette containing the 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range the color is replaced with
    /// 0 of the palette (transparent). The "force_import" parameter is ignored.
    pub fn pil_to_chunks(
        &mut self,
        img: In256ColIndexedImage,
        force_import: bool,
        py: Python,
    ) -> PyResult<(Vec<StBytes>, Vec<StU8List>)> {
        let image = img.extract(py)?;
        let w = image.0 .1;
        let h = image.0 .2;
        let (tiles, palettes, tilemap) =
            TiledImage::native_to_tiled(image, 16, DPCI_TILE_DIM, w, h, DPC_TILING_DIM, 0, true)?;
        // Validate number of palettes
        for tm in &tilemap {
            if tm.pal_idx() > (DPL_MAX_PAL - 1) as u8 {
                return Err(PyValueError::new_err(gettext!(
                    "The image to import can only use the first 12 palettes. Tried to use palette {}", tm.pal_idx()
                )));
            }
        }
        self.chunks = tilemap
            .into_iter()
            .chunks(DPC_TILING_DIM_SQUARED)
            .into_iter()
            .map(|x| {
                x.into_iter()
                    .map(|xx| Py::new(py, xx))
                    .collect::<PyResult<Vec<_>>>()
            })
            .collect::<PyResult<Vec<Vec<Py<TilemapEntry>>>>>()?;
        self.re_fill_chunks(py)?;
        Ok((
            tiles
                .into_iter()
                .map(|x| StBytes::from(x.0))
                .collect::<Vec<StBytes>>(),
            palettes
                .0
                .into_iter()
                .chunks(3 * 16)
                .into_iter()
                .take(DPL_MAX_PAL)
                .map(|x| x.into_iter().collect::<Vec<u8>>().into())
                .collect::<Vec<StU8List>>(),
        ))
    }

    #[pyo3(signature = (tile_mappings, contains_null_chunk = false, correct_tile_ids = true))]
    #[allow(unused_variables)]
    /// Replace the tile mappings of the specified layer.
    /// If contains_null_tile is False, the null chunk is added to the list, at the beginning.
    //
    /// If correct_tile_ids is True, then the tile id of tile_mappings is also increased by one. Use this,  TODO
    /// if you previously used import_tiles with contains_null_tile=False  TODO
    pub fn import_tile_mappings(
        &mut self,
        tile_mappings: Vec<Vec<InputTilemapEntry>>,
        contains_null_chunk: bool,
        correct_tile_ids: bool,
        py: Python,
    ) -> PyResult<()> {
        let tile_mappings_iter = tile_mappings.into_iter().map(|c| {
            c.into_iter()
                .map(|chunk| {
                    let mut chunk: TilemapEntry = chunk.into();
                    if correct_tile_ids {
                        chunk.0 += 1;
                    }
                    Py::new(py, chunk)
                })
                .collect::<PyResult<_>>()
        });
        let tile_mappings: Vec<Vec<Py<TilemapEntry>>> = if !contains_null_chunk {
            once(Ok(empty_chunk(py)?))
                .chain(tile_mappings_iter)
                .collect::<PyResult<_>>()?
        } else {
            tile_mappings_iter.collect::<PyResult<_>>()?
        };
        self.chunks = tile_mappings;
        self.re_fill_chunks(py)
    }
}

impl Dpc {
    fn re_fill_chunks(&mut self, py: Python) -> PyResult<()> {
        if self.chunks.len() > 400 {
            Err(PyValueError::new_err(gettext(
                "A dungeon background or tilemap can not have more than 400 chunks.",
            )))
        } else {
            for _ in 0..400 - self.chunks.len() {
                self.chunks.push(empty_chunk(py)?);
            }
            Ok(())
        }
    }
}

#[pyclass(module = "skytemple_rust.st_dpc")]
#[derive(Clone, Default)]
pub struct DpcWriter;

#[pymethods]
impl DpcWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<Dpc>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        let all_tilemaps = model.chunks.iter().flatten().collect::<Vec<_>>();
        let mut data = BytesMut::with_capacity(all_tilemaps.len() * 2);
        for tm in all_tilemaps {
            data.put_u16_le(tm.borrow(py).to_int() as u16);
        }
        Ok(data.into())
    }
}

pub(crate) fn create_st_dpc_module(py: Python) -> PyResult<(&str, Bound<'_, PyModule>)> {
    let name: &'static str = "skytemple_rust.st_dpc";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dpc>()?;
    m.add_class::<DpcWriter>()?;

    Ok((name, m))
}

/////////////////////////
/////////////////////////
// DPCs as inputs (for compatibility of including other DPC implementations from Python)

pub mod input {
    use crate::bytes::StU8List;
    use crate::image::tilemap_entry::InputTilemapEntry;
    use crate::image::{In256ColIndexedImage, InIndexedImage, IndexedImage};
    use crate::st_dpc::Dpc;
    use crate::st_dpci::input::InputDpci;
    use enum_dispatch::enum_dispatch;
    use pyo3::prelude::*;
    use pyo3::types::{PyList, PyTuple};
    use pyo3::IntoPyObjectExt;

    #[enum_dispatch(InputDpc)]
    pub trait DpcProvider {
        fn do_chunks_to_pil(
            &self,
            dpci: InputDpci,
            palettes: Vec<StU8List>,
            width_in_mtiles: usize,
            py: Python,
        ) -> PyResult<IndexedImage>;

        fn do_import_tile_mappings(
            &mut self,
            tile_mappings: Vec<Vec<InputTilemapEntry>>,
            contains_null_chunk: bool,
            correct_tile_ids: bool,
            py: Python,
        ) -> PyResult<()>;
    }

    impl DpcProvider for Py<Dpc> {
        fn do_chunks_to_pil(
            &self,
            dpci: InputDpci,
            palettes: Vec<StU8List>,
            width_in_mtiles: usize,
            py: Python,
        ) -> PyResult<IndexedImage> {
            self.borrow(py)
                .chunks_to_pil(dpci, palettes, width_in_mtiles, py)
        }

        fn do_import_tile_mappings(
            &mut self,
            tile_mappings: Vec<Vec<InputTilemapEntry>>,
            contains_null_chunk: bool,
            correct_tile_ids: bool,
            py: Python,
        ) -> PyResult<()> {
            self.borrow_mut(py).import_tile_mappings(
                tile_mappings,
                contains_null_chunk,
                correct_tile_ids,
                py,
            )
        }
    }

    impl DpcProvider for PyObject {
        fn do_chunks_to_pil(
            &self,
            dpci: InputDpci,
            palettes: Vec<StU8List>,
            width_in_mtiles: usize,
            py: Python,
        ) -> PyResult<IndexedImage> {
            let args = PyTuple::new(
                py,
                [
                    dpci.into_py_any(py)?,
                    palettes.into_py_any(py)?,
                    width_in_mtiles.into_py_any(py)?,
                ],
            )?;
            let img: In256ColIndexedImage = self
                .call_method1(py, "chunks_to_pil", args)
                .and_then(|v| v.extract(py))?;
            img.extract(py)
        }

        fn do_import_tile_mappings(
            &mut self,
            tile_mappings: Vec<Vec<InputTilemapEntry>>,
            contains_null_chunk: bool,
            correct_tile_ids: bool,
            py: Python,
        ) -> PyResult<()> {
            let args = PyTuple::new(
                py,
                [
                    PyList::new(
                        py,
                        &tile_mappings
                            .into_iter()
                            .map(|v| {
                                v.into_iter()
                                    .map(|vv| vv.0.into_py_any(py))
                                    .collect::<PyResult<Vec<_>>>()
                                    .and_then(|vok| PyList::new(py, vok))
                            })
                            .collect::<PyResult<Vec<_>>>()?,
                    )?
                    .into_py_any(py)?,
                    contains_null_chunk.into_py_any(py)?,
                    correct_tile_ids.into_py_any(py)?,
                ],
            )?;
            self.call_method1(py, "import_tile_mappings", args)
                .map(|_| ())
        }
    }

    #[enum_dispatch]
    #[derive(FromPyObject, IntoPyObject)]
    pub enum InputDpc {
        Rs(Py<Dpc>),
        Py(PyObject),
    }
}

#[inline(always)]
fn empty_chunk(py: Python) -> PyResult<Vec<Py<TilemapEntry>>> {
    Ok(vec![
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
        Py::new(py, TilemapEntry::default())?,
    ])
}
