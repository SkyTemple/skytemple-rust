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
use crate::gettext::gettext;
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::{InputTilemapEntry, TilemapEntry};
use crate::image::{In256ColIndexedImage, InIndexedImage, IndexedImage, Raster};
use crate::python::*;
use crate::st_dpc::input::InputDpc;
use crate::st_dpc::DPC_TILING_DIM;
use crate::st_dpci::input::InputDpci;
use crate::st_dpci::DPCI_TILE_DIM;
use crate::st_dpl::input::InputDpl;
use crate::st_dpl::{DPL_MAX_PAL, DPL_PAL_LEN};
use bytes::{Buf, BufMut, BytesMut};
use itertools::Itertools;

#[cfg(not(feature = "python"))]
use crate::st_dpc::input::DpcProvider;
#[cfg(not(feature = "python"))]
use crate::st_dpci::input::DpciProvider;
#[cfg(not(feature = "python"))]
use crate::st_dpl::input::DplProvider;

const DBG_TILING_DIM: usize = 3;
const DBG_CHUNK_WIDTH: usize = 24;
const DBG_WIDTH_AND_HEIGHT: usize = 32;

#[pyclass(module = "skytemple_rust.st_dbg")]
#[derive(Clone, PartialEq, Eq)]
pub struct Dbg {
    #[pyo3(get, set)]
    pub mappings: Vec<u16>,
}

#[pymethods]
impl Dbg {
    #[new]
    pub fn new(mut data: StBytes) -> PyResult<Self> {
        let mut mappings = Vec::with_capacity(data.len() / 2);
        while data.remaining() >= 2 {
            mappings.push(data.get_u16_le());
        }
        Ok(Self { mappings })
    }

    /// Place the chunk with the given ID at the X and Y position. No error checking is done.
    pub fn place_chunk(&mut self, x: usize, y: usize, chunk_index: u16) {
        let dbg_index = y * DBG_WIDTH_AND_HEIGHT + x;
        self.mappings[dbg_index] = chunk_index;
    }

    pub fn to_pil(
        &self,
        dpc: InputDpc,
        dpci: InputDpci,
        palettes: Vec<Vec<u8>>,
        py: Python,
    ) -> PyResult<IndexedImage> {
        let width_and_height_map = DBG_WIDTH_AND_HEIGHT * DBG_CHUNK_WIDTH;
        let chunks = dpc.0.do_chunks_to_pil(dpci, palettes, 1, py)?;
        let mut fimg = IndexedImage(
            Raster::new(width_and_height_map, width_and_height_map),
            chunks.1.clone(),
        );

        for (i, mt_idx) in self.mappings.iter().enumerate() {
            let x = i % DBG_WIDTH_AND_HEIGHT;
            let y = i / DBG_WIDTH_AND_HEIGHT;
            fimg.0.paste(
                chunks.0.crop(
                    0,
                    *mt_idx as usize * DBG_CHUNK_WIDTH,
                    DBG_CHUNK_WIDTH,
                    DBG_CHUNK_WIDTH,
                ),
                x * DBG_CHUNK_WIDTH,
                y * DBG_CHUNK_WIDTH,
            );
        }

        Ok(fimg)
    }

    #[pyo3(signature = (dpc, dpci, dpl, img, force_import = false))]
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
        mut dpc: InputDpc,
        mut dpci: InputDpci,
        mut dpl: InputDpl,
        img: In256ColIndexedImage,
        force_import: bool,
        py: Python,
    ) -> PyResult<()> {
        let img: IndexedImage = img.extract(py)?;
        let expected_width = DBG_TILING_DIM * DBG_WIDTH_AND_HEIGHT * DPCI_TILE_DIM;
        let expected_height = DBG_TILING_DIM * DBG_WIDTH_AND_HEIGHT * DPCI_TILE_DIM;
        if img.0 .1 != expected_width {
            return Err(create_value_user_error(gettext!(
                "Can not import map background: Width of image must match the expected width: {}px",
                expected_width
            )));
        }
        if img.0 .2 != expected_height {
            return Err(create_value_user_error(gettext!(
                "Can not import map background: Height of image must match the expected height: {}px",
                expected_height
            )));
        }

        let (tiles, palettes, all_possible_tile_mappings) = TiledImage::native_to_tiled(
            img,
            DPL_PAL_LEN as u8,
            DPCI_TILE_DIM,
            expected_width,
            expected_height,
            DBG_TILING_DIM,
            0,
            true,
        )?;

        // Remove any extra colors
        let palettes = palettes
            .0
            .chunks(DPL_PAL_LEN * 3)
            .map(|x| x.to_vec())
            .take(DPL_MAX_PAL)
            .collect::<Vec<Vec<u8>>>();

        dpci.0
            .do_import_tiles(tiles.into_iter().map(|x| x.0.into()).collect(), false, py)?;

        // Build a new list of chunks / tile mappings for the DPC based on repeating chunks
        // in the imported image. Generate chunk mappings.
        let tiles_in_chunk = DBG_TILING_DIM * DBG_TILING_DIM;
        let n_all_chunks = DBG_CHUNK_WIDTH * DBG_CHUNK_WIDTH * tiles_in_chunk;
        let mut chunk_mappings = Vec::with_capacity(n_all_chunks);
        let mut chunk_mappings_counter = 1;
        let mut tile_mappings = Vec::with_capacity(n_all_chunks);
        let chunked = all_possible_tile_mappings
            .into_iter()
            .chunks(tiles_in_chunk);
        for chunk in chunked.into_iter() {
            let mut chunk: Vec<TilemapEntry> = chunk.collect();
            match TiledImage::search_for_chunk(&chunk, &tile_mappings) {
                Some(start_of_existing_chunk) => {
                    chunk_mappings.push((start_of_existing_chunk + 1) as u16)
                }
                None => {
                    tile_mappings.append(&mut chunk);
                    chunk_mappings.push(chunk_mappings_counter);
                    chunk_mappings_counter += 1;
                }
            }
        }

        dpc.0.do_import_tile_mappings(
            tile_mappings
                .into_iter()
                .chunks(DPC_TILING_DIM * DPC_TILING_DIM)
                .into_iter()
                .map(|c| {
                    c.into_iter()
                        .map(|t| Ok(InputTilemapEntry(Py::new(py, t)?)))
                        .collect::<PyResult<Vec<_>>>()
                })
                .collect::<PyResult<Vec<Vec<InputTilemapEntry>>>>()?,
            false,
            true,
            py,
        )?;
        self.mappings = chunk_mappings;

        // Import palettes
        dpl.0.set_palettes(palettes, py)?;
        Ok(())
    }

    #[cfg(feature = "python")]
    fn __richcmp__(&self, other: PyRef<Self>, op: pyo3::basic::CompareOp) -> Py<PyAny> {
        let py = other.py();
        match op {
            pyo3::basic::CompareOp::Eq => (self == &*other).into_py(py),
            pyo3::basic::CompareOp::Ne => (self != &*other).into_py(py),
            _ => py.NotImplemented(),
        }
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

    pub fn write(&self, model: Py<Dbg>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        let mut data = BytesMut::with_capacity(2 * model.mappings.len());
        for m in &model.mappings {
            data.put_u16_le(*m);
        }
        Ok(data.into())
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
