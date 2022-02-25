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
use std::iter::{once, repeat, repeat_with};
use bytes::Buf;
use crate::bytes::StBytes;
use crate::image::{In256ColIndexedImage, IndexedImage, PixelGenerator, InIndexedImage};
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::TilemapEntry;
use crate::python::*;

pub const BGP_RES_WIDTH: usize = 256;
pub const BGP_RES_HEIGHT: usize = 192;
pub const BGP_HEADER_LENGTH: u8 = 32;
pub const BGP_PAL_ENTRY_LEN: u8 = 4;
pub const BGP_PAL_UNKNOWN4_COLOR_VAL: u8 = 0x80;
// The palette is actually a list of smaller palettes. Each palette has this many colors:
pub const BGP_PAL_NUMBER_COLORS: usize = 16;
// The maximum number of palettes supported
pub const BGP_MAX_PAL: u8 = 16;
pub const BGP_TILEMAP_ENTRY_BYTELEN: u8 = 2;
pub const BGP_PIXEL_BITLEN: u8 = 4;
pub const BGP_TILE_DIM: usize = 8;
pub const BGP_RES_WIDTH_IN_TILES: usize = BGP_RES_WIDTH / BGP_TILE_DIM;
pub const BGP_RES_HEIGHT_IN_TILES: usize = BGP_RES_HEIGHT / BGP_TILE_DIM;
pub const BGP_TOTAL_NUMBER_TILES: usize = BGP_RES_WIDTH_IN_TILES * BGP_RES_HEIGHT_IN_TILES;
// All BPGs have this many tiles and tilemapping entries for some reason
pub const BGP_TOTAL_NUMBER_TILES_ACTUALLY: usize = 1024;
// NOTE: Tile 0 is always 0x0.


#[pyclass(module = "skytemple_rust.st_bgp")]
#[derive(Clone)]
pub struct Bgp {
    #[pyo3(get, set)]
    pub palettes: Vec<Vec<u8>>,
    #[pyo3(get, set)]
    pub tilemap: Vec<Py<TilemapEntry>>,
    #[pyo3(get, set)]
    pub tiles: Vec<StBytes>,
}

#[pymethods]
impl Bgp {
    #[new]
    pub fn new(mut data: StBytes) -> PyResult<Self> {
        let palette_begin = data.get_u32_le() as usize;
        let palette_length = data.get_u32_le() as usize;
        let tiles_begin = data.get_u32_le() as usize;
        let tiles_length = data.get_u32_le() as usize;
        let tilemap_data_begin = data.get_u32_le() as usize;
        let tilemap_data_length = data.get_u32_le() as usize;
        // after that 2 unknown values.

        Ok(Self {
            palettes: Self::extract_palette(&data[palette_begin..(palette_begin + palette_length)]),
            tilemap: Self::extract_tilemap(&data[tilemap_data_begin..(tilemap_data_begin + tilemap_data_length)]),
            tiles: Self::extract_tiles(&data[tiles_begin..(tiles_begin + tiles_length)]),
        })
    }

    #[args(ignore_flip_bits = "false")]
    #[allow(unused_variables)]
    /// Convert all tiles of the BGP to one big image.
    /// The resulting image has one large palette with 256 colors.
    /// The ignore_flip_bits is not used.
    ///
    /// The image returned will have the size 256x192.
    pub fn to_pil(&self, ignore_flip_bits: bool, py: Python) -> PyResult<IndexedImage> {
        //         return to_pil(
        //             self.tilemap[:BGP_TOTAL_NUMBER_TILES], self.tiles, self.palettes, BGP_TILE_DIM, BGP_RES_WIDTH,
        //             BGP_RES_HEIGHT, 1, 1, ignore_flip_bits
        //         )
        Ok(TiledImage::tiled_to_native(
            self.tilemap.iter().map(|x| x.borrow(py)),
            PixelGenerator::tiled4bpp(&self.tiles[..]),
             self.palettes.iter().flatten().copied(),
            BGP_TILE_DIM, BGP_RES_WIDTH, BGP_RES_HEIGHT, 1
        ))
    }

    #[args(force_import = "false")]
    #[allow(unused_variables)]
    /// Modify the image data in the BGP by importing the passed image.
    /// The passed image will be split into separate tiles and the tile's palette index
    /// is determined by the first pixel value of each tile in the image. The image
    /// must have a palette containing the 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range the color is replaced with
    /// 0 of the palette (transparent). The "force_import" parameter is ignored.
    ///
    /// The image must have the size 256x192.
    pub fn from_pil(&mut self, pil: In256ColIndexedImage, force_import: bool, py: Python) -> PyResult<()> {
        let (tiles, palettes, tilemap) = TiledImage::native_to_tiled(
            pil.extract(py)?, BGP_PAL_NUMBER_COLORS as u8, BGP_TILE_DIM,
            BGP_RES_WIDTH, BGP_RES_HEIGHT, 1, 0, true
        )?;
        let tiles_len = tiles.len();
        if tiles_len >= 0x3FF {
            return Err(exceptions::PyValueError::new_err("Error when importing: max tile count reached."))
        }
        // + Fill up the tiles and tilemaps to 1024, which seems to be the required default
        // Add the 0 tile (used to clear bgs)
        self.tiles = once(StBytes::from(vec![0; BGP_TILE_DIM * BGP_TILE_DIM / 2]))
            .chain(tiles.into_iter().map(|x| x.0.into()))
            .chain(repeat(StBytes::from(vec![0; BGP_TILE_DIM * BGP_TILE_DIM / 2])).take(BGP_TOTAL_NUMBER_TILES_ACTUALLY - tiles_len))
            .collect();
        // Shift tile indices by 1
        self.tilemap = tilemap.into_iter().map(|mut x| {x.0 += 1; Py::new(py, x)})
            .chain(repeat_with(|| Py::new(py, TilemapEntry::default())).take(BGP_TOTAL_NUMBER_TILES_ACTUALLY - tiles_len))
            .collect::<PyResult<Vec<Py<TilemapEntry>>>>()?;

        self.palettes = palettes.0.chunks(BGP_PAL_NUMBER_COLORS * 3).map(|x| x.to_vec()).collect::<Vec<Vec<u8>>>();
        Ok(())
    }
}

impl Bgp {
    fn extract_palette(data: &[u8]) -> Vec<Vec<u8>> {
//         if self.header.palette_length % 16 != 0:
//             raise ValueError("Invalid BGP image: Palette must be dividable by 16")
//         pal_end = self.header.palette_begin + self.header.palette_length
//         self.palettes = []
//         current_palette = []
//         colors_read_for_current_palette = 0
//         for pal_entry in iter_bytes(self.data, BGP_PAL_ENTRY_LEN, self.header.palette_begin, pal_end):
//             r, g, b, unk = pal_entry
//             current_palette.append(r)
//             current_palette.append(g)
//             current_palette.append(b)
//             colors_read_for_current_palette += 1
//             if colors_read_for_current_palette >= 16:
//                 self.palettes.append(current_palette)
//                 current_palette = []
//                 colors_read_for_current_palette = 0
        todo!()
    }

    fn extract_tilemap(data: &[u8]) -> Vec<Py<TilemapEntry>> {
//         tilemap_end = self.header.tilemap_data_begin + self.header.tilemap_data_length
//         self.tilemap = []
//         for i, entry in enumerate(iter_bytes(self.data, BGP_TILEMAP_ENTRY_BYTELEN, self.header.tilemap_data_begin, tilemap_end)):
//             # NOTE: There will likely be more than 768 (BGP_TOTAL_NUMBER_TILES) tiles. Why is unknown, but the
//             #       rest is just zero padding.
//             self.tilemap.append(TilemapEntry.from_int(int.from_bytes(entry, 'little')))
//         if len(self.tilemap) < BGP_TOTAL_NUMBER_TILES:
//             raise ValueError(f"Invalid BGP image: Too few tiles ({len(self.tilemap)}) in tile mapping."
//                              f"Must be at least {BGP_TOTAL_NUMBER_TILES}.")
        todo!()
    }

    fn extract_tiles(data: &[u8]) -> Vec<StBytes> {
//         self.tiles = []
//         tiles_end = self.header.tiles_begin + self.header.tiles_length
//         # (8 / BGP_PIXEL_BITLEN) = 8 / 4 = 2
//         for tile in iter_bytes(self.data, int(BGP_TILE_DIM * BGP_TILE_DIM / 2), self.header.tiles_begin, tiles_end):
//             # NOTE: Again, the number of tiles is probably bigger than BGP_TOTAL_NUMBER_TILES... (zero padding)
//             self.tiles.append(bytearray(tile))
//         if len(self.tiles) < BGP_TOTAL_NUMBER_TILES:
//             raise ValueError(f"Invalid BGP image: Too few tiles ({len(self.tiles)}) in tile data."
//                              f"Must be at least {BGP_TOTAL_NUMBER_TILES}.")
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_bgp")]
#[derive(Clone, Default)]
pub struct BgpWriter;

#[pymethods]
impl BgpWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<Bgp>, py: Python) -> PyResult<StBytes> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bgp_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bgp";
    let m = PyModule::new(py, name)?;
    m.add_class::<Bgp>()?;
    m.add_class::<BgpWriter>()?;

    Ok((name, m))
}
