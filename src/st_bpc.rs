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
use std::mem::swap;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::bytes::StBytes;
use crate::compression::bpc_image::{BpcImageCompressor, BpcImageDecompressor};
use crate::compression::bpc_tilemap::{BpcTilemapCompressor, BpcTilemapDecompressor};
use crate::image::{In16ColIndexedImage, In256ColIndexedImage, IndexedImage, InIndexedImage, PixelGenerator, Tile};
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::{InputTilemapEntry, TilemapEntry};
use crate::python::*;
use crate::st_bpa::input::InputBpa;

const BPC_TILE_DIM: usize = 8;
const BPC_TILEMAP_BYTELEN: usize = 2;
const BPC_BYTELEN_TILE: usize = BPC_TILE_DIM * BPC_TILE_DIM / 2;

#[pyclass(module = "skytemple_rust.st_bpc")]
#[derive(Clone, Default)]
pub struct BpcLayer {
    // The actual number of tiles is one lower
    #[pyo3(get, set)]
    pub number_tiles: u16,
    // There must be 4 BPAs. (0 for not used)
    #[pyo3(get, set)]
    pub bpas: [u16; 4],
    // NOTE: Inconsistent with number_tiles. We are including the null chunk in this count.
    #[pyo3(get, set)]
    pub chunk_tilemap_len: u16,
    #[pyo3(get, set)]
    pub tiles:  Vec<StBytes>,
    pub tilemap: Vec<Py<TilemapEntry>>
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
    pub tiling_width: u16,
    #[pyo3(get, set)]
    pub tiling_height: u16,
    #[pyo3(get, set)]
    pub number_of_layers: u8,
    #[pyo3(get, set)]
    pub layers: Vec<Py<BpcLayer>>
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
        #[allow(unused_mut)]
        let mut layers = (0..number_of_layers).map(|_| {
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
        #[cfg(not(feature = "python"))]
            let mut l0borrowed = layers[0].borrow_mut(py);
        #[cfg(feature = "python")]
            let mut l0borrowed = (&layers[0]).borrow_mut(py);
        l0borrowed.tilemap = Self::read_tilemap_data(BpcTilemapDecompressor::run(
            &mut upper_cursor,
            (l0borrowed.chunk_tilemap_len - 1) as usize * (tiling_width * tiling_height) as usize * BPC_TILEMAP_BYTELEN
        ), tiling_width, tiling_height, py)?;
        #[cfg(feature = "python")]
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

            // Read the second layer tilemap
            let mut l1borrowed = layers[1].borrow_mut(py);
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

    /// Convert all chunks of the BPC to one big image.
    /// The chunks are all placed next to each other.
    /// The resulting image has one large palette with all palettes merged together.
    ///
    /// To get the palettes, use the data from the BPL file for this map background.
    ///
    /// The first chunk is a NULL chunk. It is always empty, even when re-imported.
    ///
    /// Does NOT export BPA tiles. Chunks that reference BPA tiles are replaced with empty tiles.
    /// The mapping to BPA tiles has to be done programmatically using set_tile or set_chunk.
    #[args(width_in_mtiles = "20")]
    pub fn chunks_to_pil(&self, layer_id: usize, palettes: Vec<StBytes>, width_in_mtiles: usize, py: Python) -> IndexedImage {
        let layer = self.layers[layer_id].borrow(py);
        let width = width_in_mtiles * self.tiling_width as usize * BPC_TILE_DIM;
        let height = ((layer.chunk_tilemap_len as f32 / width_in_mtiles as f32).ceil()) as usize * self.tiling_height as usize * BPC_TILE_DIM;

        debug_assert_eq!(self.tiling_width, self.tiling_height);
        TiledImage::tiled_to_native(
            layer.tilemap.iter().map(|x| x.borrow(py)),
            PixelGenerator::tiled4bpp(&layer.tiles[..]),
            palettes.iter().flat_map(|x| x.iter().copied()),
            BPC_TILE_DIM, width, height, self.tiling_width as usize
        )
    }

    /// Convert a single chunk of the BPC to one big PIL image. For general notes, see chunks_to_pil.
    /// Does NOT export BPA tiles. Chunks that reference BPA tiles are replaced with empty tiles.
    pub fn single_chunk_to_pil(&self, layer_id: usize, chunk_idx: usize, palettes: Vec<StBytes>, py: Python) -> IndexedImage {
        let layer = self.layers[layer_id].borrow(py);
        let mtidx = chunk_idx * self.tiling_width as usize * self.tiling_height as usize;
        debug_assert_eq!(self.tiling_width, self.tiling_height);
        TiledImage::tiled_to_native(
            layer.tilemap.iter().skip(mtidx).take(9).map(|x| x.borrow(py)),
            PixelGenerator::tiled4bpp(&layer.tiles[..]),
            palettes.iter().flat_map(|x| x.iter().copied()),
            BPC_TILE_DIM,
            BPC_TILE_DIM * self.tiling_width as usize,
            BPC_TILE_DIM * self.tiling_height as usize,
            self.tiling_width as usize
        )
    }
    /// Convert all individual tiles of the BPC into one image.
    /// The image contains all tiles next to each other, the image width is tile_width tiles.
    /// The resulting image has one large palette with all palettes merged together.
    //
    /// The tiles are exported with the palette of the first placed tile or 0 if tile is not in tilemap,
    /// for easier editing. The result image contains a palette that consists of all palettes merged together.
    //
    /// If single_palette is not None, all palettes are exported using the palette no. stored in single_palette.
    //
    /// The first tile is a NULL tile. It is always empty, even when re-imported.
    #[args(width_in_mtiles = "20", single_palette = "None")]
    pub fn tiles_to_pil(&self, layer_id: usize, palettes: Vec<StBytes>, width_in_tiles: usize, single_palette: Option<u8>, py: Python) -> IndexedImage {
        let layer = self.layers[layer_id].borrow(py);
        let tilemap = (0..layer.number_tiles + 1).into_iter().map(|i| TilemapEntry(
            i as usize, false, false, match single_palette {
                None => self.get_palette_for_tile(layer_id, i as usize, py),
                Some(p) => p
            }
        ));
        let width = width_in_tiles * BPC_TILE_DIM as usize;
        let height = (((layer.number_tiles + 1) as f32 / width_in_tiles as f32).ceil()) as usize * BPC_TILE_DIM;
        TiledImage::tiled_to_native(
            tilemap,
            PixelGenerator::tiled4bpp(&layer.tiles[..]),
            palettes.iter().flat_map(|x| x.iter().copied()),
            BPC_TILE_DIM, width, height, 1
        )
    }

    //// Exports chunks. For general notes see chunks_to_pil.
    ///
    /// However this method also exports BPA animated tiles referenced in the tilemap. This means it returns
    /// a set of images containing the chunks, including BPC tiles and BPA tiles. BPA tiles are animated, and
    /// each image contains one frame of the animation.
    ///
    /// The method does not care about frame speeds. Each step of animation is simply returned as a new image,
    /// so if BPAs use different frame speeds, this is ignored; they effectively run at the same speed.
    /// If BPAs are using a different amount of frames per tile, the length of returned list of images will be the lowest
    /// common denominator of the different frame lengths.
    ///
    /// Does not include palette animations. You can apply them by switching out the palettes of the PIL
    /// using the information provided by the BPL.
    ///
    /// The list of bpas must be the one contained in the bg_list. It needs to contain 8 slots, with empty
    /// slots being None.
    #[args(width_in_mtiles = "20")]
    pub fn chunks_animated_to_pil(&self, layer: u8, palettes: Vec<StBytes>, bpas: Vec<Option<InputBpa>>, width_in_mtiles: usize) -> Vec<IndexedImage> {
        /// TODO: The speed can be increased if we only re-render the changed animated tiles instead!
        //         ldata = self.layers[layer]
        //         # First check if we even have BPAs to use
        //         is_using_bpa = len(bpas) > 0 and any(x > 0 for x in ldata.bpas)
        //         if not is_using_bpa:
        //             # Simply build the single chunks frame
        //             return [self.chunks_to_pil(layer, palettes, width_in_mtiles)]
        //
        //         bpa_animation_indices = [0, 0, 0, 0]
        //         frames = []
        //
        //         orig_len = len(ldata.tiles)
        //         while True:  # Ended by check at end (do while)
        //             previous_end_of_tiles = orig_len
        //             # For each frame: Insert all BPA current frame tiles into their slots
        //             for bpaidx, bpa in enumerate(self.get_bpas_for_layer(layer, bpas)):
        //
        //                 # Add the BPA tiles for this frame to the set of BPC tiles:
        //                 new_end_of_tiles = previous_end_of_tiles + bpa.number_of_tiles
        //                 ldata.tiles[previous_end_of_tiles:new_end_of_tiles] = bpa.tiles_for_frame(bpa_animation_indices[bpaidx])
        //
        //                 previous_end_of_tiles = new_end_of_tiles
        //                 bpa_animation_indices[bpaidx] += 1
        //                 bpa_animation_indices[bpaidx] %= bpa.number_of_frames
        //
        //             frames.append(self.chunks_to_pil(layer, palettes, width_in_mtiles))
        //             # All animations have been played, we are done!
        //             if bpa_animation_indices == [0, 0, 0, 0]:
        //                 break
        //
        //         # RESET the layer's tiles to NOT include the BPA tiles!
        //         ldata.tiles = ldata.tiles[:orig_len]
        //         return frames
        todo!()
    }

    /// Exports a single chunk. For general notes see chunks_to_pil. For notes regarding the animation see
    /// chunks_animated_to_pil.
    ///
    pub fn single_chunk_animated_to_pil(&self, layer: u8, chunk_idx: u16, palettes: Vec<StBytes>, bpas: Vec<Option<InputBpa>>) -> Vec<IndexedImage> {
        /// TODO: Code duplication with chunks_animated_to_pil. Could probably be refactored.
        //         ldata = self.layers[layer]
        //         # First check if we even have BPAs to use
        //         is_using_bpa = len(bpas) > 0 and any(x > 0 for x in ldata.bpas)
        //         if is_using_bpa:
        //             # Also check if any of the tiles in the chunk even uses BPA tiles
        //             is_using_bpa = False
        //             for tilem in self.get_chunk(layer, chunk_idx):
        //                 if tilem.idx > ldata.number_tiles:
        //                     is_using_bpa = True
        //                     break
        //         if not is_using_bpa:
        //             # Simply build the single chunks frame
        //             return [self.single_chunk_to_pil(layer, chunk_idx, palettes)]
        //
        //         bpa_animation_indices = [0, 0, 0, 0]
        //         frames = []
        //
        //         orig_len = len(ldata.tiles)
        //         while True:  # Ended by check at end (do while)
        //             previous_end_of_tiles = orig_len
        //             # For each frame: Insert all BPA current frame tiles into their slots
        //             for bpaidx, bpa in enumerate(self.get_bpas_for_layer(layer, bpas)):
        //
        //                 # Add the BPA tiles for this frame to the set of BPC tiles:
        //                 new_end_of_tiles = previous_end_of_tiles + bpa.number_of_tiles
        //                 ldata.tiles[previous_end_of_tiles:new_end_of_tiles] = bpa.tiles_for_frame(bpa_animation_indices[bpaidx])
        //
        //                 previous_end_of_tiles = new_end_of_tiles
        //                 if bpa.number_of_frames > 0:
        //                     bpa_animation_indices[bpaidx] += 1
        //                     bpa_animation_indices[bpaidx] %= bpa.number_of_frames
        //
        //             frames.append(self.single_chunk_to_pil(layer, chunk_idx, palettes))
        //             # All animations have been played, we are done!
        //             if bpa_animation_indices == [0, 0, 0, 0]:
        //                 break
        //
        //         # RESET the layer's tiles to NOT include the BPA tiles!
        //         ldata.tiles = ldata.tiles[:orig_len]
        //         return frames
        todo!()
    }

    /// Imports tiles that are in a format as described in the documentation for tiles_to_pil.
    /// Tile mappings, chunks and palettes are not updated.
    pub fn pil_to_tiles(&self, layer_id: usize, image: In16ColIndexedImage, py: Python) -> PyResult<()> {
        let image = image.extract(py)?;
        let w = *&image.0.1;
        let h = *&image.0.2;
        let (tiles, _) = TiledImage::native_to_tiled_seq(
            image, BPC_TILE_DIM, w, h
        )?;
        let mut layer = self.layers[layer_id].borrow_mut(py);
        layer.tiles = tiles.into_iter().map(|x| x.0.into()).collect();
        layer.number_tiles = (layer.tiles.len() - 1) as u16;
        Ok(())
    }

    /// Imports chunks. Format same as for chunks_to_pil.
    /// Replaces tiles, tile mappings and therefor also chunks.
    /// "Unsets" BPA assignments! BPAs have to be manually re-assigned by using set_tile or set_chunk. BPA
    /// indices are stored after BPC tile indices.
    ///
    /// The image must have a palette containing the 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range the color is replaced with
    /// 0 of the palette (transparent). The "_force_import" parameter is ignored.
    ///
    /// Returns the palettes stored in the image for further processing (eg. replacing the BPL palettes).
    #[args(force_import = "true")]
    pub fn pil_to_chunks(&self, layer_id: usize, image: In256ColIndexedImage, _force_import: bool, py: Python) -> PyResult<Vec<StBytes>> {
        let image = image.extract(py)?;
        let w = *&image.0.1;
        let h = *&image.0.2;
        debug_assert_eq!(self.tiling_width, self.tiling_height);
        let (tiles, palettes, tilemap) = TiledImage::native_to_tiled(
            image, 16, BPC_TILE_DIM, w, h,
            self.tiling_width as usize, 0, true
        )?;
        let mut layer = self.layers[layer_id].borrow_mut(py);
        layer.tiles = tiles.into_iter().map(|x| x.0.into()).collect();
        layer.tilemap = tilemap.into_iter().map(|x| Py::new(py, x)).collect::<PyResult<Vec<Py<TilemapEntry>>>>()?;
        layer.number_tiles = (layer.tiles.len() - 1) as u16;
        layer.chunk_tilemap_len = layer.tilemap.len() as u16 / self.tiling_width / self.tiling_height;
        Ok(palettes.chunks(16).map(|x| x.into()).collect())
    }

    pub fn get_tile(&self, layer: usize, index: usize, py: Python) -> PyResult<TilemapEntry> {
        self.layers[layer].borrow(py).tilemap[index].extract::<TilemapEntry>(py)
    }

    pub fn set_tile(&mut self, layer: usize, index: usize, tile_mapping: InputTilemapEntry, py: Python) {
        self.layers[layer].borrow_mut(py).tilemap[index] = tile_mapping.0
    }

    pub fn get_chunk(&mut self, layer: usize, index: usize, py: Python) -> PyResult<Vec<TilemapEntry>> {
        let dim = self.tiling_width as usize * self.tiling_height as usize;
        let mtidx = index * dim;
        self.layers[layer].borrow_mut(py).tilemap[mtidx..mtidx+dim]
            .iter().map(|x| x.extract::<TilemapEntry>(py)).collect()
    }

    /// Replace the tiles of the specified layer.
    /// If contains_null_tile is False, the null tile is added to the list, at the beginning.
    #[args(contains_null_tile = "false")]
    pub fn import_tiles(&mut self, layer: usize, mut tiles: Vec<StBytes>, contains_null_tile: bool, py: Python) {
        if !contains_null_tile {
            tiles = once(StBytes::from(vec![0; BPC_TILE_DIM * BPC_TILE_DIM / 2])).chain(tiles).collect();
        }
        let mut layer = self.layers[layer].borrow_mut(py);
        layer.tiles = tiles;
        layer.number_tiles = (layer.tiles.len() - 1) as u16;
    }

    /// Replace the tile mappings of the specified layer.
    /// If contains_null_tile is False, the null chunk is added to the list, at the beginning.
    ///
    /// If correct_tile_ids is True, then the tile id of tile_mappings is also increased by one. Use this,
    /// if you previously used import_tiles with contains_null_tile=false
    #[args(contains_null_chunk = "false", correct_tile_ids = "true")]
    pub fn import_tile_mappings(&mut self, layer: usize, mut tile_mappings: Vec<InputTilemapEntry>, contains_null_chunk: bool, correct_tile_ids: bool, py: Python) -> PyResult<()> {
        let nb_tiles_in_chunk = self.tiling_width * self.tiling_height;
        if correct_tile_ids {
            for entry in tile_mappings.iter_mut() {
                entry.0.borrow_mut(py).0 += 1
            }
        }
        let mut borrow = self.layers[layer].borrow_mut(py);
        borrow.tilemap = if !contains_null_chunk {
            (0..nb_tiles_in_chunk).map(|_| Py::new(py, TilemapEntry::from(0)))
                .chain(tile_mappings.into_iter().map(|x| Ok(x.0))).collect::<PyResult<Vec<Py<TilemapEntry>>>>()?
        } else {
            tile_mappings.into_iter().map(|x| x.0).collect()
        };
        borrow.chunk_tilemap_len = borrow.tilemap.len() as u16 / self.tiling_width / self.tiling_height;
        Ok(())
    }

    /// This method returns a list of not None BPAs assigned to the BPC layer from an ordered list of possible candidates.
    /// What is returned depends on the BPA mapping of the layer.
    ///
    /// The bg_list.dat contains a list of 8 BPAs. The first four are for layer 0, the next four for layer 1.
    ///
    /// This method asserts, that the number of tiles stored in the layer for the BPA, matches the data in the BPA!
    pub fn get_bpas_for_layer(&self, layer: usize, bpas: Vec<Option<InputBpa>>, py: Python) -> PyResult<Vec<InputBpa>> {
        let mut not_none_bpas = Vec::with_capacity(4);
        let borrow = self.layers[layer].borrow(py);
        for (i, bpa) in bpas.into_iter().skip(layer * 4).take(4).enumerate() {
            match bpa {
                None => if borrow.bpas[i] != 0 { return Err(exceptions::PyAssertionError::new_err(format!("BPA {}: {} != 0", i, borrow.bpas[i]))) }
                Some(bpa) => {
                    let bpa_ref = &bpa.0;
                    if borrow.bpas[i] != bpa_ref.get_number_of_tiles(py)? { return Err(exceptions::PyAssertionError::new_err(format!("BPA {}: {} != {}", i, borrow.bpas[i], bpa_ref.get_number_of_tiles(py)?))) };
                    not_none_bpas.push(bpa)
                }
            }
        }
        Ok(not_none_bpas)
    }

    pub fn set_chunk(&mut self, layer: usize, index: usize, new_tilemappings: Vec<InputTilemapEntry>, py: Python) -> PyResult<()> {
        let dim = self.tiling_width as usize * self.tiling_height as usize;
        if new_tilemappings.len() < dim {
            return Err(exceptions::PyValueError::new_err(format!(
                "new tilemapping for this chunk must contain {} tiles.", dim
            )));
        }
        let mtidx = index * dim;
        self.layers[layer].borrow_mut(py).tilemap
            .splice(mtidx..mtidx+9, new_tilemappings
                .into_iter().map(|x| x.0)
            );
        Ok(())
    }

    /// Remove the upper layer. Silently does nothing when it doesn't exist.
    pub fn remove_upper_layer(&mut self, py: Python) -> PyResult<()> {
        if self.number_of_layers == 1 {
            return Ok(());
        }
        self.number_of_layers = 1;
        let mut tmp = Py::new(py, BpcLayer::default())?;
        swap(&mut self.layers[1], &mut tmp);
        self.layers = vec![tmp];
        Ok(())
    }

    /// Add an upper layer. Silently does nothing when it already exists.
    pub fn add_upper_layer(&mut self, py: Python) -> PyResult<()> {
        if self.number_of_layers == 2 {
            return Ok(())
        }
        self.number_of_layers = 2;
        let mut moved = Py::new(py, BpcLayer::default())?;
        swap(&mut moved, &mut self.layers[0]);
        if self.layers.len() < 2 {
            self.layers.push(moved);
        } else {
            self.layers[1] = moved;
        }

        let mut new_layer = self.layers[0].borrow_mut(py);
        // The first tile is not stored, but is always empty
        new_layer.number_tiles = 1;
        new_layer.chunk_tilemap_len = 1;
        new_layer.bpas = [0, 0, 0, 0];
        new_layer.tiles = vec![StBytes::from(vec![0; BPC_BYTELEN_TILE])];
        // The first chunk is not stored, but is always empty
        new_layer.tilemap = (0..(self.tiling_width * self.tiling_height)).map(|_| Py::new(py, TilemapEntry::from(0))).collect::<PyResult<Vec<Py<TilemapEntry>>>>()?;
        Ok(())
    }

    /// Update the layer entries for BPA tile number change and also re-map all tilemappings,
    /// so that they still match their original tile, even though some tiles in-between may now
    /// be new or removed.
    pub fn process_bpa_change(&mut self, bpa_index: usize, tiles_bpa_new: usize, py: Python) -> PyResult<()> {
        let layer_idx = bpa_index / 4;
        let bpa_layer_idx = bpa_index % 4;
        // Re-map all affected tile mappings.
        let mut layer = self.layers[layer_idx].borrow_mut(py);
        let mut tile_idx_start = layer.tiles.len();
        for (bpaidx, n_pas) in layer.bpas.iter().enumerate() {
            if bpaidx >= bpa_layer_idx {
                break;
            }
            tile_idx_start += *n_pas as usize;
        }

        let old_tile_idx_end = tile_idx_start + layer.bpas[bpa_layer_idx] as usize;
        let number_tiles_added = tiles_bpa_new as isize - layer.bpas[bpa_layer_idx] as isize;  // may be negative, of course.
        for mapping in layer.tilemap.iter_mut() {
            let mut mapping = mapping.borrow_mut(py);
            if mapping.0 > old_tile_idx_end {
                // We need to move this back by the full amount
                mapping.0 = (mapping.0 as isize + number_tiles_added) as usize
            } else if mapping.0 >= tile_idx_start {
                // We may need to set to 0, if we removed
                let relative_old_mapping = mapping.0 - tile_idx_start;
                if relative_old_mapping >= tiles_bpa_new {
                    mapping.0 = 0;
                }
            }
        }

        // Finally: Update layer entry.
        layer.bpas[bpa_layer_idx] = tiles_bpa_new as u16;
        Ok(())
    }
}

impl Bpc {
    /// Returns the first found palette of the tile with idx i. Or 0.
    fn get_palette_for_tile(&self, layer: usize, i: usize, py: Python) -> u8 {
        for t in self.layers[layer].borrow(py).tilemap.iter() {
            let t = t.borrow(py);
            if t.0 == i {
                return t.3
            }
        }
        0
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
            .flat_map(|x| {
                debug_assert_eq!(BPC_BYTELEN_TILE, x.len());
                x.0.to_vec()
            })
            .collect();

        BpcImageCompressor::run(data)
    }

    fn convert_tilemap(model: &Bpc, layer: &Py<BpcLayer>, py: Python) -> PyResult<Bytes> {
        let layer = layer.borrow(py);
        let length = (layer.chunk_tilemap_len - 1) * (model.tiling_width * model.tiling_height);
        let mut data = BytesMut::with_capacity(length as usize * BPC_TILEMAP_BYTELEN);
        // Skip first chunk (null)
        for entry in layer.tilemap.iter().skip((model.tiling_width * model.tiling_height) as usize) {
            let entry = entry.extract::<TilemapEntry>(py)?;
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
