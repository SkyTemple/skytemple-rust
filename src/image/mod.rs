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

use crate::bytes::{StBytes, StBytesMut};
use crate::image::tilemap_entry::TilemapEntry;
use bytes::{Buf, Bytes, BytesMut};
use std::iter::zip;
use std::iter::Copied;
use std::slice::{ChunksExact, Iter};

use crate::python::*;

// ---

#[derive(Clone)]
pub struct Raster(pub StBytesMut, pub usize, pub usize); // data, width, height

impl Raster {
    pub fn new(width: usize, height: usize) -> Self {
        Self(StBytesMut::from(vec![0; width * height]), width, height)
    }

    /// Returns the part of the Raster enclosed by (x, y, x + w, y + h)
    pub fn crop(&self, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut out = BytesMut::with_capacity(w * h);
        for row in self.0.chunks(self.1).skip(y).take(h) {
            out.extend(row.iter().skip(x).take(w));
        }
        Self(out.into(), w, h)
    }

    // Pastes the other Raster at position (x, y)
    pub fn paste(&mut self, source: Self, x: usize, y: usize) {
        for (self_row, source_row) in zip(
            self.0.chunks_mut(self.1).skip(y).take(source.2),
            source.0.chunks(source.1),
        ) {
            for (self_px, source_px) in zip(
                self_row.iter_mut().skip(x).take(source.1),
                source_row.iter(),
            ) {
                *self_px = *source_px
            }
        }
    }

    // Like paste, but if a value in source is 0 (or % 16 == 0 if uses_sub_palettes), it is not copied.
    pub fn paste_masked(&mut self, source: Self, x: usize, y: usize, uses_sub_palettes: bool) {
        for (self_row, source_row) in zip(
            self.0.chunks_mut(self.1).skip(y).take(source.2),
            source.0.chunks(source.1),
        ) {
            for (self_px, source_px) in zip(
                self_row.iter_mut().skip(x).take(source.1),
                source_row.iter(),
            ) {
                if (!uses_sub_palettes && *source_px != 0) || *source_px % 16 != 0 {
                    *self_px = *source_px
                }
            }
        }
    }
}

pub type Palette = Bytes;

#[derive(Clone)]
pub struct IndexedImage(pub Raster, pub Palette);

pub type TilesGenerator<G> = Vec<PixelGenerator<G>>;
pub type Tile = StBytesMut;
pub type Tiles = Vec<Tile>;
pub type TiledImageDataSeq<T /*: AsRef<[Tile]>*/> = (T, StBytesMut);
pub type TiledImageData = (Tiles, StBytesMut, Vec<TilemapEntry>);

// ---

pub trait InIndexedImage<'py>: Sized {
    const MAX_COLORS: usize;
    const CAN_HAVE_TRANSPARENCY: bool;
    #[cfg(feature = "python")]
    fn unwrap_py(self) -> PyObject;
    #[cfg(feature = "python")]
    fn extract(self, py: Python<'py>) -> PyResult<IndexedImage> {
        match in_from_py(self, py) {
            Ok((raster, pal, width, height)) => Ok(IndexedImage(
                Raster(raster, width, height),
                Bytes::from(pal),
            )),
            Err(e) => Err(e),
        }
    }
    #[cfg(not(feature = "python"))]
    fn extract(self, _py: Python) -> PyResult<IndexedImage>;
}

#[cfg(feature = "python")]
#[derive(FromPyObject)]
pub struct In16ColIndexedImage(pub PyObject); // PIL Image
#[cfg(feature = "python")]
#[derive(FromPyObject)]
/// Like above, but expected to have no transparency.
/// (will be converted to RGB when imported and not already indexed).
/// This is only relevant for the assumption that can be made during import, the first color
/// is still reserved for transparency!
///
/// Why does this exist? We use Pillow to convert and quantize non-indexed images. When going
/// via RGBA there is some loss in the color data when doing that, so ideally we want to avoid
/// having to go through RGBA if possible.
pub struct In16ColSolidIndexedImage(pub PyObject); // PIL Image
#[cfg(feature = "python")]
#[derive(FromPyObject)]
pub struct In256ColIndexedImage(pub PyObject); // PIL Image

#[cfg(not(feature = "python"))]
pub struct In16ColIndexedImage(pub IndexedImage);
#[cfg(not(feature = "python"))]
pub struct In16ColSolidIndexedImage(pub IndexedImage);
#[cfg(not(feature = "python"))]
pub struct In256ColIndexedImage(pub IndexedImage);

impl InIndexedImage<'_> for In16ColIndexedImage {
    const MAX_COLORS: usize = 16;
    const CAN_HAVE_TRANSPARENCY: bool = true;
    #[cfg(feature = "python")]
    fn unwrap_py(self) -> PyObject {
        self.0
    }
    #[cfg(not(feature = "python"))]
    fn extract(self, _py: Python) -> PyResult<IndexedImage> {
        Ok(self.0)
    }
}
impl InIndexedImage<'_> for In16ColSolidIndexedImage {
    const MAX_COLORS: usize = 16;
    const CAN_HAVE_TRANSPARENCY: bool = false;
    #[cfg(feature = "python")]
    fn unwrap_py(self) -> PyObject {
        self.0
    }
    #[cfg(not(feature = "python"))]
    fn extract(self, _py: Python) -> PyResult<IndexedImage> {
        Ok(self.0)
    }
}
impl InIndexedImage<'_> for In256ColIndexedImage {
    const MAX_COLORS: usize = 256;
    const CAN_HAVE_TRANSPARENCY: bool = true;
    #[cfg(feature = "python")]
    fn unwrap_py(self) -> PyObject {
        self.0
    }
    #[cfg(not(feature = "python"))]
    fn extract(self, _py: Python) -> PyResult<IndexedImage> {
        Ok(self.0)
    }
}

// ---

pub mod tilemap_entry;

// ---

pub struct PixelGenerator<T>(pub T)
where
    T: Iterator<Item = u8>;

impl PixelGenerator<FourBppIterator> {
    pub fn pack4bpp(tiledata: &[u8], tile_dim: usize) -> Vec<Self> {
        let chunks: ChunksExact<u8> = tiledata.chunks_exact(tile_dim * tile_dim / 2);
        debug_assert_eq!(chunks.remainder().len(), 0);
        chunks
            .map(|x| PixelGenerator(FourBppIterator::new(x.to_vec())))
            .collect()
    }
    pub fn tiled4bpp(tiledata: &[StBytes]) -> Vec<Self> {
        tiledata
            .iter()
            .map(|x| PixelGenerator(FourBppIterator::new(x.0.clone())))
            .collect()
    }
}

impl<'a> PixelGenerator<Copied<Iter<'a, u8>>> {
    pub fn pack8bpp(tiledata: &'a [u8], tile_dim: usize) -> Vec<Self> {
        let chunks: ChunksExact<u8> = tiledata.chunks_exact(tile_dim * tile_dim);
        debug_assert_eq!(chunks.remainder().len(), 0);
        chunks.map(|x| PixelGenerator(x.iter().copied())).collect()
    }
}

// ---

/// Iterates a byte buffer one nibble at a time (low nibble first)
#[derive(Clone)]
pub struct FourBppIterator(Bytes, u8, bool); // data, next high nibble, on high nibble

impl FourBppIterator {
    pub fn new(data: impl Into<Bytes>) -> Self {
        Self(data.into(), 0, false)
    }
}

impl Iterator for FourBppIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.2 && !self.0.has_remaining() {
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

// ---

pub mod tiled;
