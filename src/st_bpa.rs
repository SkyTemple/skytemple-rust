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
use crate::gettext::gettext;
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::TilemapEntry;
use crate::image::{In256ColIndexedImage, InIndexedImage, IndexedImage, PixelGenerator};
use crate::python::*;
use bytes::{Buf, BufMut};
use std::cmp::Ordering;
use std::mem::take;

pub const BPA_TILE_DIM: usize = 8;

#[pyclass(module = "skytemple_rust.st_bpa")]
#[derive(Clone)]
pub struct BpaFrameInfo {
    #[pyo3(get, set)]
    pub duration_per_frame: u16,
    #[pyo3(get, set)]
    pub unk2: u16,
}

#[pymethods]
impl BpaFrameInfo {
    #[new]
    pub fn new(duration_per_frame: u16, unk2: u16) -> Self {
        Self {
            duration_per_frame,
            unk2,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_bpa")]
#[derive(Clone)]
pub struct Bpa {
    #[pyo3(get, set)]
    pub number_of_tiles: u16,
    #[pyo3(get, set)]
    pub number_of_frames: u16,
    #[pyo3(get, set)]
    pub tiles: Vec<StBytes>,
    #[pyo3(get, set)]
    pub frame_info: Vec<Py<BpaFrameInfo>>,
}

#[pymethods]
impl Bpa {
    #[new]
    pub fn new(mut data: StBytes, py: Python) -> PyResult<Self> {
        let number_of_tiles = data.get_u16_le();
        let number_of_frames = data.get_u16_le();

        let frame_info = (0..number_of_frames)
            .map(|_| Py::new(py, BpaFrameInfo::new(data.get_u16_le(), data.get_u16_le())))
            .collect::<PyResult<Vec<Py<BpaFrameInfo>>>>()?;

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
            frame_info,
        })
    }
    /// Returns a new empty Bpa.
    #[classmethod]
    pub fn new_empty(_cls: &PyType) -> PyResult<Self> {
        Ok(Self {
            number_of_tiles: 0,
            number_of_frames: 0,
            tiles: Vec::new(),
            frame_info: Vec::new(),
        })
    }
    /// Returns the tile data of tile no. tile_idx for frame frame_idx.
    pub fn get_tile(&self, tile_idx: usize, frame_idx: usize) -> StBytes {
        self.tiles[frame_idx * self.number_of_tiles as usize + tile_idx].clone()
    }
    /// Exports the BPA as an image, where each row of 8x8 tiles is the
    /// animation set for a single tile. The 16 color palette passed is used to color the image.
    ///
    /// Returns None if the BPA has no tiles.
    pub fn tiles_to_pil(&self, palette: StBytes) -> Option<IndexedImage> {
        if self.number_of_tiles < 1 {
            return None;
        }
        // Create a dummy tile map containing all the tiles.
        // The tiles in the BPA are stored so, that each tile of the each frame is next
        // to each other. So the second frame of the first tile is at self.number_of_images + 1.
        let mut dummy_chunks =
            Vec::with_capacity((self.number_of_tiles * self.number_of_frames) as usize);
        for tile_idx in 0..self.number_of_tiles {
            for frame_idx in 0..self.number_of_frames {
                dummy_chunks.push(TilemapEntry(
                    (frame_idx * self.number_of_tiles + tile_idx) as usize,
                    false,
                    false,
                    0,
                ));
            }
        }

        let etr = (self.number_of_frames * self.number_of_tiles) as usize;
        let width = self.number_of_frames as usize * BPA_TILE_DIM;
        let height = ((etr as f32 / self.number_of_frames as f32).ceil()) as usize * BPA_TILE_DIM;

        Some(TiledImage::tiled_to_native(
            dummy_chunks.into_iter(),
            PixelGenerator::tiled4bpp(&self.tiles[..]),
            palette.iter().copied(),
            BPA_TILE_DIM,
            width,
            height,
            1,
        ))
    }
    #[pyo3(signature = (palette, width_in_tiles = 20))]
    /// Exports the BPA as an image, where each row of 8x8 tiles is the
    /// animation set for a single tile. The 16 color palette passed is used to color the image.
    pub fn tiles_to_pil_separate(
        &self,
        palette: Vec<u8>,
        width_in_tiles: usize,
    ) -> PyResult<Vec<IndexedImage>> {
        if self.number_of_tiles < 1 {
            return Ok(vec![]);
        }
        let dummy_chunks = (0..(self.number_of_tiles * self.number_of_frames))
            .map(|tile_idx| TilemapEntry(tile_idx as usize, false, false, 0))
            .collect::<Vec<TilemapEntry>>();
        let dummy_chunks_chunked = dummy_chunks.chunks(self.number_of_tiles as usize);

        let width = width_in_tiles * BPA_TILE_DIM;
        let height =
            ((self.number_of_tiles as f32 / width_in_tiles as f32).ceil()) as usize * BPA_TILE_DIM;

        let mut images = Vec::with_capacity(self.number_of_frames as usize);
        for chunk in dummy_chunks_chunked {
            images.push(TiledImage::tiled_to_native(
                chunk.iter(),
                PixelGenerator::tiled4bpp(&self.tiles[..]),
                palette.iter().copied(),
                BPA_TILE_DIM,
                width,
                height,
                1,
            ))
        }
        Ok(images)
    }
    /// Converts an image back to the BPA.
    /// The format is expected to be the same as tiles_to_pil. This means, that
    /// each rows of tiles is one image set and each column is one frame.
    pub fn pil_to_tiles(&mut self, image: In256ColIndexedImage, py: Python) -> PyResult<()> {
        let image = image.extract(py)?;
        let w = image.0 .1;
        let h = image.0 .2;

        self.number_of_frames = (w / BPA_TILE_DIM) as u16;
        self.number_of_tiles = (h / BPA_TILE_DIM) as u16;

        let (mut tiles, _pal) = TiledImage::native_to_tiled_seq(image, BPA_TILE_DIM, w, h)?;

        self.tiles = Vec::with_capacity((self.number_of_frames * self.number_of_tiles) as usize);
        for frame_idx in 0..self.number_of_frames {
            for tile_idx in 0..self.number_of_tiles {
                self.tiles.push(
                    take(&mut tiles[(tile_idx * self.number_of_frames + frame_idx) as usize])
                        .freeze(),
                );
            }
        }

        #[cfg(debug_assertions)]
        {
            assert_eq!(
                (self.number_of_tiles * self.number_of_frames) as usize,
                tiles.len()
            );
            assert_eq!(
                (self.number_of_tiles * self.number_of_frames) as usize,
                self.tiles.len()
            );
            for tile in &self.tiles {
                assert_eq!(BPA_TILE_DIM * BPA_TILE_DIM / 2, tile.len());
            }
        }

        self._correct_frame_info(py)
    }
    pub fn pil_to_tiles_separate(
        &mut self,
        images: Vec<In256ColIndexedImage>,
        py: Python,
    ) -> PyResult<()> {
        let mut frames = Vec::with_capacity(images.len());
        let mut first_image_dims: Option<(usize, usize)> = None;
        for image in images {
            let image = image.extract(py)?;
            let w = image.0 .1;
            let h = image.0 .2;
            let (tiles, _) = TiledImage::native_to_tiled_seq(image, BPA_TILE_DIM, w, h)?;
            frames.push(tiles);
            if first_image_dims.is_none() {
                first_image_dims = Some((w, h));
            }
            if Some((w, h)) != first_image_dims {
                return Err(exceptions::PyValueError::new_err(gettext(
                    "The dimensions of all images must be the same.",
                )));
            }
        }

        self.number_of_frames = frames.len() as u16;
        let (first_w, first_h) = first_image_dims.unwrap();
        self.number_of_tiles = ((first_w * first_h) / (BPA_TILE_DIM * BPA_TILE_DIM)) as u16;
        self.tiles = frames
            .into_iter()
            .flat_map(|x| x.into_iter().map(|y| y.freeze()))
            .collect();

        #[cfg(debug_assertions)]
        {
            assert_eq!(
                (self.number_of_tiles * self.number_of_frames) as usize,
                self.tiles.len()
            );
            for tile in &self.tiles {
                assert_eq!(BPA_TILE_DIM * BPA_TILE_DIM / 2, tile.len());
            }
        }

        self._correct_frame_info(py)
    }
    /// Returns the tiles for the specified frame. Strips the empty dummy tile image at the beginning.
    pub fn tiles_for_frame(&self, frame: u16) -> Vec<StBytes> {
        Vec::from(
            &self.tiles[((frame * self.number_of_tiles) as usize)
                ..((frame + 1) * self.number_of_tiles) as usize],
        )
    }

    fn _correct_frame_info(&mut self, py: Python) -> PyResult<()> {
        // Correct frame info size
        let len_finfo = self.frame_info.len();

        match len_finfo.cmp(&(self.number_of_frames as usize)) {
            Ordering::Greater => {
                let finfo = take(&mut self.frame_info);
                self.frame_info = finfo
                    .into_iter()
                    .take(self.number_of_frames as usize)
                    .collect();
            }
            Ordering::Less => {
                for _ in len_finfo..(self.number_of_frames as usize) {
                    // If the length is shorter, we just copy the last entry
                    if len_finfo > 0 {
                        let entry_before = self.frame_info[len_finfo - 1].borrow(py).clone();
                        self.frame_info.push(Py::new(
                            py,
                            BpaFrameInfo::new(entry_before.duration_per_frame, entry_before.unk2),
                        )?);
                    } else {
                        self.frame_info.push(Py::new(py, BpaFrameInfo::new(10, 0))?);
                    }
                }
            }
            Ordering::Equal => {}
        }
        Ok(())
    }
}

#[pyclass(module = "skytemple_rust.st_bpa")]
#[derive(Clone, Default)]
pub struct BpaWriter;

#[pymethods]
impl BpaWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<Bpa>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        // 4 byte header + animation info for each + images
        let mut data = Vec::with_capacity(
            (4 + (model.number_of_frames * 4)
                + (model.number_of_tiles * model.number_of_frames / 2)) as usize,
        );

        data.put_u16_le(model.number_of_tiles);
        data.put_u16_le(model.number_of_frames);

        assert_eq!(model.number_of_frames as usize, model.frame_info.len());
        for finfo in &model.frame_info {
            let finfo = finfo.borrow(py);
            data.put_u16_le(finfo.duration_per_frame);
            data.put_u16_le(finfo.unk2);
        }

        // Tiles
        data.extend(model.tiles.iter().flat_map(|t| t.0.to_vec()));

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

/////////////////////////
/////////////////////////
// BPAs as inputs (for compatibility of including other BPA implementations from Python)
#[cfg(feature = "python")]
pub mod input {
    use crate::bytes::StBytes;
    use crate::python::*;
    use crate::st_bpa::{Bpa, BpaFrameInfo};
    use pyo3::types::PyTuple;

    pub trait BpaProvider: ToPyObject {
        fn get_number_of_tiles(&self, py: Python) -> PyResult<u16>;
        fn get_number_of_frames(&self, py: Python) -> PyResult<u16>;
        fn provide_tiles_for_frame(&self, frame: u16, py: Python) -> PyResult<Vec<StBytes>>;

        // python only (needed for clone):
        fn __get_cloned_tiles(&self, py: Python) -> PyResult<Vec<StBytes>>;
        fn __get_cloned_frame_info(&self, py: Python) -> PyResult<Vec<Py<BpaFrameInfo>>>;
    }

    impl BpaProvider for Py<Bpa> {
        fn get_number_of_tiles(&self, py: Python) -> PyResult<u16> {
            Ok(self.borrow(py).number_of_tiles)
        }

        fn get_number_of_frames(&self, py: Python) -> PyResult<u16> {
            Ok(self.borrow(py).number_of_frames)
        }

        fn provide_tiles_for_frame(&self, frame: u16, py: Python) -> PyResult<Vec<StBytes>> {
            Ok(self.borrow(py).tiles_for_frame(frame))
        }

        fn __get_cloned_tiles(&self, py: Python) -> PyResult<Vec<StBytes>> {
            Ok(self.borrow(py).tiles.clone())
        }

        fn __get_cloned_frame_info(&self, py: Python) -> PyResult<Vec<Py<BpaFrameInfo>>> {
            Ok(self.borrow(py).frame_info.clone())
        }
    }

    impl BpaProvider for PyObject {
        fn get_number_of_tiles(&self, py: Python) -> PyResult<u16> {
            self.getattr(py, "number_of_tiles")?.extract(py)
        }

        fn get_number_of_frames(&self, py: Python) -> PyResult<u16> {
            self.getattr(py, "number_of_frames")?.extract(py)
        }

        fn provide_tiles_for_frame(&self, frame: u16, py: Python) -> PyResult<Vec<StBytes>> {
            let args = PyTuple::new(py, [frame]);
            self.call_method1(py, "tiles_for_frame", args)?.extract(py)
        }

        fn __get_cloned_tiles(&self, py: Python) -> PyResult<Vec<StBytes>> {
            self.getattr(py, "tiles")?.extract(py)
        }

        fn __get_cloned_frame_info(&self, py: Python) -> PyResult<Vec<Py<BpaFrameInfo>>> {
            let frames: Vec<PyObject> = self.getattr(py, "frame_info")?.extract(py)?;
            frames
                .into_iter()
                .map(|x| {
                    Py::new(
                        py,
                        BpaFrameInfo {
                            duration_per_frame: x.getattr(py, "duration_per_frame")?.extract(py)?,
                            unk2: x.getattr(py, "unk2")?.extract(py)?,
                        },
                    )
                })
                .collect::<PyResult<Vec<Py<BpaFrameInfo>>>>()
        }
    }

    pub struct InputBpa(pub Box<dyn BpaProvider>);

    impl<'source> FromPyObject<'source> for InputBpa {
        fn extract(ob: &'source PyAny) -> PyResult<Self> {
            if let Ok(obj) = ob.extract::<Py<Bpa>>() {
                Ok(Self(Box::new(obj)))
            } else {
                Ok(Self(Box::new(ob.to_object(ob.py()))))
            }
        }
    }

    impl IntoPy<PyObject> for InputBpa {
        fn into_py(self, py: Python) -> PyObject {
            self.0.to_object(py)
        }
    }

    impl From<InputBpa> for Bpa {
        fn from(obj: InputBpa) -> Self {
            Python::with_gil(|py| obj.0.to_object(py).extract(py).unwrap())
        }
    }

    impl Clone for InputBpa {
        fn clone(&self) -> Self {
            Python::with_gil(|py| {
                Self(Box::new(
                    Py::new(
                        py,
                        Bpa {
                            number_of_tiles: self.0.get_number_of_tiles(py).unwrap(),
                            number_of_frames: self.0.get_number_of_frames(py).unwrap(),
                            tiles: self.0.__get_cloned_tiles(py).unwrap(),
                            frame_info: self.0.__get_cloned_frame_info(py).unwrap(),
                        },
                    )
                    .unwrap(),
                ))
            })
        }
    }
}

#[cfg(not(feature = "python"))]
pub mod input {
    use crate::bytes::StBytes;
    use crate::python::{PyResult, Python};
    use crate::st_bpa::Bpa;

    pub trait BpaProvider {
        fn get_number_of_tiles(&self, py: Python) -> PyResult<u16>;
        fn get_number_of_frames(&self, py: Python) -> PyResult<u16>;
        fn provide_tiles_for_frame(&self, frame: u16, py: Python) -> PyResult<Vec<StBytes>>;
    }

    impl BpaProvider for Bpa {
        fn get_number_of_tiles(&self, _py: Python) -> PyResult<u16> {
            Ok(self.number_of_tiles)
        }

        fn get_number_of_frames(&self, py: Python) -> PyResult<u16> {
            Ok(self.number_of_frames)
        }

        fn provide_tiles_for_frame(&self, frame: u16, py: Python) -> PyResult<Vec<StBytes>> {
            Ok(self.tiles_for_frame(frame))
        }
    }

    #[derive(Clone)]
    pub struct InputBpa(pub(crate) Bpa);

    impl From<InputBpa> for Bpa {
        fn from(obj: InputBpa) -> Self {
            obj.0
        }
    }
}
