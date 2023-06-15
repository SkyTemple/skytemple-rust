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

use crate::bytes::StBytesMut;
use crate::gettext::gettext;
use crate::image::tilemap_entry::{ProvidesTilemapEntry, TilemapEntry};
use crate::image::{
    IndexedImage, Raster, Tile, TiledImageData, TiledImageDataSeq, Tiles, TilesGenerator,
};
use crate::python::*;
use crate::util::init_default_vec;
use log::warn;
use std::fmt::Debug;
use std::vec::IntoIter;

// ---

/// Yields tiles using a tilemap
pub struct ChunkBasedImageIterator(Tiles, IntoIter<TilemapEntry>, usize); // tiles, chunks (tilemap), tile_dim

impl ChunkBasedImageIterator {
    pub fn new(tiles: Tiles, chunks: Vec<TilemapEntry>, tile_dim: usize) -> Self {
        Self(tiles, chunks.into_iter(), tile_dim)
    }
}

impl Iterator for ChunkBasedImageIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|e| {
            let tile = &self.0[e.0];
            let ftile;
            if !e.1 && !e.2 {
                ftile = tile.clone();
            } else if e.1 {
                ftile = flip_tile_x(tile, self.2);
            } else {
                ftile = flip_tile_y(tile, self.2);
            }
            apply_palette_shift(ftile, e.3)
        })
    }
}

// ---

#[derive(Default)]
struct BuiltTile(usize, Tile);

pub struct TiledImage {}

impl TiledImage {
    #[inline]
    pub fn unpack_tiles<T>(tiles: Tiles) -> T
    where
        T: FromIterator<u8>,
    {
        tiles.into_iter().flat_map(|x| x.0).collect()
    }

    /// Note: Output images are 4bpp
    pub fn native_to_tiled_seq(
        n_img: IndexedImage,
        tile_dim: usize,
        img_width: usize,
        img_height: usize,
    ) -> PyResult<TiledImageDataSeq<Tiles>> {
        // "slow" but "direct"/"clean" way:
        //let (ptiledata, paldata, chunks) = Self::native_to_tiled(
        //    n_img, 16, tile_dim, img_width, img_height, 1, 0, true
        //)?;
        //Ok((ChunkBasedImageIterator::new(ptiledata, chunks, tile_dim), (&paldata[0..16 * 3]).to_vec()))

        // optimized (by not "optimizing" duplicate tiles, thus getting all of them sequentially directly):
        let (ptiledata, paldata, _) =
            Self::native_to_tiled(n_img, 16, tile_dim, img_width, img_height, 1, 0, false)?;
        Ok((ptiledata, paldata))
    }

    /// Note: Output images are 4bpp
    /// Note: If optimize_chunks is false, an empty chunk list is returned. Please use native_to_tiled_seq instead.
    #[allow(clippy::too_many_arguments)]
    pub fn native_to_tiled(
        n_img: IndexedImage,
        single_palette_size: u8,
        tile_dim: usize,
        img_width: usize,
        img_height: usize,
        chunk_dim: usize,
        palette_offset: usize,
        optimize_chunks: bool,
    ) -> PyResult<TiledImageData> {
        if n_img.0 .1 != img_width || n_img.0 .2 != img_height {
            return Err(exceptions::PyValueError::new_err(format!(
                "Can not convert PIL image to PMD tiled image: Image dimensions must be {}x{}px.",
                img_width, img_height
            )));
        }

        let number_of_tiles = (img_width * img_height) / tile_dim / tile_dim;

        let mut tiles_with_sum: Vec<BuiltTile> = init_default_vec(number_of_tiles);
        let mut chunks: Vec<TilemapEntry> = Vec::with_capacity(number_of_tiles);
        let mut the_two_px_to_write: [u8; 2] = [0, 0];
        let mut tile_palette_indices: Vec<u8> = init_default_vec(number_of_tiles);
        let mut already_initialised_tiles: Vec<usize> = Vec::with_capacity(number_of_tiles);

        let mut x = 0;
        let mut y = 0;
        let mut tile_x = 0;
        let mut tile_y = 0;
        let mut tile_id = 0;
        let mut nidx = 0;
        for (idx, pix) in n_img.0 .0.into_iter().enumerate() {
            let pix: usize = pix as usize + palette_offset * single_palette_size as usize;
            // Only calculate position for first pixel in two pixel pair (it's always the even one)
            if idx % 2 == 0 {
                // I'm (still :( ) so sorry for this, if someone wants to rewrite this, please go ahead!
                x = idx % img_width;
                y = idx / img_width;
                let chunk_x = x / (tile_dim * chunk_dim);
                let chunk_y = y / (tile_dim * chunk_dim);
                let tiles_up_to_current_chunk_y = img_width / tile_dim * chunk_y * chunk_dim;

                tile_x =
                    (chunk_x * chunk_dim * chunk_dim) + ((x / tile_dim) - (chunk_x * chunk_dim));
                tile_y = (chunk_y * chunk_dim) + ((y / tile_dim) - (chunk_y * chunk_dim));
                tile_id = tiles_up_to_current_chunk_y
                    + ((tile_y - chunk_dim * chunk_y) * chunk_dim)
                    + tile_x;

                let in_tile_x = x - tile_dim * (x / tile_dim);
                let in_tile_y = y - tile_dim * (y / tile_dim);
                let idx_in_tile = in_tile_y * tile_dim + in_tile_x;

                nidx = idx_in_tile / 2;

                if !already_initialised_tiles.contains(&tile_id) {
                    already_initialised_tiles.push(tile_id);
                    // Begin a new tile
                    tiles_with_sum[tile_id] =
                        BuiltTile(0, init_default_vec(tile_dim * tile_dim / 2));
                    // Get the palette index from the first pixel
                    tile_palette_indices[tile_id] = (pix / single_palette_size as usize) as u8;
                }
            }
            // The "real" value is the value of the color in the currently used palette of the tile
            let mut real_pix: i64 =
                pix as i64 - (tile_palette_indices[tile_id] as i64 * single_palette_size as i64);
            if real_pix > (single_palette_size - 1) as i64 || real_pix < 0 {
                warn!(
                    "Can not convert PIL image to PMD tiled image: \
                      The color {} (from palette {}) used by \
                      pixel {}x{} in tile {} ({}x{} is out of range. \
                      Expected are colors from palette {} ({} - {}).",
                    pix,
                    pix / single_palette_size as usize,
                    x + (idx % 2),
                    y,
                    tile_id,
                    tile_x,
                    tile_y,
                    tile_palette_indices[tile_id],
                    tile_palette_indices[tile_id] * single_palette_size,
                    (tile_palette_indices[tile_id] + 1) * (single_palette_size - 1)
                );
                real_pix = 0
            }

            // We store 2 bytes as one... in LE
            the_two_px_to_write[idx % 2] = real_pix as u8;
            if idx % 2 == 1 {
                // Only store when we are on the second pixel
                tiles_with_sum[tile_id].0 +=
                    (the_two_px_to_write[0] + the_two_px_to_write[1]) as usize;
                tiles_with_sum[tile_id].1[nidx] =
                    the_two_px_to_write[0] + (the_two_px_to_write[1] << 4)
            }
        }

        let mut final_tiles_with_sum: Vec<BuiltTile>;
        if optimize_chunks {
            // Create tilemap and optimize tiles list
            final_tiles_with_sum = Vec::with_capacity(number_of_tiles);
            tiles_with_sum
                .into_iter()
                .enumerate()
                .for_each(|(tile_id, built_tile)| {
                    let (reusable_tile_idx, flip_x, flip_y) = Self::_search_for_tile_with_sum(
                        &final_tiles_with_sum,
                        &built_tile,
                        tile_dim,
                    );

                    let tile_id_to_use = match reusable_tile_idx {
                        Some(x) => x,
                        None => {
                            final_tiles_with_sum.push(built_tile);
                            final_tiles_with_sum.len() - 1
                        }
                    };
                    chunks.push(TilemapEntry(
                        tile_id_to_use,
                        flip_x,
                        flip_y,
                        tile_palette_indices[tile_id],
                    ))
                });
        } else {
            final_tiles_with_sum = tiles_with_sum;
        }
        if final_tiles_with_sum.len() > 1024 {
            return Err(create_value_user_error(gettext!(
                "An image selected to import is too complex. It has too many unique tiles ({}, max allowed are 1024).\nTry to have less unique tiles. Unique tiles are 8x8 sections of the images that can't be found anywhere else in the image (including flipped or with a different sub-palette).",
                final_tiles_with_sum.len()
            )));
        }

        Ok((
            final_tiles_with_sum.into_iter().map(|x| x.1).collect(),
            StBytesMut::from(n_img.1),
            chunks,
        ))
    }

    pub fn tiled_to_native_seq<J, P>(
        tiledata: TilesGenerator<J>,
        paldata: P,
        tile_dim: usize,
        img_width: usize,
        img_height: usize,
    ) -> IndexedImage
    where
        J: Iterator<Item = u8> + Clone,
        P: Iterator<Item = u8>,
    {
        let number_chunks = img_width * img_height / tile_dim / tile_dim;
        Self::tiled_to_native(
            (0..number_chunks).map(|i| TilemapEntry(i, false, false, 0)),
            tiledata,
            paldata,
            tile_dim,
            img_width,
            img_height,
            1,
        )
    }

    pub fn tiled_to_native<I, J, T, P>(
        chunks: I,
        tiledata: TilesGenerator<J>,
        paldata: P,
        tile_dim: usize,
        img_width: usize,
        img_height: usize,
        chunk_dim: usize,
    ) -> IndexedImage
    where
        I: Iterator<Item = T>,
        J: Iterator<Item = u8> + Clone,
        T: ProvidesTilemapEntry + Debug,
        P: Iterator<Item = u8>,
    {
        let img_width_in_tiles = img_width / tile_dim;

        let mut imagedata: StBytesMut = init_default_vec(img_width * img_height);
        for (i, chunk_spec) in chunks.enumerate() {
            let tiles_in_chunks = chunk_dim * chunk_dim;
            let chunk_x: usize = (i / tiles_in_chunks) % (img_width_in_tiles / chunk_dim);
            let chunk_y: usize = (i / tiles_in_chunks) / (img_width_in_tiles / chunk_dim);
            let tile_x: usize = (chunk_x * chunk_dim) + (i % chunk_dim);
            let tile_y: usize = (chunk_y * chunk_dim) + ((i / chunk_dim) % chunk_dim);
            let chunk_iter: J = if tiledata.len() <= chunk_spec.idx() {
                warn!("TiledImage: TileMappingEntry {:?} contains invalid tile reference. Replaced with 0.", chunk_spec);
                tiledata[0].0.clone()
            } else {
                tiledata[chunk_spec.idx()].0.clone()
            };
            // Since our internal image has one big flat palette, we need to calculate the offset to that
            let pal_start_offset = 16 * chunk_spec.pal_idx();
            for (idx, col) in chunk_iter.enumerate() {
                let (x_in_tile, y_in_tile) = Self::_px_pos_flipped(
                    idx % tile_dim,
                    idx / tile_dim,
                    tile_dim,
                    tile_dim,
                    chunk_spec.flip_x(),
                    chunk_spec.flip_y(),
                );
                let real_x = tile_x * tile_dim + x_in_tile;
                let real_y = tile_y * tile_dim + y_in_tile;
                imagedata[real_y * img_width + real_x] = pal_start_offset + col;
            }
        }

        debug_assert_eq!(img_width * img_height, imagedata.len());
        IndexedImage(Raster(imagedata, img_width, img_height), paldata.collect())
    }

    /// In the provided list of tile mappings, find an existing chunk.
    /// Returns the position of the first tile of the chunk or None if not found.
    pub fn search_for_chunk(
        chunk: &[TilemapEntry],
        tile_mappings: &[TilemapEntry],
    ) -> Option<usize> {
        for (i, check_chunk) in tile_mappings.chunks(chunk.len()).enumerate() {
            if chunk == check_chunk {
                return Some(i);
            }
        }
        None
    }

    /// Search for the tile, or a flipped version of it, in tiles and return the index and flipped state
    /// Increases performance by comparing the bytes sum of each tile before actually compare them
    fn _search_for_tile_with_sum(
        tiles: &[BuiltTile],
        needle: &BuiltTile,
        tile_dim: usize,
    ) -> (Option<usize>, bool, bool) {
        for (i, candidate) in tiles.iter().enumerate() {
            if candidate.0 == needle.0 {
                if candidate.1 == needle.1 {
                    return (Some(i), false, false);
                }
                let x_flipped = flip_tile_x(&candidate.1, tile_dim);
                if x_flipped == needle.1 {
                    return (Some(i), true, false);
                } else if flip_tile_y(&candidate.1, tile_dim) == needle.1 {
                    return (Some(i), false, true);
                } else if flip_tile_y(&x_flipped, tile_dim) == needle.1 {
                    return (Some(i), true, true);
                }
            }
        }
        (None, false, false)
    }

    /// Returns the flipped x and y position for a pixel in a fixed size image.
    /// If x and/or y actually get flipped is controlled by the flip_ params.
    #[inline]
    fn _px_pos_flipped(
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        flip_x: bool,
        flip_y: bool,
    ) -> (usize, usize) {
        (
            if flip_x { w - x - 1 } else { x },
            if flip_y { h - y - 1 } else { y },
        )
    }
}

// ---

/// Flip all pixels in tile on the x-axis
pub fn flip_tile_x(tile: &Tile, tile_dim: usize) -> Tile {
    let mut tile_flipped: Tile = init_default_vec(tile.len());
    for (i, b) in tile.iter().enumerate() {
        let row_idx = i * 2 % tile_dim;
        let col_idx = i * 2 / tile_dim;
        tile_flipped[(col_idx * tile_dim + (tile_dim - 1 - row_idx)) / 2] =
            (b & 0x0F) << 4 | (b & 0xF0) >> 4;
    }
    tile_flipped
}

/// Flip all pixels in tile on the y-axis
pub fn flip_tile_y(tile: &Tile, tile_dim: usize) -> Tile {
    let mut tile_flipped: Tile = init_default_vec(tile.len());
    for (i, b) in tile.iter().enumerate() {
        let row_idx = i * 2 % tile_dim;
        let col_idx = i * 2 / tile_dim;
        tile_flipped[((tile_dim - 1 - col_idx) * tile_dim + row_idx) / 2] = *b;
    }
    tile_flipped
}

/// Shift all color values by 16 * palette_num
pub fn apply_palette_shift(tile: Tile, palette: u8) -> Tile {
    tile.into_iter().map(|x| x + (16 * palette)).collect()
}
