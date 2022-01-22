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
use std::cmp::Ordering;
use std::mem::take;
use std::ops::Index;
use bytes::{Buf, BufMut};
use pyo3::mapping::len;
use crate::bytes::StBytes;
use crate::image::{In256ColIndexedImage, IndexedImage, InIndexedImage, PixelGenerator, TiledImage, TilemapEntry};
use crate::python::*;

const BPA_PIXEL_BITLEN: u8 = 4;
const BPA_TILE_DIM: usize = 8;

#[pyclass(module = "st_bpa")]
#[derive(Clone)]
pub struct BpaFrameInfo {
    #[pyo3(get, set)]
    duration_per_frame: u16,
    #[pyo3(get, set)]
    unk2: u16
}

#[pymethods]
impl BpaFrameInfo {
    #[new]
    pub fn new(duration_per_frame: u16, unk2: u16) -> Self {
        Self {
            duration_per_frame, unk2
        }
    }
}

#[pyclass(module = "st_bpa")]
#[derive(Clone)]
pub struct Bpa {
    #[pyo3(get, set)]
    number_of_tiles: u16,
    #[pyo3(get, set)]
    number_of_frames: u16,
    #[pyo3(get, set)]
    tiles: Vec<StBytes>,
    #[pyo3(get, set)]
    frame_info: Vec<Py<BpaFrameInfo>>
}

#[pymethods]
impl Bpa {
    #[new]
    pub fn new(mut data: StBytes, py: Python) -> PyResult<Self> {
        let number_of_tiles = data.get_u16_le();
        let number_of_frames = data.get_u16_le();

        let frame_info = (0..number_of_frames).map(
            |_| Py::new(py, BpaFrameInfo::new(data.get_u16_le(), data.get_u16_le()))
        ).collect::<PyResult<Vec<Py<BpaFrameInfo>>>>()?;

        let mut tiles = Vec::with_capacity((number_of_tiles * number_of_frames) as usize);

        let sz = BPA_TILE_DIM * BPA_TILE_DIM / 2;
        for i in 0..(number_of_tiles * number_of_frames) {
            let pos = i as usize * sz;
            tiles.push(StBytes::from(data.slice(pos..(pos + sz))));
        }

        Ok(Self {
            number_of_tiles,
            number_of_frames,
            tiles,
            frame_info
        })
    }
    /// Returns the tile data of tile no. tile_idx for frame frame_idx.
    pub fn get_tile(&self, tile_idx: usize, frame_idx: usize) -> StBytes {
        self.tiles[frame_idx * self.number_of_tiles as usize + tile_idx].clone()
    }
    /// Exports the BPA as an image, where each row of 8x8 tiles is the
    /// animation set for a single tile. The 16 color palette passed is used to color the image.
    pub fn tiles_to_pil(&self, palette: StBytes) -> IndexedImage {
        // Create a dummy tile map containing all the tiles.
        // The tiles in the BPA are stored so, that each tile of the each frame is next
        // to each other. So the second frame of the first tile is at self.number_of_images + 1.
        let dummy_chunks = (0..self.number_of_tiles)
            .zip(0..self.number_of_frames)
            .map(|(tile_idx, frame_idx)| TilemapEntry(
                (frame_idx * self.number_of_tiles + tile_idx) as usize, false, false, 0
            ));

        let etr = (self.number_of_frames * self.number_of_tiles) as usize;
        let width = self.number_of_frames as usize * BPA_TILE_DIM as usize;
        let height = ((etr as f32 / self.number_of_frames as f32).ceil()) as usize * BPA_TILE_DIM as usize;

        TiledImage::tiled_to_native(
            dummy_chunks,
            PixelGenerator::tiled4bpp(&self.tiles[..]),
            &palette[..], BPA_TILE_DIM as usize, width, height, 1
        )
    }
    #[args(width_in_tiles = "20")]
    /// Exports the BPA as an image, where each row of 8x8 tiles is the
    /// animation set for a single tile. The 16 color palette passed is used to color the image.
    pub fn tiles_to_pil_separate(&self, palette: Vec<u8>, width_in_tiles: usize) -> PyResult<Vec<IndexedImage>> {
        let dummy_chunks = (0..(self.number_of_tiles * self.number_of_frames))
            .map(|tile_idx| TilemapEntry(
               tile_idx as usize, false, false, 0
            ))
            .collect::<Vec<TilemapEntry>>();
        let dummy_chunks_chunked = dummy_chunks.chunks(self.number_of_tiles as usize);

        let width = width_in_tiles * BPA_TILE_DIM as usize;
        let height = ((self.number_of_tiles as f32 / width_in_tiles as f32).ceil()) as usize * BPA_TILE_DIM as usize;

        let mut images = Vec::with_capacity(self.number_of_frames as usize);
        for chunk in dummy_chunks_chunked {
            images.push(TiledImage::tiled_to_native(
                chunk.iter(),
                PixelGenerator::tiled4bpp(&self.tiles[..]),
                &palette[..], BPA_TILE_DIM as usize, width, height, 1
            ))
        }
        Ok(images)
    }
    /// Converts an image back to the BPA.
    /// The format is expected to be the same as tiles_to_pil. This means, that
    /// each rows of tiles is one image set and each column is one frame.
    pub fn pil_to_tiles(&mut self, image: In256ColIndexedImage, py: Python) -> PyResult<()> {
        let image = image.extract(py)?;
        let w = *&image.0.1;
        let h = *&image.0.2;

        self.number_of_frames = (w / BPA_TILE_DIM) as u16;
        self.number_of_tiles = (h / BPA_TILE_DIM) as u16;

        let (mut tiles, pal) = TiledImage::native_to_tiled_seq(
            image, BPA_TILE_DIM, w, h
        )?;

        self.tiles = (0..self.number_of_tiles)
            .zip(0..self.number_of_frames)
            .map(|(tile_idx, frame_idx)| take(&mut tiles[
                    (tile_idx * self.number_of_frames + frame_idx) as usize
                ]).freeze()
            ).collect();

        self._correct_frame_info(py)
    }
    pub fn pil_to_tiles_separate(&mut self, images: Vec<In256ColIndexedImage>, py: Python) -> PyResult<()> {
        let mut frames = Vec::with_capacity(images.len());
        let mut first_image_dims: Option<(usize, usize)> = None;
        for image in images {
            let image = image.extract(py)?;
            let w = *&image.0.1;
            let h = *&image.0.2;
            let (tiles, _) = TiledImage::native_to_tiled_seq(
                image, BPA_TILE_DIM, w, h
            )?;
            frames.push(tiles);
            if first_image_dims.is_none() {
                first_image_dims = Some((w, h));
            }
            if Some((w, h)) != first_image_dims {
                return Err(exceptions::PyValueError::new_err("The dimensions of all images must be the same."))
            }
        }

        self.number_of_frames = frames.len() as u16;
        let (first_w, first_h) = first_image_dims.unwrap();
        self.number_of_tiles = ((first_w * first_h) / (BPA_TILE_DIM * BPA_TILE_DIM)) as u16;
        self.tiles = frames.into_iter()
            .flat_map(|x| x.into_iter()
                .map(|y| y.freeze())
            ).collect();

        self._correct_frame_info(py)
    }
    /// Returns the tiles for the specified frame. Strips the empty dummy tile image at the beginning.
    pub fn tiles_for_frame(&self, frame: u16) -> StBytes {
        self.tiles[(frame * self.number_of_tiles) as usize].clone()
    }

    fn _correct_frame_info(&mut self, py: Python) -> PyResult<()> {
        // Correct frame info size
        let len_finfo = self.frame_info.len();

        match len_finfo.cmp(&(self.number_of_frames as usize)) {
            Ordering::Greater => {
                let finfo = take(&mut self.frame_info);
                self.frame_info = finfo.into_iter().take(self.number_of_frames as usize).collect();
            }
            Ordering::Less => {
                for _ in len_finfo..(self.number_of_frames as usize) {
                    // If the length is shorter, we just copy the last entry
                    if len_finfo > 0 {
                        let entry_before = self.frame_info[len_finfo-1].borrow(py).clone();
                        self.frame_info.push(Py::new(py, BpaFrameInfo::new(
                            entry_before.duration_per_frame,
                            entry_before.unk2
                        ))?);
                    } else {
                        self.frame_info.push(Py::new(py, BpaFrameInfo::new(
                            10, 0
                        ))?);
                    }
                }
            }
            Ordering::Equal => {}
        }
        Ok(())
    }
}

#[pyclass(module = "st_bpa")]
#[derive(Clone)]
pub struct BpaWriter;

#[pymethods]
impl BpaWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Bpa, py: Python) -> PyResult<StBytes> {
        // 4 byte header + animation info for each + images
        let mut data = Vec::with_capacity(
            (4 + (model.number_of_frames * 4) + (model.number_of_tiles * model.number_of_frames / 2)) as usize
        );

        data.put_u16_le(model.number_of_tiles);
        data.put_u16_le(model.number_of_frames);

        assert_eq!(model.number_of_frames as usize, model.frame_info.len());
        for finfo in model.frame_info {
            let finfo = finfo.borrow(py);
            data.put_u16_le(finfo.duration_per_frame);
            data.put_u16_le(finfo.unk2);
        }

        // Tiles
        data.extend(model.tiles.iter().map(|t| t.0.to_vec()).flatten());

        Ok(StBytes::from(data))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bpa_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bpa";
    let m = PyModule::new(py, name)?;
    m.add_class::<BpaFrameInfo>()?;
    m.add_class::<Bpa>()?;
    m.add_class::<BpaWriter>()?;

    Ok((name, m))
}
