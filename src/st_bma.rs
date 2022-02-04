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
use std::iter::{Copied, Enumerate};
use std::slice::Iter;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::bytes::{StBytes};
use crate::compression::bma_collision_rle::{BmaCollisionRleCompressor, BmaCollisionRleDecompressor};
use crate::compression::bma_layer_nrl::{BmaLayerNrlCompressor, BmaLayerNrlDecompressor};
use crate::compression::generic::nrl::{compression_step, decompression_step, NrlCompRead};
use crate::image::{In256ColIndexedImage, IndexedImage};
use crate::python::*;
use crate::st_bpa::input::InputBpa;
use crate::st_bpc::input::InputBpc;
use crate::st_bpl::input::InputBpl;

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
    pub collision2: Option<Vec<bool>>
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

        let mut number_of_bytes_per_layer = map_width_chunks as usize * map_height_chunks as usize * 2;
        //  If the map width is odd, we have one extra tile per row:
        if map_width_chunks % 2 != 0 {
            number_of_bytes_per_layer += map_height_chunks as usize * 2;
        }
        // Read first layer
        //println!("Check 1: {} .. {}", data.position(), data.remaining());
        let layer0 = Self::read_layer(
            map_width_chunks as usize, map_height_chunks as usize,
            BmaLayerNrlDecompressor::run(
                &mut data, number_of_bytes_per_layer
        )?);
        //println!("Check 2: {} .. {}", data.position(), data.remaining());
        let layer1 = if number_of_layers > 1 {
            // Read second layer
            Some(Self::read_layer(
                map_width_chunks as usize, map_height_chunks as usize,
                BmaLayerNrlDecompressor::run(
                    &mut data, number_of_bytes_per_layer
            )?))
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
                    )))
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
            Some(Self::read_collision(map_width_camera as usize, BmaCollisionRleDecompressor::run(
                &mut data, map_width_camera as usize * map_height_camera as usize
            )?))
        } else {
            None
        };

        //println!("Check 5: {} .. {}", data.position(), data.remaining());
        let collision2 = if number_of_collision_layers > 1 {
            Some(Self::read_collision(map_width_camera as usize, BmaCollisionRleDecompressor::run(
                &mut data, map_width_camera as usize * map_height_camera as usize
            )?))
        } else {
            None
        };

        //println!("Done! : {} .. {}", data.position(), data.remaining());
        Ok(Self {
            map_width_camera, map_height_camera, tiling_width, tiling_height, map_width_chunks,
            map_height_chunks, number_of_layers, unk6, number_of_collision_layers, layer0, layer1,
            unknown_data_block, collision, collision2
        })
    }

    pub fn to_pil_single_layer(&self, bpc: InputBpc, palettes: Vec<Vec<u8>>, bpas: Vec<Option<InputBpa>>, layer: u8) -> IndexedImage {
        //         """
        //         Converts one layer of the map into an image. The exported image has the same format as expected by from_pil.
        //         Exported is a single frame.
        //
        //         The list of bpas must be the one contained in the bg_list. It needs to contain 8 slots, with empty
        //         slots being None.
        //
        //         0: lower layer
        //         1: upper layer
        //
        //         Example, of how to export and then import again using images:
        //             >>> l_upper = bma.to_pil_single_layer(bpc, bpl.palettes, bpas, 1)
        //             >>> l_lower = bma.to_pil_single_layer(bpc, bpl.palettes, bpas, 0)
        //             >>> bma.from_pil(bpc, bpl, l_lower, l_upper)
        //         """
        //         chunk_width = BPC_TILE_DIM * self.tiling_width
        //         chunk_height = BPC_TILE_DIM * self.tiling_height
        //
        //         width_map = self.map_width_chunks * chunk_width
        //         height_map = self.map_height_chunks * chunk_height
        //
        //         if layer == 0:
        //             bma_layer = self.layer0
        //             bpc_layer_id = 0 if bpc.number_of_layers == 1 else 1
        //         else:
        //             assert self.layer1 is not None
        //             bma_layer = self.layer1
        //             bpc_layer_id = 0
        //
        //         chunks = bpc.chunks_animated_to_pil(bpc_layer_id, palettes, bpas, 1)[0]
        //         fimg = Image.new('P', (width_map, height_map))
        //         fimg.putpalette(chunks.getpalette())  # type: ignore
        //
        //         for i, mt_idx in enumerate(bma_layer):
        //             x = i % self.map_width_chunks
        //             y = math.floor(i / self.map_width_chunks)
        //             fimg.paste(
        //                 chunks.crop((0, mt_idx * chunk_width, chunk_width, mt_idx * chunk_width + chunk_height)),
        //                 (x * chunk_width, y * chunk_height)
        //             )
        //
        //         return fimg
        todo!()
    }

    #[allow(clippy::too_many_arguments)]
    #[args(include_collision = "true", include_unknown_data_block = "true", pal_ani = "true", single_frame = "false")]
    pub fn to_pil(
        &self, bpc: InputBpc, bpl: InputBpl, bpas: Vec<Option<InputBpa>>, include_collision: bool,
        include_unknown_data_block: bool, pal_ani: bool, single_frame: bool
    ) -> Vec<IndexedImage> {
        //         """
        //         Converts the entire map into an image, as shown in the game. Each PIL image in the list returned is one
        //         frame. The palettes argument can be retrieved from the map's BPL (bpl.palettes).
        //
        //         The method does not care about frame speeds. Each step of animation is simply returned as a new image,
        //         so if BPAs use different frame speeds, this is ignored; they effectively run at the same speed.
        //         If BPAs are using a different amount of frames per tile, the length of returned list of images will be the lowest
        //         common multiple of the different frame lengths.
        //
        //         If pal_ani=True, then also includes palette animations.
        //
        //         The list of bpas must be the one contained in the bg_list. It needs to contain 8 slots, with empty
        //         slots being None.
        //
        //         TODO: The speed can be increased if we only re-render the changed animated tiles instead!
        //         """
        //
        //         chunk_width = BPC_TILE_DIM * self.tiling_width
        //         chunk_height = BPC_TILE_DIM * self.tiling_height
        //
        //         width_map = self.map_width_chunks * chunk_width
        //         height_map = self.map_height_chunks * chunk_height
        //
        //         final_images = []
        //         lower_layer_bpc = 0 if bpc.number_of_layers == 1 else 1
        //         chunks_lower = bpc.chunks_animated_to_pil(lower_layer_bpc, bpl.palettes, bpas, 1)
        //         for img in chunks_lower:
        //             fimg = Image.new('P', (width_map, height_map))
        //             fimg.putpalette(img.getpalette())  # type: ignore
        //
        //             # yes. self.layer0 is always the LOWER layer! It's the opposite from BPC
        //             for i, mt_idx in enumerate(self.layer0):
        //                 x = i % self.map_width_chunks
        //                 y = math.floor(i / self.map_width_chunks)
        //                 fimg.paste(
        //                     img.crop((0, mt_idx * chunk_width, chunk_width, mt_idx * chunk_width + chunk_height)),
        //                     (x * chunk_width, y * chunk_height)
        //                 )
        //
        //             final_images.append(fimg)
        //             if single_frame:
        //                 break
        //
        //         if bpc.number_of_layers > 1:
        //             # Overlay higher layer tiles
        //             chunks_higher = bpc.chunks_animated_to_pil(0, bpl.palettes, bpas, 1)
        //             len_lower = len(chunks_lower)
        //             len_higher = len(chunks_higher)
        //             if len_higher != len_lower and not single_frame:
        //                 # oh fun! We are missing animations for one of the layers, let's stretch to the lowest common multiple
        //                 lm = lcm(len_higher, len_lower)
        //                 for i in range(len_lower, lm):
        //                     final_images.append(final_images[i % len_lower].copy())
        //                 for i in range(len_higher, lm):
        //                     chunks_higher.append(chunks_higher[i % len_higher].copy())
        //
        //             for j, img in enumerate(chunks_higher):
        //                 fimg = final_images[j]
        //                 assert self.layer1 is not None
        //                 for i, mt_idx in enumerate(self.layer1):
        //                     x = i % self.map_width_chunks
        //                     y = math.floor(i / self.map_width_chunks)
        //
        //                     cropped_img = img.crop((0, mt_idx * chunk_width, chunk_width, mt_idx * chunk_width + chunk_height))
        //                     cropped_img_mask = cropped_img.copy()
        //                     cropped_img_mask.putpalette(MASK_PAL)
        //                     fimg.paste(
        //                         cropped_img,
        //                         (x * chunk_width, y * chunk_height),
        //                         mask=cropped_img_mask.convert('1')
        //                     )
        //                 if single_frame:
        //                     break
        //
        //         final_images_were_rgb_converted = False
        //         if include_collision and self.number_of_collision_layers > 0:
        //             for i, img in enumerate(final_images):
        //                 final_images_were_rgb_converted = True
        //                 # time for some RGB action!
        //                 final_images[i] = img.convert('RGB')
        //                 img = final_images[i]
        //                 draw = ImageDraw.Draw(img, 'RGBA')
        //                 assert self.collision is not None
        //                 for j, col in enumerate(self.collision):
        //                     x = j % self.map_width_camera
        //                     y = math.floor(j / self.map_width_camera)
        //                     if col:
        //                         draw.rectangle((
        //                             (x * BPC_TILE_DIM, y * BPC_TILE_DIM),
        //                             ((x+1) * BPC_TILE_DIM, (y+1) * BPC_TILE_DIM)
        //                         ), fill=(0xff, 0x00, 0x00, 0x40))
        //                 # Second collision layer
        //                 if self.number_of_collision_layers > 1:
        //                     assert self.collision2 is not None
        //                     for j, col in enumerate(self.collision2):
        //                         x = j % self.map_width_camera
        //                         y = math.floor(j / self.map_width_camera)
        //                         if col:
        //                             draw.ellipse((
        //                                 (x * BPC_TILE_DIM, y * BPC_TILE_DIM),
        //                                 ((x+1) * BPC_TILE_DIM, (y+1) * BPC_TILE_DIM)
        //                             ), fill=(0x00, 0x00, 0xff, 0x40))
        //
        //         if include_unknown_data_block and self.unk6 > 0:
        //             fnt = ImageFont.load_default()
        //             for i, img in enumerate(final_images):
        //                 if not final_images_were_rgb_converted:
        //                     final_images[i] = img.convert('RGB')
        //                     img = final_images[i]
        //                 draw = ImageDraw.Draw(img, 'RGBA')
        //                 assert self.unknown_data_block is not None
        //                 for j, unk in enumerate(self.unknown_data_block):
        //                     x = j % self.map_width_camera
        //                     y = math.floor(j / self.map_width_camera)
        //                     if unk > 0:
        //                         draw.text(
        //                             (x * BPC_TILE_DIM, y * BPC_TILE_DIM),
        //                             str(unk),
        //                             font=fnt,
        //                             fill=(0x00, 0xff, 0x00)
        //                         )
        //
        //         # Apply palette animations
        //         if pal_ani and bpl.has_palette_animation and len(bpl.animation_palette) > 0 and not single_frame:
        //             old_images = final_images
        //             old_images_i = 0
        //             final_images = []
        //
        //             for ppal_ani in range(0, len(bpl.animation_palette)):
        //                 current_img = old_images[old_images_i].copy()
        //                 # Switch out the palette with that from the palette animation
        //                 pal_for_frame = itertools.chain.from_iterable(bpl.apply_palette_animations(ppal_ani))
        //                 current_img.putpalette(pal_for_frame)
        //                 final_images.append(current_img)
        //                 old_images_i += 1
        //                 if old_images_i >= len(old_images):
        //                     old_images_i = 0
        //
        //         return final_images
        todo!()
    }

    #[allow(clippy::too_many_arguments)]
    #[args(lower_img = "None", upper_img = "None", force_import = "true", how_many_palettes_lower_layer = "16")]
    pub fn from_pil(
        &mut self, bpc: InputBpc, bpl: InputBpl, lower_img: Option<In256ColIndexedImage>,
        upper_img: Option<In256ColIndexedImage>, force_import: bool,
        how_many_palettes_lower_layer: u16
    ) -> PyResult<()> {
        //         """
        //         Import an entire map from one or two images (for each layer).
        //         Changes all tiles, tilemappings and chunks in the BPC and re-writes the two layer mappings of the BMA.
        //         Imports the palettes of the image to the BPL.
        //         The palettes of the images passed into this method must either identical or can be merged.
        //         The how_many_palettes_lower_layer parameter controls how many palettes
        //         from the lower layer image will then be used.
        //
        //         The passed PIL will be split into separate tiles and the tile's palette index in the tile mapping for this
        //         coordinate is determined by the first pixel value of each tile in the PIL. The PIL
        //         must have a palette containing up to 16 sub-palettes with 16 colors each (256 colors).
        //
        //         If a pixel in a tile uses a color outside of it's 16 color range, an error is thrown or
        //         the color is replaced with 0 of the palette (transparent). This is controlled by
        //         the force_import flag.
        //
        //         Does not import animations. BPA tiles must be manually mapped to the tilemappings of the BPC after the import.
        //         BPL palette animations are not modified.
        //
        //         The input images must have the same dimensions as the BMA (same dimensions as to_pil_single_layer would export).
        //         The input image can have a different number of layers, than the BMA. BPC and BMA layers are changed accordingly.
        //
        //         BMA collision and data layer are not modified.
        //         """
        //         expected_width = self.tiling_width * self.map_width_chunks * BPC_TILE_DIM
        //         expected_height = self.tiling_height * self.map_height_chunks * BPC_TILE_DIM
        //         if (False if lower_img is None else lower_img.width != expected_width) \
        //                 or (False if upper_img is None else upper_img.width != expected_width):
        //             raise ValueError(f(_("Can not import map background: Width of both images must match the current map width: "
        //                                  "{expected_width}px")))
        //         if (False if lower_img is None else lower_img.height != expected_height) \
        //                 or (False if upper_img is None else upper_img.height != expected_height):
        //             raise ValueError(f(_("Can not import map background: Height of both images must match the current map height: "
        //                                  "{expected_height}px")))
        //         upper_palette_palette_color_offset = 0
        //         if upper_img is not None and lower_img is not None and how_many_palettes_lower_layer < BPL_MAX_PAL:
        //             # Combine palettes
        //             lower_palette = lower_img.getpalette()[:how_many_palettes_lower_layer * (BPL_PAL_LEN + 1) * 3]  # type: ignore
        //             upper_palette = upper_img.getpalette()[:(BPL_MAX_PAL - how_many_palettes_lower_layer) * (BPL_PAL_LEN + 1) * 3]  # type: ignore
        //             new_palette = lower_palette + upper_palette
        //             lower_img.putpalette(new_palette)
        //             upper_img.putpalette(new_palette)
        //             # We need to offset the colors in the upper image now, when we read it.
        //             upper_palette_palette_color_offset = how_many_palettes_lower_layer
        //
        //         # Adjust layer numbers
        //         number_of_layers = 2 if upper_img is not None else 1
        //         low_map_idx = 0 if lower_img is not None else 1
        //         if number_of_layers > self.number_of_layers:
        //             self.add_upper_layer()
        //             bpc.add_upper_layer()
        //
        //         # Import tiles, tile mappings and chunks mappings
        //         for layer_idx in range(low_map_idx, number_of_layers):
        //             if layer_idx == 0:
        //                 bpc_layer_id = 0 if bpc.number_of_layers == 1 else 1
        //                 img = lower_img
        //                 palette_offset = 0
        //             else:
        //                 bpc_layer_id = 0
        //                 img = upper_img
        //                 palette_offset = upper_palette_palette_color_offset
        //
        //             tiles, all_possible_tile_mappings, palettes = from_pil(
        //                 img, BPL_IMG_PAL_LEN, BPL_MAX_PAL, BPC_TILE_DIM,
        //                 img.width, img.height, 3, 3, force_import, palette_offset=palette_offset  # type: ignore
        //             )
        //             bpc.import_tiles(bpc_layer_id, tiles)
        //
        //             # Build a new list of chunks / tile mappings for the BPC based on repeating chunks
        //             # in the imported image. Generate chunk mappings.
        //             chunk_mappings = []
        //             chunk_mappings_counter = 1
        //             tile_mappings: List[TilemapEntryProtocol] = []
        //             tiles_in_chunk = self.tiling_width * self.tiling_height
        //             for chk_fst_tile_idx in range(0, self.map_width_chunks * self.map_height_chunks * tiles_in_chunk, tiles_in_chunk):
        //                 chunk = all_possible_tile_mappings[chk_fst_tile_idx:chk_fst_tile_idx+tiles_in_chunk]
        //                 start_of_existing_chunk = search_for_chunk(chunk, tile_mappings)
        //                 if start_of_existing_chunk is not None:
        //                     chunk_mappings.append(int(start_of_existing_chunk / tiles_in_chunk) + 1)
        //                 else:
        //                     tile_mappings += chunk
        //                     chunk_mappings.append(chunk_mappings_counter)
        //                     chunk_mappings_counter += 1
        //
        //             bpc.import_tile_mappings(bpc_layer_id, tile_mappings)
        //             if layer_idx == 0:
        //                 self.layer0 = chunk_mappings
        //             else:
        //                 self.layer1 = chunk_mappings
        //
        //         # Import palettes
        //         bpl.import_palettes(palettes)
        todo!()
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
            self.layer1 = Some(vec![0; self.map_width_chunks as usize * self.map_height_chunks as usize]);
        }
    }

    /// Change the dimensions of the map. Existing tiles and chunks will keep their position in the grid.
    /// If the size is reduced, all tiles and chunks that are moved out of the new dimension box are removed.
    pub fn resize(&mut self, new_width_chunks: u8, new_height_chunks: u8, new_width_camera: u8, new_height_camera: u8) {
        // Layer 0
        self.layer0 = self.layer0.layer_resize(
            self.map_width_chunks,
            new_width_chunks, new_height_chunks
        );
        // Layer 1
        self.layer1 = self.layer1.layer_resize(
            self.map_width_chunks,
            new_width_chunks, new_height_chunks
        );
        // Collision
        self.collision = self.collision.layer_resize(
            self.map_width_camera,
            new_width_camera, new_height_camera
        );
        // Collision 2
        self.collision2 = self.collision2.layer_resize(
            self.map_width_camera,
            new_width_camera, new_height_camera
        );
        // Data Layer
        self.unknown_data_block = self.unknown_data_block.layer_resize(
            self.map_width_camera,
            new_width_camera, new_height_camera
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
                Some(layer1) => layer1[bma_index] = chunk_index
            };
        }
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
            model.map_width_chunks as usize, model.map_height_chunks as usize,
            &model.layer0
        )?);
        match &model.layer1 {
            Some(layer1) => data.extend(Self::convert_layer(
                model.map_width_chunks as usize, model.map_height_chunks as usize,
                layer1
            )?),
            None => {}
        }

        match &model.unknown_data_block {
            Some(unknown_data_block) => data.extend(Self::convert_unknown_data_layer(
                model.map_width_camera as usize, model.map_height_camera as usize,
                unknown_data_block
            )?),
            None => {}
        }

        match &model.collision {
            Some(collision) => data.extend(Self::convert_collision(
                model.map_width_camera as usize, model.map_height_camera as usize,
                collision
            )?),
            None => {}
        }
        match &model.collision2 {
            Some(collision2) => data.extend(Self::convert_collision(
                model.map_width_camera as usize, model.map_height_camera as usize,
                collision2
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
    fn convert_layer(map_width_chunks: usize, map_height_chunks: usize, layer: &[u16]) -> PyResult<BytesMut> {
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
    fn convert_collision(map_width_camera: usize, map_height_camera: usize, collision_layer: &[bool]) -> PyResult<BytesMut> {
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
    fn convert_unknown_data_layer(map_width_camera: usize, map_height_camera: usize, unknown_data_block: &[u8]) -> PyResult<BytesMut> {
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

trait Resizable<T>: Sized where T: Default + Copy {
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
        for (row_i, row) in rows.iter_mut().enumerate() {
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
    fn collect<U>(iter: U) -> Self where U: Iterator<Item=T>;
}

impl<T> Resizable<T> for Vec<T> where T: Default + Copy {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn enumerate(&self) -> Enumerate<Copied<Iter<T>>> {
        self.iter().copied().enumerate()
    }

    fn collect<U>(iter: U) -> Self where U: Iterator<Item=T> {
        iter.collect()
    }
}

impl<T, U> Resizable<U> for Option<T> where T: Resizable<U>, U: Default + Copy {
    fn layer_resize(&self, old_w: u8, new_w: u8, new_h: u8) -> Self {
        self.as_ref().map(|v| v.layer_resize(old_w, new_w, new_h))
    }

    fn len(&self) -> usize {
        self.as_ref().unwrap().len()
    }

    fn enumerate(&self) -> Enumerate<Copied<Iter<U>>> {
        self.as_ref().unwrap().enumerate()
    }

    fn collect<V>(iter: V) -> Self where V: Iterator<Item=U> {
        Some(T::collect(iter))
    }
}
