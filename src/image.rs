/*
 * Copyright 2021-2021 Parakoopa and the SkyTemple Contributors
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

use std::vec::IntoIter;
use bytes::{Buf, Bytes, BytesMut};
use log::warn;

use crate::python::*;

pub struct Raster(pub Bytes, pub usize, pub usize);  // data, width, height
pub type Palette = Bytes;

#[cfg(feature = "python")]
#[derive(FromPyObject)]
pub struct InWrappedImage(pub PyObject); // PIL Image

#[cfg(not(feature = "python"))]
pub struct InWrappedImage(pub IndexedImage);

pub struct IndexedImage(pub Raster, pub Palette);

#[derive(PartialEq, Eq, Debug)]
pub struct TilemapEntry(u16, bool, bool, u8);  // idx, flip_x, flip_y, pal_idx

impl From<usize> for TilemapEntry {
    fn from(entry: usize) -> Self {
        TilemapEntry(
            // 0000 0011 1111 1111, tile index
            (entry & 0x3FF) as u16,
            // 0000 0100 0000 0000, hflip
            (entry & 0x400) > 0,
            // 0000 1000 0000 0000, vflip
            (entry & 0x800) > 0,
            // 1111 0000 0000 0000, pal index
            ((entry & 0xF000) >> 12) as u8
        )
    }
}

impl From<TilemapEntry> for usize {
    fn from(entry: TilemapEntry) -> Self {
        (entry.0 & 0x3FF) as usize +
            (if entry.1 { 1 } else { 0 } << 10) as usize +
            (if entry.2 { 1 } else { 0 } << 11) as usize +
            ((entry.3 as usize & 0x3F) << 12) as usize
    }
}

pub struct Tile<T>(pub T) where T: Iterator<Item = u8>;

impl Tile<FourBppIterator> {
    pub fn pack4bpp<J>(tiledata: J) -> Vec<Self> where J: Iterator<Item = u8> {
        todo!()
    }
}

impl<T> Tile<T> where T: Iterator<Item = u8> {
    pub fn unpack(us: Vec<Self>) -> Vec<u8> {
        todo!()
    }
}

/// Iterates a byte buffer one nibble at a time (low nibble first)
#[derive(Clone)]
pub struct FourBppIterator(Bytes, u8, bool);  // data, next high nibble, on high nibble

impl FourBppIterator {
    pub fn new(data: impl Into<Bytes>) -> Self {
        Self(data.into(), 0, false)
    }
}

impl Iterator for FourBppIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.0.has_remaining() {
            return None;
        }
        if self.2 {
            self.2 = false;
            Some(self.1)
        } else {
            self.2 = true;
            let byte = self.0.get_u8();
            self.1 = byte >> 4;
            Some(byte & 0x0f)
        }
    }
}

pub type TiledImageDataSeq = (Vec<Tile<IntoIter<u8>>>, Vec<u8>);
pub type TiledImageData = (Vec<Tile<IntoIter<u8>>>, Vec<u8>, Vec<TilemapEntry>);
pub struct TiledImage {}

impl TiledImage {
    pub fn native_to_tiled_seq(
        n_img: IndexedImage, tile_dim: usize, img_dim: usize
    ) -> PyResult<TiledImageDataSeq>
    {
        let (ptiledata, paldata, _) = Self::native_to_tiled(n_img, 16, 1, tile_dim, img_dim, 1, 0)?;
        Ok((ptiledata, paldata))
    }

    pub fn native_to_tiled(
        n_img: IndexedImage, single_palette_size: usize, max_nb_palettes: usize,
        tile_dim: usize, img_dim: usize, chunk_dim: usize, palette_offset: usize
    ) -> PyResult<TiledImageData>
    {
        //--img_width=img_dim
        //--img_height=img_dim
        //--tiling_width=chunk_dim
        //--tiling_height=chunk_dim
        //--force_import=false
        //--optimize=true

        //     max_len_pal = single_palette_size * max_nb_palettes
        //     if pil.mode != 'P':
        //         raise ValueError(_('Can not convert PIL image to PMD tiled image: Must be indexed image (=using a palette)'))
        //     if pil.palette.mode != 'RGB' \
        //             or len(pil.palette.palette) > max_len_pal * 3 \
        //             or len(pil.palette.palette) % single_palette_size * 3 != 0:
        //         raise ValueError(f(_('Can not convert PIL image to PMD tiled image: '
        //                              'Palette must contain max {max_len_pal} RGB colors '
        //                              'and be divisible by {single_palette_size}.')))
        //     if pil.width != img_width or pil.height != img_height:
        //         raise ValueError(f(_('Can not convert PIL image to PMD tiled image: '
        //                              'Image dimensions must be {img_width}x{img_height}px.')))
        //
        //     # Build new palette
        //     new_palette = memoryview(pil.palette.palette)
        //     palettes: List[List[int]] = []
        //     for i, col in enumerate(new_palette):
        //         if i % (single_palette_size * 3) == 0:
        //             cur_palette: List[int] = []
        //             palettes.append(cur_palette)
        //         cur_palette.append(col)
        //
        //     raw_pil_image = pil.tobytes('raw', 'P')
        //     number_of_tiles = int(len(raw_pil_image) / tile_dim / tile_dim)
        //
        //     tiles_with_sum: List[Tuple[int, bytearray]] = [None for __ in range(0, number_of_tiles)]
        //     tilemap: List[TilemapEntry] = [None for __ in range(0, number_of_tiles)]
        //     the_two_px_to_write = [0, 0]
        //
        //     # Set inside the loop:
        //     tile_palette_indices = [None for __ in range(0, number_of_tiles)]
        //
        //     already_initialised_tiles = []
        //
        //     for idx, pix in enumerate(raw_pil_image):
        //         pix = pix + palette_offset * single_palette_size
        //         # Only calculate position for first pixel in two pixel pair (it's always the even one)
        //         if idx % 2 == 0:
        //             x = idx % img_width
        //             y = int(idx / img_width)
        //
        //             # I'm so sorry for this, if someone wants to rewrite this, please go ahead!
        //             chunk_x = math.floor(x / (tile_dim * tiling_width))
        //             chunk_y = math.floor(y / (tile_dim * tiling_height))
        //             tiles_up_to_current_chunk_y = int(img_width / tile_dim * chunk_y * tiling_height)
        //
        //             tile_x = (chunk_x * tiling_width * tiling_height) + (math.floor(x / tile_dim) - (chunk_x * tiling_width))
        //             tile_y = (chunk_y * tiling_height) + (math.floor(y / tile_dim) - (chunk_y * tiling_height))
        //             tile_id = tiles_up_to_current_chunk_y + ((tile_y - tiling_height * chunk_y) * tiling_width) + tile_x
        //
        //             in_tile_x = x - tile_dim * math.floor(x / tile_dim)
        //             in_tile_y = y - tile_dim * math.floor(y / tile_dim)
        //             idx_in_tile = in_tile_y * tile_dim + in_tile_x
        //
        //             nidx = int(idx_in_tile / 2)
        //             #print(f"{idx}@{x}x{y}: {tile_id} : [chunk {chunk_x}x{chunk_y}] "
        //             #      f"{tile_x}x{tile_y} -- {idx_in_tile} : {in_tile_x}x{in_tile_y} = {nidx}")
        //
        //             if tile_id not in already_initialised_tiles:
        //                 already_initialised_tiles.append(tile_id)
        //                 # Begin a new tile
        //                 tiles_with_sum[tile_id] = [0, bytearray(int(tile_dim * tile_dim / 2))]
        //                 # Get the palette index from the first pixel
        //                 tile_palette_indices[tile_id] = math.floor(pix / single_palette_size)
        //
        //         # The "real" value is the value of the color in the currently used palette of the tile
        //         real_pix = pix - (tile_palette_indices[tile_id] * single_palette_size)
        //         if real_pix > (single_palette_size - 1) or real_pix < 0:
        //             # The color is out of range!
        //             if not force_import:
        //                 raise ValueError(f(_("Can not convert PIL image to PMD tiled image: "
        //                                      "The color {pix} (from palette {math.floor(pix / single_palette_size)}) used by "
        //                                      "pixel {x+(idx % 2)}x{y} in tile {tile_id} ({tile_x}x{tile_y} is out of range. "
        //                                      "Expected are colors from palette {tile_palette_indices[tile_id]} ("
        //                                      "{tile_palette_indices[tile_id] * single_palette_size} - "
        //                                      "{(tile_palette_indices[tile_id]+1) * single_palette_size - 1}).")))
        //             # Just set the color to 0 instead if invalid...
        //             else:
        //                 logger.warning(f(_("Can not convert PIL image to PMD tiled image: "
        //                                    "The color {pix} (from palette {math.floor(pix / single_palette_size)}) used by "
        //                                    "pixel {x+(idx % 2)}x{y} in tile {tile_id} ({tile_x}x{tile_y} is out of range. "
        //                                    "Expected are colors from palette {tile_palette_indices[tile_id]} ("
        //                                    "{tile_palette_indices[tile_id] * single_palette_size} - "
        //                                    "{(tile_palette_indices[tile_id]+1) * single_palette_size - 1}).")))
        //             real_pix = 0
        //
        //         # We store 2 bytes as one... in LE
        //         the_two_px_to_write[idx % 2] = real_pix
        //
        //         # Only store when we are on the second pixel
        //         if idx % 2 == 1:
        //             # Little endian:
        //             tiles_with_sum[tile_id][0] += (the_two_px_to_write[0] + the_two_px_to_write[1])
        //             tiles_with_sum[tile_id][1][nidx] = the_two_px_to_write[0] + (the_two_px_to_write[1] << 4)
        //
        //     final_tiles_with_sum: List[Tuple[int, bytearray]] = []
        //     len_final_tiles = 0
        //     # Create tilemap and optimize tiles list
        //     for tile_id, tile_with_sum in enumerate(tiles_with_sum):
        //         reusable_tile_idx = None
        //         flip_x = False
        //         flip_y = False
        //         if optimize:
        //             reusable_tile_idx, flip_x, flip_y = search_for_tile_with_sum(final_tiles_with_sum, tile_with_sum, tile_dim)
        //         if reusable_tile_idx is not None:
        //             tile_id_to_use = reusable_tile_idx
        //         else:
        //             final_tiles_with_sum.append(tile_with_sum)
        //             tile_id_to_use = len_final_tiles
        //             len_final_tiles += 1
        //         tilemap[tile_id] = TilemapEntry(
        //             idx=tile_id_to_use,
        //             pal_idx=tile_palette_indices[tile_id],
        //             flip_x=flip_x,
        //             flip_y=flip_y,
        //             ignore_too_large=True
        //         )
        //     if len_final_tiles > 1024:
        //         raise ValueError(f(_("An image selected to import is too complex. It has too many unique tiles "
        //                              "({len_final_tiles}, max allowed are 1024).\nTry to have less unique tiles. Unique tiles "
        //                              "are 8x8 sections of the images that can't be found anywhere else in the image (including "
        //                              "flipped or with a different sub-palette).")))
        //     final_tiles: List[bytearray] = []
        //     for s, tile in final_tiles_with_sum:
        //         final_tiles.append(tile)
        //     return final_tiles, tilemap, palettes
        todo!()
    }



    pub fn tiled_to_native_seq<J>(
        tiledata: Vec<Tile<J>>, paldata: &[u8], tile_dim: usize, img_dim: usize
    ) -> PyResult<IndexedImage>
        where
            J: Iterator<Item = u8> + Clone
    {
        Self::tiled_to_native(
            (0..).into_iter().map(|i| TilemapEntry(i, false, false, 0)),
            tiledata, paldata, tile_dim, img_dim, 1
        )
    }

    pub fn tiled_to_native<I, J>(
        chunks: I, tiledata: Vec<Tile<J>>, paldata: &[u8], tile_dim: usize, img_dim: usize, chunk_dim: usize
    ) -> PyResult<IndexedImage>
        where
            I: Iterator<Item = TilemapEntry>,
            J: Iterator<Item = u8> + Clone
    {
        let img_width_in_tiles = img_dim / tile_dim;

        let mut imagedata = BytesMut::with_capacity(img_dim * img_dim);
        for (i, chunk_spec) in chunks.enumerate() {
            let tiles_in_chunks = chunk_dim * chunk_dim;
            let chunk_x: usize = (i / tiles_in_chunks) % (img_width_in_tiles / chunk_dim);
            let chunk_y: usize = (i / tiles_in_chunks) / (img_width_in_tiles / chunk_dim);
            let tile_x: usize = (chunk_x * chunk_dim) + (i % chunk_dim);
            let tile_y: usize = (chunk_y * chunk_dim) + ((i / chunk_dim) % chunk_dim);
            let chunk_iter: J;
            if tiledata.len() <= chunk_spec.0 as usize {
                warn!("TiledImage: TileMappingEntry {:?} contains invalid tile reference. Replaced with 0.", chunk_spec);
                chunk_iter = tiledata[0].0.clone();
            } else {
                chunk_iter = tiledata[chunk_spec.0 as usize].0.clone();
            }
            // Since our internal image has one big flat palette, we need to calculate the offset to that
            let pal_start_offset = 16 * chunk_spec.3;
            for (idx, col) in chunk_iter.enumerate() {
                let (x_in_tile, y_in_tile) = Self::_px_pos_flipped(
                    idx % tile_dim, idx / tile_dim, tile_dim, tile_dim,
                    chunk_spec.1, chunk_spec.2
                );
                let real_x = tile_x * tile_dim + x_in_tile;
                let real_y = tile_y * tile_dim + y_in_tile;
                imagedata[idx] = pal_start_offset + col;
            }
        }

        Ok(IndexedImage(Raster(imagedata.freeze(), img_dim, img_dim), paldata.to_vec().into()))
    }

    /// Returns the flipped x and y position for a pixel in a fixed size image.
    /// If x and/or y actually get flipped is controlled by the flip_ params.
    #[inline]
    fn _px_pos_flipped(x: usize, y: usize, w: usize, h: usize, flip_x: bool, flip_y: bool) -> (usize, usize) {
        (if flip_x {w - x - 1} else {x}, if flip_y { h - y - 1} else {y})
    }
}
