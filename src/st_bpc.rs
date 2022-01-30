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
use std::io::Cursor;
use std::iter::once;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::bytes::StBytes;
use crate::compression::bpc_image::{BpcImageCompressor, BpcImageDecompressor};
use crate::compression::bpc_tilemap::{BpcTilemapCompressor, BpcTilemapDecompressor};
use crate::image::{In256ColIndexedImage, IndexedImage};
use crate::image::tilemap_entry::{InputTilemapEntry, TilemapEntry};
use crate::python::*;
use crate::st_bpa::Bpa;

const BPC_TILE_DIM: usize = 8;
const BPC_TILEMAP_BYTELEN: usize = 2;
const BPC_BYTELEN_TILE: usize = BPC_TILE_DIM * BPC_TILE_DIM / 2;

#[pyclass(module = "skytemple_rust.st_bpc")]
#[derive(Clone)]
pub struct BpcLayer {
    // The actual number of tiles is one lower
    #[pyo3(get, set)]
    number_tiles: u16,
    // There must be 4 BPAs. (0 for not used)
    #[pyo3(get, set)]
    bpas: [u16; 4],
    // NOTE: Inconsistent with number_tiles. We are including the null chunk in this count.
    #[pyo3(get, set)]
    chunk_tilemap_len: u16,
    #[pyo3(get, set)]
    tiles:  Vec<StBytes>,
    tilemap: Vec<Py<TilemapEntry>>
}

#[pymethods]
impl BpcLayer {
    #[new]
    pub fn new(number_tiles: u16, bpas: [u16; 4], chunk_tilemap_len: u16, tiles: Vec<StBytes>, tilemap: Vec<InputTilemapEntry>) -> Self {
        Self {
            number_tiles, bpas, chunk_tilemap_len, tiles, tilemap: tilemap.into_iter().map(|x| x.into()).collect()
        }
    }
    #[cfg(feature = "python")]
    #[getter]
    fn get_tilemap(&self) -> PyResult<Vec<Py<TilemapEntry>>> {
        // todo: could be optimized
        Ok(self.tilemap.clone())
    }
    #[cfg(feature = "python")]
    #[setter]
    fn set_tilemap(&mut self, value: Vec<InputTilemapEntry>) -> PyResult<()> {
        self.tilemap = value.into_iter().map(|x| x.into()).collect();
        Ok(())
    }
}

#[pyclass(module = "skytemple_rust.st_bpc")]
#[derive(Clone)]
pub struct Bpc {
    #[pyo3(get, set)]
    tiling_width: u16,
    #[pyo3(get, set)]
    tiling_height: u16,
    #[pyo3(get, set)]
    number_of_layers: u8,
    #[pyo3(get, set)]
    layers: Vec<Py<BpcLayer>>
}

#[pymethods]
impl Bpc {
    #[new]
    /// Loads a BPC. A BPC contains two layers of image data. The image data is
    /// grouped in 8x8 tiles, and these tiles are grouped in {tiling_width}x{tiling_height}
    /// chunks using a tile mapping.
    ///
    /// These chunks are referenced in the BMA tile to build the actual image.
    /// The tiling sizes are also stored in the BMA file.
    /// Each tile mapping is also assigned a palette number. The palettes are stored in the BPL
    /// file for the map background and always contain 16 colors.
    pub fn new(data: StBytes, tiling_width: u16, tiling_height: u16, py: Python) -> PyResult<Self> {
        let mut toc_data = data.clone();
        let upper_layer_pnt = toc_data.get_u16_le();
        let lower_layer_pnt = toc_data.get_u16_le();
        let number_of_layers = if lower_layer_pnt > 0 {2} else {1};

        // Depending on the number of layers there are now one or two metadata sections
        // for these layers. The layers are completed by a BMA file that comes with this BPC file!
        // The BMA contains tiling w/h and w/h of the map. See bg_list.dat for mapping.
        let layers = (0..number_of_layers).map(|_| {
            // The actual number of tiles is one lower
            let number_tiles = toc_data.get_u16_le() - 1;
            let bpas = [
                toc_data.get_u16_le(), toc_data.get_u16_le(), toc_data.get_u16_le(), toc_data.get_u16_le()
            ];
            Py::new(py, BpcLayer::new(
                number_tiles, bpas, toc_data.get_u16_le(),
                // dummies:
                vec![], vec![]
            ))
        }).collect::<PyResult<Vec<Py<BpcLayer>>>>()?;

        // Read the first layer image data
        let mut upper_cursor = Cursor::new(data.clone());
        upper_cursor.set_position(upper_layer_pnt as u64);
        let tiles = Self::read_tile_data(BpcImageDecompressor::run(
            &mut upper_cursor,
            (layers[0].borrow(py).number_tiles * 32) as usize
        ))?;
        layers[0].borrow_mut(py).tiles = tiles;
        #[cfg(debug_assertions)]
            {
                let borrowed = layers[0].borrow(py);
                debug_assert_eq!(borrowed.tiles.len() - 1, borrowed.number_tiles as usize)
            }

        if upper_cursor.position() % 2 != 0 {
            upper_cursor.advance(1)
        }

        // Read the first layer tilemap
        let mut l0borrowed = (&layers[0]).borrow_mut(py);
        l0borrowed.tilemap = Self::read_tilemap_data(BpcTilemapDecompressor::run(
            &mut upper_cursor,
            (l0borrowed.chunk_tilemap_len - 1) as usize * (tiling_width * tiling_height) as usize * BPC_TILEMAP_BYTELEN
        ), tiling_width, tiling_height, py)?;
        drop(l0borrowed);

        if number_of_layers > 1 {
            let mut lower_cursor = Cursor::new(data);
            lower_cursor.set_position(lower_layer_pnt as u64);
            // Read the second layer image data
            let tiles = Self::read_tile_data(BpcImageDecompressor::run(
                &mut lower_cursor,
                (layers[1].borrow(py).number_tiles * 32) as usize
            ))?;
            layers[1].borrow_mut(py).tiles = tiles;
            #[cfg(debug_assertions)]
                {
                    let borrowed = layers[1].borrow(py);
                    debug_assert_eq!(borrowed.tiles.len() - 1, borrowed.number_tiles as usize)
                }

            if lower_cursor.position() % 2 != 0 {
                lower_cursor.advance(1)
            }

            // Read the first layer tilemap
            let mut l1borrowed = (&layers[1]).borrow_mut(py);
            l1borrowed.tilemap = Self::read_tilemap_data(BpcTilemapDecompressor::run(
                &mut lower_cursor,
                (l1borrowed.chunk_tilemap_len - 1) as usize * (tiling_width * tiling_height) as usize * BPC_TILEMAP_BYTELEN
            ), tiling_width, tiling_height, py)?;
        }
        Ok(Self {
            tiling_width,
            tiling_height,
            number_of_layers,
            layers
        })
    }

    #[args(width_in_mtiles = "20")]
    pub fn chunks_to_pil(&self, layer: u8, palettes: Vec<StBytes>, width_in_mtiles: u16) -> IndexedImage {
        todo!()
    }
    pub fn single_chunk_to_pil(&self, layer: u8, chunk_idx: u16, palettes: Vec<StBytes>) -> IndexedImage {
        todo!()
    }
    #[args(width_in_mtiles = "20", single_palette = "None")]
    pub fn tiles_to_pil(&self, layer: u8, palettes: Vec<StBytes>, width_in_tiles: u16, single_palette: Option<u16>) -> IndexedImage {
        todo!()
    }
    #[args(width_in_mtiles = "20")]
    pub fn chunks_animated_to_pil(&self, layer: u8, palettes: Vec<StBytes>, bpas: Vec<Option<Bpa>>, width_in_mtiles: usize) -> Vec<IndexedImage> {
        todo!()
    }
    pub fn single_chunk_animated_to_pil(&self, layer: u8, chunk_idx: u16, palettes: Vec<StBytes>, bpas: Vec<Option<Bpa>>) -> Vec<IndexedImage> {
        todo!()
    }
    pub fn pil_to_tiles(&self, layer: u8, image: In256ColIndexedImage) -> PyResult<()> {
        todo!()
    }
    #[args(force_import = "true")]
    pub fn pil_to_chunks(&self, layer: u8, image: In256ColIndexedImage, force_import: bool) -> Vec<StBytes> {
        todo!()
    }
    pub fn get_tile(&self, layer: u8, index: u16) -> TilemapEntry {
        todo!()
    }
    pub fn set_tile(&self, layer: u8, index: u16, tile_mapping: TilemapEntry) -> PyResult<()> {
        todo!()
    }
    pub fn get_chunk(&self, layer: u8, index: u16) -> Vec<TilemapEntry> {
        todo!()
    }
    #[args(contains_null_tile = "false")]
    pub fn import_tiles(&self, layer: u8, tiles: StBytes, contains_null_tile: bool) -> PyResult<()> {
        todo!()
    }
    #[args(contains_null_chunk = "false", correct_tile_ids = "true")]
    pub fn import_tile_mappings(&self, layer: u8, tile_mappings: Vec<TilemapEntry>, contains_null_chunk: bool, correct_tile_ids: bool) -> PyResult<()> {
        todo!()
    }
    pub fn get_bpas_for_layer(&self, layer: u8, bpas_from_bg_list: Vec<Option<Bpa>>) -> Vec<Bpa> {
        todo!()
    }
    pub fn set_chunk(&mut self, layer: u8, index: u16, new_tilemappings: Vec<InputTilemapEntry>) -> PyResult<()> {
        todo!()
    }
    pub fn remove_upper_layer(&self) -> PyResult<()> {
        todo!()
    }
    pub fn add_upper_layer(&self) -> PyResult<()> {
        todo!()
    }
    pub fn process_bpa_change(&self, bpa_index: u8, tiles_bpa_new: u16) -> PyResult<()> {
        todo!()
    }
}

impl Bpc {
    /// Handles the decompressed tile data returned by the BpcImageDecompressor decompressor.
    fn read_tile_data(data: PyResult<Bytes>) -> PyResult<Vec<StBytes>> {
        match data {
            Err(e) => Err(e),
            Ok(tiles) => {
                Ok(once(StBytes::from(vec![0; BPC_BYTELEN_TILE]))
                    .chain(tiles
                        .chunks(BPC_BYTELEN_TILE)
                        .map(StBytes::from)
                    ).collect())

            }
        }
    }
    /// Handles the decompressed tile data returned by the BpcTilemapDecompressor.
    fn read_tilemap_data(data: PyResult<Bytes>, tiling_width: u16, tiling_height: u16, py: Python) -> PyResult<Vec<Py<TilemapEntry>>> {
        match data {
            Err(e) => Err(e),
            Ok(data) => (0..(tiling_width * tiling_height))
                .map(|_| Py::new(py, TilemapEntry::from(0)))
                .chain(data
                    .chunks(BPC_TILEMAP_BYTELEN)
                    .map(|c| {
                        let tme: TilemapEntry = (u16::from_le_bytes(
                            c.try_into().expect("Unexpected internal array conversion error.")
                        ) as usize).into();
                        Py::new(py, tme)
                    })
                ).collect()
        }
    }
}

#[pyclass(module = "skytemple_rust.st_bpc")]
#[derive(Clone, Default)]
pub struct BpcWriter;

#[pymethods]
impl BpcWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Bpc, py: Python) -> PyResult<StBytes> {
        debug_assert!(model.number_of_layers > 0 && model.number_of_layers < 3);
        let end_of_layer_specs = 4 + (12 * model.number_of_layers) as u16;

        // First collect tiles and tilemaps for both layers, so we can calculate the pointers
        let first_tiles = Self::convert_tiles(&model.layers[0], py)?;
        let first_tilemap = Self::convert_tilemap(&model, &model.layers[0], py)?;

        let mut length_of_first_layer = (first_tiles.len() + first_tilemap.len()) as u16;
        // The length is increased by 1 if a padding has to be added:
        if (end_of_layer_specs as usize + first_tiles.len()) % 2 != 0 {
            length_of_first_layer += 1;
        }
        if (end_of_layer_specs + length_of_first_layer) % 2 != 0 {
            length_of_first_layer += 1;
        }

        let mut second_tiles = None;
        let mut second_tilemap = None;
        let mut length_of_second_layer = 0;
        if model.number_of_layers == 2 {
            second_tiles = Some(Self::convert_tiles(&model.layers[1], py)?);
            second_tilemap = Some(Self::convert_tilemap(&model, &model.layers[1], py)?);
            
            length_of_second_layer = (
                second_tiles.as_ref().unwrap().len() + second_tilemap.as_ref().unwrap().len()
            ) as u16;
            // The length is increased by 1 if a padding has to be added:
            if (end_of_layer_specs as usize + second_tiles.as_ref().unwrap().len() % 2) != 0 {
                length_of_second_layer += 1;
            }
            if (end_of_layer_specs + length_of_second_layer) % 2 != 0 {
                length_of_second_layer += 1;
            }
        }

        let mut data = BytesMut::with_capacity(
            // 4 byte header + layer specs + layer data
            end_of_layer_specs as usize + length_of_first_layer as usize + length_of_second_layer as usize
        );

        // upper layer pointer
        data.put_u16_le(end_of_layer_specs);
        // lower layer pointer ( if two layers )
        if model.number_of_layers > 1{
            data.put_u16_le(end_of_layer_specs + length_of_first_layer);
        } else {
            data.put_u16_le(0);
        }

        for layer in model.layers {
            let layer = layer.borrow(py);
            // number tiles + 1
            data.put_u16_le(layer.number_tiles + 1);
            // bpa1-4
            for bpa in layer.bpas {
                data.put_u16_le(bpa);
            }
            // tilemap length
            data.put_u16_le(layer.chunk_tilemap_len);
        }

        debug_assert_eq!(end_of_layer_specs as usize, data.len());
        // layer 1 tiles
        data.put(first_tiles);
        // 2 byte alignment
        if data.len() % 2 != 0 {
            data.put_u8(0);
        }
        // layer 1 tilemap
        data.put(first_tilemap);
        // 2 byte alignment
        if data.len() % 2 != 0 {
            data.put_u8(0);
        }

        // layer 2 tiles
        debug_assert_eq!((end_of_layer_specs + length_of_first_layer) as usize, data.len());
        if let Some(second_tiles) = second_tiles {
            data.put(second_tiles);
            // 2 byte alignment
            if data.len() % 2 != 0 {
                data.put_u8(0);
            }
        }
        if let Some(second_tilemap) = second_tilemap {
            // layer 2 tilemap
            data.put(second_tilemap);
            // 2 byte alignment
            if data.len() % 2 != 0 {
                data.put_u8(0);
            }
            //debug_assert_eq!((end_of_layer_specs + length_of_first_layer + length_of_second_layer) as usize, data.len());
        }

        Ok(data.into())
    }
}

impl BpcWriter {
    fn convert_tiles(layer: &Py<BpcLayer>, py: Python) -> PyResult<Bytes> {
        let layer = layer.borrow(py);
        // Skip first (null tile)
        let data: Bytes = layer.tiles
            .iter()
            .skip(1)
            .map(|x| {
                debug_assert_eq!(BPC_BYTELEN_TILE, x.len());
                x.0.to_vec()
            })
            .flatten()
            .collect();

        BpcImageCompressor::run(data)
    }

    fn convert_tilemap(model: &Bpc, layer: &Py<BpcLayer>, py: Python) -> PyResult<Bytes> {
        let layer = layer.borrow(py);
        let length = (layer.chunk_tilemap_len - 1) * (model.tiling_width * model.tiling_height);
        let mut data = BytesMut::with_capacity(length as usize * BPC_TILEMAP_BYTELEN);
        // Skip first chunk (null)
        for entry in layer.tilemap.iter().skip((model.tiling_width * model.tiling_height) as usize) {
            let entry: TilemapEntry = entry.extract(py)?;
            data.put_u16_le(usize::from(entry) as u16)
        }

        BpcTilemapCompressor::run(data.freeze())
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bpc_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bpc";
    let m = PyModule::new(py, name)?;
    m.add_class::<BpcLayer>()?;
    m.add_class::<Bpc>()?;
    m.add_class::<BpcWriter>()?;

    Ok((name, m))
}
