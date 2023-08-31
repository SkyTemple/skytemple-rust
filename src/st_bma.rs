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
use crate::compression::bma_collision_rle::{
    BmaCollisionRleCompressor, BmaCollisionRleDecompressor,
};
use crate::compression::bma_layer_nrl::{BmaLayerNrlCompressor, BmaLayerNrlDecompressor};
use crate::compression::generic::nrl::{compression_step, decompression_step, NrlCompRead};
use crate::gettext::gettext;
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::{InputTilemapEntry, TilemapEntry};
use crate::image::{In256ColIndexedImage, InIndexedImage, IndexedImage, Palette, Raster};
use crate::python::*;
use crate::st_bpa::input::InputBpa;
#[cfg(not(feature = "python"))]
use crate::st_bpc::input::BpcProvider;
use crate::st_bpc::input::InputBpc;
use crate::st_bpc::BPC_TILE_DIM;
#[cfg(not(feature = "python"))]
use crate::st_bpl::input::BplProvider;
use crate::st_bpl::input::InputBpl;
use crate::st_bpl::{BPL_IMG_PAL_LEN, BPL_MAX_PAL, BPL_PAL_LEN};
use crate::util::lcm;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use itertools::Itertools;
use std::io::Cursor;
use std::iter::{Copied, Enumerate};
use std::slice::Iter;

#[pyclass(module = "skytemple_rust.st_bma")]
#[derive(Clone)]
pub struct Bma {
    #[pyo3(get, set)]
    pub map_width_camera: u8,
    #[pyo3(get, set)]
    pub map_height_camera: u8,
    #[pyo3(get, set)]
    pub tiling_width: u8,
    #[pyo3(get, set)]
    pub tiling_height: u8,
    #[pyo3(get, set)]
    pub map_width_chunks: u8,
    #[pyo3(get, set)]
    pub map_height_chunks: u8,
    #[pyo3(get, set)]
    pub number_of_layers: u16,
    #[pyo3(get, set)]
    pub unk6: u16,
    #[pyo3(get, set)]
    pub number_of_collision_layers: u16,

    #[pyo3(get, set)]
    pub layer0: Vec<u16>,
    #[pyo3(get, set)]
    pub layer1: Option<Vec<u16>>,

    // if unk6:
    #[pyo3(get, set)]
    pub unknown_data_block: Option<Vec<u8>>,
    // if number_of_collision_layers > 0:
    #[pyo3(get, set)]
    pub collision: Option<Vec<bool>>,
    // if number_of_collision_layers > 1:
    #[pyo3(get, set)]
    pub collision2: Option<Vec<bool>>,
}

#[pymethods]
impl Bma {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        let mut data = Cursor::new(data);
        let map_width_camera = data.get_u8();
        let map_height_camera = data.get_u8();
        // ALL game maps have the same values here. Changing them does nothing,
        // so the game seems to be hardcoded to 3x3.
        let tiling_width = data.get_u8();
        let tiling_height = data.get_u8();
        // Map width & height in chunks, so map.map_width_camera / map.tiling_width
        // The only maps this is not true for are G01P08A. S01P01B, S15P05A, S15P05B, it seems they
        // are missing one tile in width (32x instead of 33x)
        // The game doesn't seem to care if this value is off by less than 3 (tiling_w/h).
        // But NOTE that this has consequences for the collision and unknown data layers! See notes at collision
        // below!
        let map_width_chunks = data.get_u8();
        let map_height_chunks = data.get_u8();

        let number_of_layers = data.get_u16_le();
        // Effectively a boolean, whether or not the data layer exists
        let unk6 = data.get_u16_le();
        let number_of_collision_layers = data.get_u16_le();

        let mut number_of_bytes_per_layer =
            map_width_chunks as usize * map_height_chunks as usize * 2;
        //  If the map width is odd, we have one extra tile per row:
        if map_width_chunks % 2 != 0 {
            number_of_bytes_per_layer += map_height_chunks as usize * 2;
        }
        // Read first layer
        //println!("Check 1: {} .. {}", data.position(), data.remaining());
        let layer0 = Self::read_layer(
            map_width_chunks as usize,
            map_height_chunks as usize,
            BmaLayerNrlDecompressor::run(&mut data, number_of_bytes_per_layer)?,
        );
        //println!("Check 2: {} .. {}", data.position(), data.remaining());
        let layer1 = if number_of_layers > 1 {
            // Read second layer
            Some(Self::read_layer(
                map_width_chunks as usize,
                map_height_chunks as usize,
                BmaLayerNrlDecompressor::run(&mut data, number_of_bytes_per_layer)?,
            ))
        } else {
            None
        };

        //println!("Check 3: {} .. {}", data.position(), data.remaining());
        let unknown_data_block = if unk6 > 0 {
            let stop_when_size = map_width_camera as usize * map_height_camera as usize;
            let mut decompressed_data = Vec::with_capacity(stop_when_size);

            while decompressed_data.len() < stop_when_size {
                if !NrlCompRead::nrl_has_remaining(&data) {
                    return Err(exceptions::PyValueError::new_err(format!(
                        "BMA Collision Decompressor: End result length unexpected. Should be {}, is {}.",
                        stop_when_size, decompressed_data.len()
                    )));
                }
                decompression_step(&mut data, &mut decompressed_data);
            }
            Some(decompressed_data)
        } else {
            None
        };

        // Read level collision
        //println!("Check 4: {} .. {}", data.position(), data.remaining());
        let collision = if number_of_collision_layers > 0 {
            Some(Self::read_collision(
                map_width_camera as usize,
                BmaCollisionRleDecompressor::run(
                    &mut data,
                    map_width_camera as usize * map_height_camera as usize,
                )?,
            ))
        } else {
            None
        };

        //println!("Check 5: {} .. {}", data.position(), data.remaining());
        let collision2 = if number_of_collision_layers > 1 {
            Some(Self::read_collision(
                map_width_camera as usize,
                BmaCollisionRleDecompressor::run(
                    &mut data,
                    map_width_camera as usize * map_height_camera as usize,
                )?,
            ))
        } else {
            None
        };

        //println!("Done! : {} .. {}", data.position(), data.remaining());
        Ok(Self {
            map_width_camera,
            map_height_camera,
            tiling_width,
            tiling_height,
            map_width_chunks,
            map_height_chunks,
            number_of_layers,
            unk6,
            number_of_collision_layers,
            layer0,
            layer1,
            unknown_data_block,
            collision,
            collision2,
        })
    }

    /// Converts one layer of the map into an image. The exported image has the same format as expected by from_pil.
    /// Exported is a single frame.
    ///
    /// The list of bpas must be the one contained in the bg_list. It needs to contain 8 slots, with empty
    /// slots being None.
    ///
    /// 0: lower layer
    /// 1: upper layer
    ///
    /// (Python) example, of how to export and then import again using images:
    /// ```py
    /// l_upper = bma.to_pil_single_layer(bpc, bpl.palettes, bpas, 1)
    /// l_lower = bma.to_pil_single_layer(bpc, bpl.palettes, bpas, 0)
    /// bma.from_pil(bpc, bpl, l_lower, l_upper)
    /// ```
    pub fn to_pil_single_layer(
        &self,
        mut bpc: InputBpc,
        palettes: Vec<StBytes>,
        bpas: Vec<Option<InputBpa>>,
        layer: usize,
        py: Python,
    ) -> PyResult<IndexedImage> {
        let chunk_width = BPC_TILE_DIM * self.tiling_width as usize;
        let chunk_height = BPC_TILE_DIM * self.tiling_height as usize;

        let width_map = self.map_width_chunks as usize * chunk_width;
        let height_map = self.map_height_chunks as usize * chunk_height;

        let bma_layer;
        let bpc_layer_id;
        if layer == 0 {
            bma_layer = &self.layer0;
            bpc_layer_id = if bpc.0.get_number_of_layers(py)? == 1 {
                0
            } else {
                1
            };
        } else {
            bma_layer = self.layer1.as_ref().unwrap();
            bpc_layer_id = 0;
        }

        let chunks = &bpc
            .0
            .get_chunks_animated_to_pil(bpc_layer_id, &palettes, &bpas, 1, py)?[0];

        let mut fimg = IndexedImage(Raster::new(width_map, height_map), chunks.1.clone());

        for (i, mt_idx) in bma_layer.iter().enumerate() {
            let x = i % self.map_width_chunks as usize;
            let y = i / self.map_width_chunks as usize;
            fimg.0.paste(
                chunks
                    .0
                    .crop(0, *mt_idx as usize * chunk_width, chunk_width, chunk_height),
                x * chunk_width,
                y * chunk_height,
            );
        }
        Ok(fimg)
    }

    /// Converts the entire map into an image, as shown in the game. Each PIL image in the list returned is one
    /// frame. The palettes argument can be retrieved from the map's BPL (bpl.palettes).
    ///
    /// This implementation does NOT support drawing the unknown data block or collision.
    /// The parameters will be ignored. Use the Python implementation if you need this debugging information.
    ///
    /// The method does not care about frame speeds. Each step of animation is simply returned as a new image,
    /// so if BPAs use different frame speeds, this is ignored; they effectively run at the same speed.
    /// If BPAs are using a different amount of frames per tile, the length of returned list of images will be the lowest
    /// common multiple of the different frame lengths.
    ///
    /// If pal_ani=true, then also includes palette animations.
    ///
    /// The list of bpas must be the one contained in the bg_list. It needs to contain 8 slots, with empty
    /// slots being None.
    #[allow(clippy::too_many_arguments)]
    #[allow(unused_variables)]
    #[pyo3(signature = (
        bpc,
        bpl,
        bpas,
        include_collision = true,
        include_unknown_data_block = true,
        pal_ani = true,
        single_frame = false
    ))]
    pub fn to_pil(
        &self,
        mut bpc: InputBpc,
        bpl: InputBpl,
        bpas: Vec<Option<InputBpa>>,
        include_collision: bool,
        include_unknown_data_block: bool,
        pal_ani: bool,
        single_frame: bool,
        py: Python,
    ) -> PyResult<Vec<IndexedImage>> {
        let chunk_width = BPC_TILE_DIM * self.tiling_width as usize;
        let chunk_height = BPC_TILE_DIM * self.tiling_height as usize;

        let width_map = self.map_width_chunks as usize * chunk_width;
        let height_map = self.map_height_chunks as usize * chunk_height;

        let palettes = bpl.0.get_palettes(py)?;

        let mut final_images = Vec::with_capacity(50);
        let lower_layer_bpc = if bpc.0.get_number_of_layers(py)? == 1 {
            0
        } else {
            1
        };
        let chunks_lower =
            bpc.0
                .get_chunks_animated_to_pil(lower_layer_bpc, &palettes, &bpas, 1, py)?;
        let len_lower = chunks_lower.len();
        for img in chunks_lower {
            let mut fimg = IndexedImage(Raster::new(width_map, height_map), img.1.clone());

            // yes. self.layer0 is always the LOWER layer! It's the opposite from BPC
            for (i, mt_idx) in self.layer0.iter().enumerate() {
                let x = i % self.map_width_chunks as usize;
                let y = i / self.map_width_chunks as usize;
                fimg.0.paste(
                    img.0
                        .crop(0, *mt_idx as usize * chunk_width, chunk_width, chunk_height),
                    x * chunk_width,
                    y * chunk_height,
                );
            }

            final_images.push(fimg);
            if single_frame {
                break;
            }
        }
        if bpc.0.get_number_of_layers(py)? > 1 {
            // Overlay higher layer tiles
            let mut chunks_higher =
                bpc.0
                    .get_chunks_animated_to_pil(0, &bpl.0.get_palettes(py)?, &bpas, 1, py)?;
            let len_higher = chunks_higher.len();
            if len_higher != len_lower && !single_frame {
                // oh fun! We are missing animations for one of the layers, let's stretch to the lowest common multiple
                let lm = lcm(len_higher, len_lower);
                for i in len_lower..lm {
                    final_images.push(final_images[i % len_lower].clone())
                }
                for i in len_higher..lm {
                    chunks_higher.push(chunks_higher[i % len_higher].clone())
                }
            }

            for (j, img) in chunks_higher.iter().enumerate() {
                let fimg = &mut final_images[j];
                debug_assert!(self.layer1.is_some());
                for (i, mt_idx) in self.layer1.as_ref().unwrap().iter().enumerate() {
                    let x = i % self.map_width_chunks as usize;
                    let y = i / self.map_width_chunks as usize;

                    fimg.0.paste_masked(
                        img.0
                            .crop(0, *mt_idx as usize * chunk_width, chunk_width, chunk_height),
                        x * chunk_width,
                        y * chunk_height,
                        true,
                    );
                }
                if single_frame {
                    break;
                }
            }
        }

        // Apply palette animations
        if pal_ani
            && !single_frame
            && bpl.0.get_has_palette_animation(py)?
            && !bpl.0.get_animation_palette(py)?.is_empty()
        {
            let old_images = final_images;
            let mut old_images_i = 0;

            final_images = Vec::with_capacity(old_images.len());

            for ppal_ani in 0..bpl.0.get_animation_palette(py)?.len() {
                let mut current_img = old_images[old_images_i].clone();
                // Switch out the palette with that from the palette animation
                let pal_for_frame = bpl
                    .0
                    .do_apply_palette_animations(ppal_ani as u16, py)?
                    .into_iter()
                    .flatten()
                    .collect();
                current_img.1 = pal_for_frame;
                final_images.push(current_img);
                old_images_i += 1;
                if old_images_i >= old_images.len() {
                    old_images_i = 0;
                }
            }
        }
        Ok(final_images)
    }

    /// Import an entire map from one or two images (for each layer).
    /// Changes all tiles, tilemappings and chunks in the BPC and re-writes the two layer mappings of the BMA.
    /// Imports the palettes of the image to the BPL.
    /// The palettes of the images passed into this method must either identical or can be merged.
    /// The how_many_palettes_lower_layer parameter controls how many palettes
    /// from the lower layer image will then be used.
    ///
    /// The passed PIL will be split into separate tiles and the tile's palette index in the tile mapping for this
    /// coordinate is determined by the first pixel value of each tile in the PIL. The PIL
    /// must have a palette containing up to 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range, the color is replaced with
    /// 0 of the palette (transparent). The force_import flag is ignored.
    ///
    /// Does not import animations. BPA tiles must be manually mapped to the tilemappings of the BPC after the import.
    /// BPL palette animations are not modified.
    ///
    /// The input images must have the same dimensions as the BMA (same dimensions as to_pil_single_layer would export).
    /// The input image can have a different number of layers, than the BMA. BPC and BMA layers are changed accordingly.
    ///
    /// BMA collision and data layer are not modified.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::option_map_unit_fn)]
    #[allow(unused_variables)]
    #[pyo3(signature = (
        bpc,
        bpl,
        lower_img = None,
        upper_img = None,
        force_import = true,
        how_many_palettes_lower_layer = 16
    ))]
    pub fn from_pil(
        &mut self,
        mut bpc: InputBpc,
        mut bpl: InputBpl,
        lower_img: Option<In256ColIndexedImage>,
        upper_img: Option<In256ColIndexedImage>,
        force_import: bool,
        how_many_palettes_lower_layer: usize,
        py: Python,
    ) -> PyResult<()> {
        let expected_width =
            self.tiling_width as usize * self.map_width_chunks as usize * BPC_TILE_DIM;
        let expected_height =
            self.tiling_height as usize * self.map_height_chunks as usize * BPC_TILE_DIM;
        let mut lower_img: Option<IndexedImage> = match lower_img {
            None => None,
            Some(img) => Some(img.extract(py)?),
        };
        let mut upper_img: Option<IndexedImage> = match upper_img {
            None => None,
            Some(img) => Some(img.extract(py)?),
        };
        if lower_img
            .as_ref()
            .map_or(false, |lower_img| lower_img.0 .1 != expected_width)
            || upper_img
                .as_ref()
                .map_or(false, |upper_img| upper_img.0 .1 != expected_width)
        {
            return Err(exceptions::PyValueError::new_err(gettext!(
                "Can not import map background: Width of both images must match the current map width: {}px",
                expected_width
            )));
        }
        if lower_img
            .as_ref()
            .map_or(false, |lower_img| lower_img.0 .2 != expected_height)
            || upper_img
                .as_ref()
                .map_or(false, |upper_img| upper_img.0 .2 != expected_height)
        {
            return Err(exceptions::PyValueError::new_err(gettext!(
                "Can not import map background: Height of both images must match the current map width: {}px",
                expected_height
            )));
        }
        let mut upper_palette_palette_color_offset = 0;
        if upper_img.is_some()
            && lower_img.is_some()
            && how_many_palettes_lower_layer < BPL_MAX_PAL as usize
        {
            // Combine palettes
            let lower_palette = &lower_img.as_ref().unwrap().1
                [..(how_many_palettes_lower_layer * (BPL_PAL_LEN + 1) * 3)];
            let upper_palette = &upper_img.as_ref().unwrap().1[..((BPL_MAX_PAL as usize
                - how_many_palettes_lower_layer)
                * (BPL_PAL_LEN + 1)
                * 3)];
            let new_palette: Palette = lower_palette
                .iter()
                .chain(upper_palette.iter())
                .copied()
                .collect();
            lower_img.as_mut().map(|x| x.1 = new_palette.clone());
            upper_img.as_mut().map(|x| x.1 = new_palette);
            // We need to offset the colors in the upper image now, when we read it.
            upper_palette_palette_color_offset = how_many_palettes_lower_layer
        }
        // Adjust layer numbers
        let number_of_layers = if upper_img.is_some() { 2 } else { 1 };
        let low_map_idx = if lower_img.is_some() { 0 } else { 1 };
        if number_of_layers > self.number_of_layers {
            self.add_upper_layer();
            bpc.0.do_add_upper_layer(py)?;
        }

        // Import tiles, tile mappings and chunks mappings
        let mut palettes: Vec<Vec<u8>> = Default::default();
        if let Some(lower_img) = lower_img {
            palettes = self.from_pil_step(
                false,
                if bpc.0.get_number_of_layers(py)? == 1 {
                    0
                } else {
                    1
                },
                lower_img,
                0,
                &mut bpc,
                py,
            )?;
        }
        if let Some(upper_img) = upper_img {
            palettes = self.from_pil_step(
                true,
                0,
                upper_img,
                upper_palette_palette_color_offset,
                &mut bpc,
                py,
            )?;
        }

        // Import palettes
        bpl.0.do_import_palettes(palettes, py)?;
        Ok(())
    }

    /// Remove the upper layer. Silently does nothing when it doesn't exist.
    pub fn remove_upper_layer(&mut self) {
        if self.number_of_layers > 1 {
            self.number_of_layers = 1;
            self.layer1 = None
        }
    }

    /// Add an upper layer. Silently does nothing when it already exists.
    pub fn add_upper_layer(&mut self) {
        if self.number_of_layers < 2 {
            self.number_of_layers = 2;
            self.layer1 = Some(vec![
                0;
                self.map_width_chunks as usize
                    * self.map_height_chunks as usize
            ]);
        }
    }

    /// Change the dimensions of the map. Existing tiles and chunks will keep their position in the grid.
    /// If the size is reduced, all tiles and chunks that are moved out of the new dimension box are removed.
    pub fn resize(
        &mut self,
        new_width_chunks: u8,
        new_height_chunks: u8,
        new_width_camera: u8,
        new_height_camera: u8,
    ) {
        // Layer 0
        self.layer0 =
            self.layer0
                .layer_resize(self.map_width_chunks, new_width_chunks, new_height_chunks);
        // Layer 1
        self.layer1 =
            self.layer1
                .layer_resize(self.map_width_chunks, new_width_chunks, new_height_chunks);
        // Collision
        self.collision =
            self.collision
                .layer_resize(self.map_width_camera, new_width_camera, new_height_camera);
        // Collision 2
        self.collision2 = self.collision2.layer_resize(
            self.map_width_camera,
            new_width_camera,
            new_height_camera,
        );
        // Data Layer
        self.unknown_data_block = self.unknown_data_block.layer_resize(
            self.map_width_camera,
            new_width_camera,
            new_height_camera,
        );
        //
        self.map_width_chunks = new_width_chunks;
        self.map_height_chunks = new_height_chunks;
        self.map_width_camera = new_width_camera;
        self.map_height_camera = new_height_camera;
    }

    /// Place the chunk with the given ID at the X and Y position. No error checking is done.
    pub fn place_chunk(&mut self, layer_id: u8, x: usize, y: usize, chunk_index: u16) {
        let bma_index = y * self.map_width_chunks as usize + x;
        if layer_id == 0 {
            self.layer0[bma_index] = chunk_index;
        } else {
            match &mut self.layer1 {
                None => panic!("No layer 1 exists"),
                Some(layer1) => layer1[bma_index] = chunk_index,
            };
        }
    }

    /// Set the collision at the X and Y position. No error checking is done.
    pub fn place_collision(&mut self, collision_layer_id: u8, x: usize, y: usize, is_solid: bool) {
        let bma_index = y * self.map_width_camera as usize + x;
        if collision_layer_id == 0 {
            match &mut self.collision {
                None => panic!("No collision layer exists"),
                Some(collision) => collision[bma_index] = is_solid,
            };
        } else {
            match &mut self.collision2 {
                None => panic!("No second collision layer exists"),
                Some(collision2) => collision2[bma_index] = is_solid,
            };
        }
    }

    /// Set data at the X and Y position. No error checking is done.
    pub fn place_data(&mut self, x: usize, y: usize, data: u8) {
        let bma_index = y * self.map_width_camera as usize + x;
        match &mut self.unknown_data_block {
            None => panic!("No unknown data layer exists"),
            Some(unknown_data_block) => unknown_data_block[bma_index] = data,
        };
    }

    pub fn deepcopy(&self) -> Self {
        // Cloning isn't enough (pyo3 weirdness).
        self.clone()
    }
}

impl Bma {
    fn read_layer(map_width_chunks: usize, map_height_chunks: usize, mut data: Bytes) -> Vec<u16> {
        // To get the actual index of a chunk, the value is XORed with the tile value right above!
        let mut previous_row_values = vec![0; map_width_chunks];
        let mut layer = Vec::with_capacity(data.len());
        let max_tiles = map_width_chunks * map_height_chunks;
        let mut i = 0;
        let mut skipped_on_prev = true;
        while data.has_remaining() {
            let chunk_i = data.get_u16_le();
            if i >= max_tiles {
                // this happens if there is a leftover 12bit word.
                break;
            }
            let index_in_row = i % map_width_chunks;
            // If the map width is odd, there is one extra chunk at the end of every row,
            // we remove this chunk.
            if !skipped_on_prev && index_in_row == 0 && map_width_chunks % 2 != 0 {
                skipped_on_prev = true;
                continue;
            }
            skipped_on_prev = false;
            let cv = chunk_i ^ previous_row_values[index_in_row];
            previous_row_values[index_in_row] = cv;
            layer.push(cv);
            i += 1;
        }
        layer
    }

    fn read_collision(map_width_camera: usize, data: Bytes) -> Vec<bool> {
        // To get the actual index of a chunk, the value is XORed with the tile value right above!
        let mut previous_row_values = vec![false; map_width_camera];
        let mut col = Vec::with_capacity(data.len());
        for (i, chunk) in data.into_iter().enumerate() {
            let index_in_row = i % map_width_camera;
            let cv = (chunk ^ previous_row_values[index_in_row] as u8) != 0;
            previous_row_values[index_in_row] = cv;
            col.push(cv);
        }
        col
    }

    #[allow(clippy::option_map_unit_fn)]
    #[allow(clippy::wrong_self_convention)]
    fn from_pil_step(
        &mut self,
        is_upper: bool,
        bpc_layer_id: usize,
        img: IndexedImage,
        palette_offset: usize,
        bpc: &mut InputBpc,
        py: Python,
    ) -> PyResult<Vec<Vec<u8>>> {
        let layer = match is_upper {
            true => self.layer1.as_mut(),
            false => Some(&mut self.layer0),
        };
        let w = img.0 .1;
        let h = img.0 .2;
        let (tiles, palettes, mut all_possible_tile_mappings) = TiledImage::native_to_tiled(
            img,
            BPL_IMG_PAL_LEN as u8,
            BPC_TILE_DIM,
            w,
            h,
            self.tiling_width as usize,
            palette_offset,
            true,
        )?;
        bpc.0.do_import_tiles(
            bpc_layer_id,
            tiles.into_iter().map(|x| x.freeze()).collect(),
            false,
            py,
        )?;

        // Build a new list of chunks / tile mappings for the BPC based on repeating chunks
        let tiles_in_chunk = self.tiling_width as usize * self.tiling_height as usize;
        let n_all_chunks =
            self.map_width_chunks as usize * self.map_height_chunks as usize * tiles_in_chunk;
        let mut chunk_mappings = Vec::with_capacity(n_all_chunks);
        let mut chunk_mappings_counter = 1;
        let mut tile_mappings = Vec::with_capacity(n_all_chunks);
        all_possible_tile_mappings.truncate(n_all_chunks);
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

        bpc.0.do_import_tile_mappings(
            bpc_layer_id,
            tile_mappings
                .into_iter()
                .map(|t| Ok(InputTilemapEntry(Py::new(py, t)?)))
                .collect::<PyResult<Vec<InputTilemapEntry>>>()?,
            false,
            true,
            py,
        )?;
        layer.map(|x| *x = chunk_mappings);
        Ok(palettes
            .0
            .chunks(BPL_IMG_PAL_LEN * 3)
            .map(|x| x.to_vec())
            .collect::<Vec<Vec<u8>>>())
    }
}

#[pyclass(module = "skytemple_rust.st_bma")]
#[derive(Clone, Default)]
pub struct BmaWriter;

#[pymethods]
impl BmaWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<Bma>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        let mut data = BytesMut::with_capacity(4086);

        data.put_u8(model.map_width_camera);
        data.put_u8(model.map_height_camera);
        data.put_u8(model.tiling_width);
        data.put_u8(model.tiling_height);
        data.put_u8(model.map_width_chunks);
        data.put_u8(model.map_height_chunks);
        debug_assert!(model.number_of_layers < 3);
        data.put_u16_le(model.number_of_layers);
        debug_assert!(model.unk6 < 3);
        data.put_u16_le(model.unk6);
        debug_assert!(model.number_of_collision_layers < 3);
        data.put_u16_le(model.number_of_collision_layers);

        data.extend(Self::convert_layer(
            model.map_width_chunks as usize,
            model.map_height_chunks as usize,
            &model.layer0,
        )?);
        match &model.layer1 {
            Some(layer1) => data.extend(Self::convert_layer(
                model.map_width_chunks as usize,
                model.map_height_chunks as usize,
                layer1,
            )?),
            None => {}
        }

        match &model.unknown_data_block {
            Some(unknown_data_block) => data.extend(Self::convert_unknown_data_layer(
                model.map_width_camera as usize,
                model.map_height_camera as usize,
                unknown_data_block,
            )?),
            None => {}
        }

        match &model.collision {
            Some(collision) => data.extend(Self::convert_collision(
                model.map_width_camera as usize,
                model.map_height_camera as usize,
                collision,
            )?),
            None => {}
        }
        match &model.collision2 {
            Some(collision2) => data.extend(Self::convert_collision(
                model.map_width_camera as usize,
                model.map_height_camera as usize,
                collision2,
            )?),
            None => {}
        }

        Ok(data.into())
    }
}

impl BmaWriter {
    /// Converts chunk mappings for a layer back into bytes.
    /// If map size is odd, adds one extra tiles per row.
    /// Every row is NRL encoded separately, because the game decodes the rows separately!
    #[allow(clippy::needless_range_loop)]
    fn convert_layer(
        map_width_chunks: usize,
        map_height_chunks: usize,
        layer: &[u16],
    ) -> PyResult<BytesMut> {
        // The actual values are "encoded" using XOR.
        let mut previous_row_values = vec![0; map_width_chunks];
        let mut size = map_width_chunks * map_height_chunks * 2;
        debug_assert_eq!(size, layer.len() * 2);
        if map_width_chunks % 2 != 0 {
            // Keep in mind there's an extra null tile to be added per row
            size += map_height_chunks * 2;
        }

        let mut layer_bytes = BytesMut::with_capacity(size);

        // Each tile is separately encoded, so we also build them separately
        for row in 0..map_height_chunks {
            let mut row_bytes = BytesMut::with_capacity(size / map_height_chunks);
            for col in 0..map_width_chunks {
                let i = row * map_width_chunks + col;
                row_bytes.put_u16_le(layer[i] ^ previous_row_values[col]);
                previous_row_values[col] = layer[i];
            }
            if map_width_chunks % 2 != 0 {
                row_bytes.put_u16_le(0);
            }
            debug_assert_eq!(row_bytes.len(), size / map_height_chunks);
            layer_bytes.extend(BmaLayerNrlCompressor::run(row_bytes.freeze())?)
        }
        Ok(layer_bytes)
    }

    /// Converts collision mappings back into bytes.
    /// If map size is odd, adds one extra tiles per row
    /// Every row is NRL encoded separately, because the game decodes the rows separately!
    #[allow(clippy::needless_range_loop)]
    fn convert_collision(
        map_width_camera: usize,
        map_height_camera: usize,
        collision_layer: &[bool],
    ) -> PyResult<BytesMut> {
        // The actual values are "encoded" using XOR.
        let mut previous_row_values = vec![false; map_width_camera];
        let size = map_width_camera * map_height_camera;
        debug_assert_eq!(size, collision_layer.len());

        let mut layer_bytes = BytesMut::with_capacity(size);

        // Each tile is separately encoded, so we also build them separately
        for row in 0..map_height_camera {
            let mut row_bytes = BytesMut::with_capacity(size / map_height_camera);
            for col in 0..map_width_camera {
                let i = row * map_width_camera + col;
                row_bytes.put_u8(collision_layer[i] as u8 ^ previous_row_values[col] as u8);
                previous_row_values[col] = collision_layer[i];
            }
            debug_assert_eq!(row_bytes.len(), size / map_height_camera);
            layer_bytes.extend(BmaCollisionRleCompressor::run(row_bytes.freeze())?)
        }
        Ok(layer_bytes)
    }

    /// Converts the unknown data layer back into bytes
    /// Every row is NRL encoded separately, because the game decodes the rows separately!
    fn convert_unknown_data_layer(
        map_width_camera: usize,
        map_height_camera: usize,
        unknown_data_block: &[u8],
    ) -> PyResult<BytesMut> {
        let size = map_width_camera * map_height_camera;
        debug_assert_eq!(size, unknown_data_block.len());

        let mut layer_bytes = BytesMut::with_capacity(size);

        // Each tile is separately encoded, so we also build them separately
        for row in 0..map_height_camera {
            let mut row_bytes = BytesMut::with_capacity(size / map_height_camera);
            for col in 0..map_width_camera {
                let i = row * map_width_camera + col;
                row_bytes.put_u8(unknown_data_block[i]);
            }
            debug_assert_eq!(row_bytes.len(), size / map_height_camera);
            let mut compressed_data = BytesMut::with_capacity(row_bytes.len() * 2);
            let mut cursor = Cursor::new(row_bytes);
            while NrlCompRead::nrl_has_remaining(&cursor) {
                compression_step(&mut cursor, &mut compressed_data);
            }
            layer_bytes.extend(compressed_data);
        }
        Ok(layer_bytes)
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

trait Resizable<T>: Sized
where
    T: Default + Copy,
{
    fn layer_resize(&self, old_w: u8, new_w: u8, new_h: u8) -> Self {
        // Convert existing data into a grid
        let mut rows: Vec<Vec<T>> = Vec::with_capacity((self.len() / old_w as usize) + 1);
        let mut current_row: Vec<T> = Vec::with_capacity(old_w as usize);
        for (i, el) in self.enumerate() {
            if i > 0 && i % old_w as usize == 0 {
                rows.push(current_row);
                current_row = Vec::with_capacity(old_w as usize);
            }
            current_row.push(el);
        }
        rows.push(current_row);

        // Shrink / enlarge the grid
        // Y: Enlarge
        for _ in 0..(new_h as i64 - rows.len() as i64) {
            rows.push(Vec::with_capacity(old_w as usize));
        }
        // Y: Shrink
        if (new_h as usize) < rows.len() {
            rows.truncate(new_h as usize);
        }
        for row in rows.iter_mut() {
            // X: Enlarge
            for _ in 0..(new_w as i64 - row.len() as i64) {
                row.push(T::default());
            }
            // X: Shrink
            if (new_w as usize) < row.len() {
                row.truncate(new_w as usize);
            }
        }
        Self::collect(rows.into_iter().flatten())
    }
    fn len(&self) -> usize;
    fn enumerate(&self) -> Enumerate<Copied<Iter<T>>>;
    fn collect<U>(iter: U) -> Self
    where
        U: Iterator<Item = T>;
}

impl<T> Resizable<T> for Vec<T>
where
    T: Default + Copy,
{
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn enumerate(&self) -> Enumerate<Copied<Iter<T>>> {
        self.iter().copied().enumerate()
    }

    fn collect<U>(iter: U) -> Self
    where
        U: Iterator<Item = T>,
    {
        iter.collect()
    }
}

impl<T, U> Resizable<U> for Option<T>
where
    T: Resizable<U>,
    U: Default + Copy,
{
    fn layer_resize(&self, old_w: u8, new_w: u8, new_h: u8) -> Self {
        self.as_ref().map(|v| v.layer_resize(old_w, new_w, new_h))
    }

    fn len(&self) -> usize {
        self.as_ref().unwrap().len()
    }

    fn enumerate(&self) -> Enumerate<Copied<Iter<U>>> {
        self.as_ref().unwrap().enumerate()
    }

    fn collect<V>(iter: V) -> Self
    where
        V: Iterator<Item = U>,
    {
        Some(T::collect(iter))
    }
}
