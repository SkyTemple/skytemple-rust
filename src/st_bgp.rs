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
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::TilemapEntry;
use crate::image::{In256ColIndexedImage, InIndexedImage, IndexedImage, PixelGenerator};
use crate::python::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use itertools::Itertools;
use std::cmp::{max, min};
use std::io::Cursor;
use std::iter::{once, repeat, repeat_with};

pub const BGP_RES_WIDTH: usize = 256;
pub const BGP_RES_HEIGHT: usize = 192;
pub const BGP_HEADER_LENGTH: u32 = 32;
pub const BGP_PAL_ENTRY_LEN: u8 = 4;
pub const BGP_PAL_UNKNOWN4_COLOR_VAL: u8 = 0x80;
// The palette is actually a list of smaller palettes. Each palette has this many colors:
pub const BGP_PAL_NUMBER_COLORS: usize = 16;
// The maximum number of palettes supported
pub const BGP_MAX_PAL: u8 = 16;
pub const BGP_TILEMAP_ENTRY_BYTELEN: usize = 2;
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
    unknown1: u32,
    unknown2: u32,
}

#[pymethods]
impl Bgp {
    #[new]
    pub fn new(data: StBytes, py: Python) -> PyResult<Self> {
        let mut header = Cursor::new(&data.0);
        let palette_begin = header.get_u32_le() as usize;
        debug_assert_eq!(BGP_HEADER_LENGTH as usize, palette_begin);
        let palette_length = header.get_u32_le() as usize;
        let tiles_begin = header.get_u32_le() as usize;
        let tiles_length = header.get_u32_le() as usize;
        let tilemap_data_begin = header.get_u32_le() as usize;
        let tilemap_data_length = header.get_u32_le() as usize;
        let unknown1 = header.get_u32_le();
        let unknown2 = header.get_u32_le();

        Ok(Self {
            palettes: Self::extract_palette(&data[palette_begin..(palette_begin + palette_length)]),
            tilemap: Self::extract_tilemap(
                &data[tilemap_data_begin..(tilemap_data_begin + tilemap_data_length)],
                py,
            )?,
            tiles: Self::extract_tiles(
                &data[tiles_begin..min(tiles_begin + tiles_length, data.len())],
            ),
            unknown1,
            unknown2,
        })
    }

    /// Convert all tiles of the BGP to one big image.
    /// The resulting image has one large palette with 256 colors.
    /// The ignore_flip_bits is not used.
    ///
    /// The image returned will have the size 256x192.
    #[pyo3(signature = (ignore_flip_bits = false))]
    #[allow(unused_variables)]
    pub fn to_pil(&self, ignore_flip_bits: bool, py: Python) -> PyResult<IndexedImage> {
        Ok(TiledImage::tiled_to_native(
            self.tilemap
                .iter()
                .map(|x| x.borrow(py))
                .take(BGP_TOTAL_NUMBER_TILES),
            PixelGenerator::tiled4bpp(&self.tiles[..]),
            self.palettes.iter().flatten().copied(),
            BGP_TILE_DIM,
            BGP_RES_WIDTH,
            BGP_RES_HEIGHT,
            1,
        ))
    }

    /// Modify the image data in the BGP by importing the passed image.
    /// The passed image will be split into separate tiles and the tile's palette index
    /// is determined by the first pixel value of each tile in the image. The image
    /// must have a palette containing the 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range the color is replaced with
    /// 0 of the palette (transparent). The "force_import" parameter is ignored.
    ///
    /// The image must have the size 256x192.
    #[pyo3(signature = (pil, force_import = false))]
    #[allow(unused_variables)]
    pub fn from_pil(
        &mut self,
        pil: In256ColIndexedImage,
        force_import: bool,
        py: Python,
    ) -> PyResult<()> {
        let (tiles, palettes, tilemap) = TiledImage::native_to_tiled(
            pil.extract(py)?,
            BGP_PAL_NUMBER_COLORS as u8,
            BGP_TILE_DIM,
            BGP_RES_WIDTH,
            BGP_RES_HEIGHT,
            1,
            0,
            true,
        )?;
        let tiles_len = tiles.len();
        if tiles_len >= 0x3FF {
            return Err(exceptions::PyValueError::new_err(
                "Error when importing: max tile count reached.",
            ));
        }
        // + Fill up the tiles and tilemaps to 1024, which seems to be the required default
        // Add the 0 tile (used to clear bgs)
        self.tiles = once(StBytes::from(vec![0; BGP_TILE_DIM * BGP_TILE_DIM / 2]))
            .chain(tiles.into_iter().map(|x| x.0.into()))
            .chain(
                repeat(StBytes::from(vec![0; BGP_TILE_DIM * BGP_TILE_DIM / 2])).take(max(
                    0isize,
                    BGP_TOTAL_NUMBER_TILES_ACTUALLY as isize - tiles_len as isize,
                )
                    as usize),
            )
            .collect();
        // Shift tile indices by 1
        self.tilemap = tilemap
            .into_iter()
            .map(|mut x| {
                x.0 += 1;
                Py::new(py, x)
            })
            .chain(
                repeat_with(|| Py::new(py, TilemapEntry::default())).take(max(
                    0isize,
                    BGP_TOTAL_NUMBER_TILES_ACTUALLY as isize - tiles_len as isize,
                )
                    as usize),
            )
            .collect::<PyResult<Vec<Py<TilemapEntry>>>>()?;

        self.palettes = palettes
            .0
            .chunks(BGP_PAL_NUMBER_COLORS * 3)
            .map(|x| x.to_vec())
            .collect::<Vec<Vec<u8>>>();
        Ok(())
    }
}

impl Bgp {
    fn extract_palette(data: &[u8]) -> Vec<Vec<u8>> {
        let mut palettes = Vec::with_capacity(BGP_MAX_PAL as usize);
        for palette in data.chunks(BGP_PAL_NUMBER_COLORS * BGP_PAL_ENTRY_LEN as usize) {
            palettes.push(
                palette
                    .chunks(BGP_PAL_ENTRY_LEN as usize)
                    .flat_map(|bl| vec![bl[0], bl[1], bl[2]])
                    .collect::<Vec<u8>>(),
            )
        }
        palettes
    }

    fn extract_tilemap(mut data: &[u8], py: Python) -> PyResult<Vec<Py<TilemapEntry>>> {
        let mut tilemap = Vec::with_capacity(data.len() / 2);
        while data.has_remaining() {
            tilemap.push(Py::new(py, TilemapEntry::from(data.get_u16_le() as usize))?)
        }
        Ok(tilemap)
    }

    fn extract_tiles(data: &[u8]) -> Vec<StBytes> {
        data.chunks(BGP_TILE_DIM * BGP_TILE_DIM / 2)
            .map(StBytes::from)
            .collect()
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
        let model = model.borrow(py);
        let palettes_length =
            model.palettes.len() as u32 * BGP_PAL_NUMBER_COLORS as u32 * BGP_PAL_ENTRY_LEN as u32;
        let tiles_length = (model.tiles.len() * (BGP_TILE_DIM * BGP_TILE_DIM / 2)) as u32;
        let tilemapping_length = (model.tilemap.len() * BGP_TILEMAP_ENTRY_BYTELEN) as u32;
        let palettes_begin = BGP_HEADER_LENGTH;
        let tilemapping_begin = palettes_begin + palettes_length;
        let tiles_begin = tilemapping_begin + tilemapping_length;

        let mut header = BytesMut::with_capacity(32);
        header.put_u32_le(palettes_begin);
        header.put_u32_le(palettes_length);
        header.put_u32_le(tiles_begin);
        header.put_u32_le(tiles_length);
        header.put_u32_le(tilemapping_begin);
        header.put_u32_le(tilemapping_length);
        header.put_u32_le(model.unknown1);
        header.put_u32_le(model.unknown2);

        Ok(StBytes(
            header
                .into_iter()
                .chain(
                    model
                        .palettes
                        .iter()
                        .flatten()
                        .chunks(3)
                        .into_iter()
                        .flat_map(|c| {
                            c.into_iter()
                                .copied()
                                .chain(once(BGP_PAL_UNKNOWN4_COLOR_VAL))
                        }),
                )
                .chain(
                    model
                        .tilemap
                        .iter()
                        .flat_map(|tm| (tm.borrow(py)._to_int() as u16).to_le_bytes()),
                )
                .chain(model.tiles.iter().flat_map(|v| v.0.iter().copied()))
                .collect::<Bytes>(),
        ))
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
