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
use std::iter::repeat;
use std::mem::take;
use bytes::{Buf, BufMut};
use crate::bytes::StBytes;
use crate::python::*;

// Length of a palette in colors. Color 0 is auto-generated (transparent)
const BPL_PAL_LEN: usize = 15;
// Actual colors in an image, (including the color 0)
const BPL_IMG_PAL_LEN: usize = BPL_PAL_LEN + 1;
// Maximum number of palettes
const BPL_MAX_PAL: u8 = 16;
// The value of the fourth color
const BPL_FOURTH_COLOR: u8 = 0x00;
// Number of color channels
const BPL_PAL_ENTRY_LEN: usize = 4;
// Byte size of a Bpl header
const BPL_HEADER_LEN: usize = 4;
// Byte size of a BplAnimationSpec
const BPL_COL_INDEX_ENTRY_LEN: usize = 4;
// Size of a single palette in bytes
const BPL_PAL_SIZE: usize = BPL_PAL_LEN * BPL_PAL_ENTRY_LEN;

#[pyclass(module = "skytemple_rust.st_bpl")]
#[derive(Clone)]
pub struct BplAnimationSpec {
    #[pyo3(get, set)]
    duration_per_frame: u16,
    #[pyo3(get, set)]
    number_of_frames: u16
}

#[pymethods]
impl BplAnimationSpec {
    #[new]
    pub fn new(duration_per_frame: u16, number_of_frames: u16) -> Self {
        Self {
            duration_per_frame, number_of_frames
        }
    }
}

#[pyclass(module = "skytemple_rust.st_bpl")]
#[derive(Clone)]
pub struct Bpl {
    #[pyo3(get, set)]
    number_palettes: u16,
    #[pyo3(get, set)]
    has_palette_animation: bool,
    #[pyo3(get, set)]
    palettes: Vec<Vec<u8>>,
    #[pyo3(get, set)]
    animation_specs: Vec<Py<BplAnimationSpec>>,
    #[pyo3(get, set)]
    animation_palette: Vec<Vec<u8>>
}

impl Bpl {
    /// Gets the actual palettes defined (without dummy grayscale entries).
    pub fn get_real_palettes_slices(&self) -> &[Vec<u8>] {
        &self.palettes[..(self.number_palettes as usize)]
    }

    fn add_dummy_palettes(palettes: &mut Vec<Vec<u8>>) {
        while palettes.len() < BPL_MAX_PAL as usize {
            palettes.push((0..(BPL_MAX_PAL * 3)).map(|i| (i / 3) * BPL_MAX_PAL).collect())
        }
    }
}

#[pymethods]
impl Bpl {
    #[new]
    pub fn new(data: StBytes, py: Python) -> PyResult<Self> {
        let mut data = &data[..];
        let number_palettes = data.get_u16_le();
        let has_palette_animation = data.get_u16_le() > 0;

        let mut palettes: Vec<Vec<u8>> = Vec::with_capacity(number_palettes as usize);
        for _i in 0..number_palettes {
            let mut current_palette = Vec::with_capacity(BPL_IMG_PAL_LEN * 3);
            current_palette.extend(repeat(0).take(3));
            for _ic in 0..BPL_PAL_LEN {
                current_palette.push(data.get_u8());
                current_palette.push(data.get_u8());
                current_palette.push(data.get_u8());
                let unk = data.get_u8();
                debug_assert_eq!(BPL_FOURTH_COLOR, unk);
            }
            palettes.push(current_palette);
        }
        Self::add_dummy_palettes(&mut palettes);

        let mut animation_specs;
        let mut animation_palette;

        if has_palette_animation {
            animation_specs = Vec::with_capacity(number_palettes as usize);
            animation_palette = Vec::with_capacity(number_palettes as usize);

            // Read color index table
            for _i in 0..number_palettes {
                animation_specs.push(Py::new(py, BplAnimationSpec::new(
                    data.get_u16_le(), data.get_u16_le()
                ))?);
            }

            // Read animation color table
            // We don't know the length, so read until EOF
            let mut current_ani_pal = Vec::with_capacity(16 * 3);
            while data.has_remaining() {
                current_ani_pal.push(data.get_u8());
                current_ani_pal.push(data.get_u8());
                current_ani_pal.push(data.get_u8());
                let unk = data.get_u8();
                debug_assert_eq!(BPL_FOURTH_COLOR, unk);
                if current_ani_pal.len() == BPL_PAL_LEN * 3 {
                    animation_palette.push(current_ani_pal);
                    current_ani_pal = Vec::with_capacity(16 * 3);
                }
            }
        } else {
            animation_specs = vec![];
            animation_palette = vec![];
        }

        Ok(Self {
            number_palettes,
            has_palette_animation,
            palettes,
            animation_specs,
            animation_palette
        })
    }
    /// Replace all palettes with the ones passed in.
    /// Animated palette is not changed, but the number of spec entries is adjusted.
    pub fn import_palettes(&mut self, palettes: Vec<Vec<u8>>, py: Python) -> PyResult<()> {
        if palettes.len() > BPL_MAX_PAL as usize {
            return Err(exceptions::PyAssertionError::new_err(format!(
                "Number of palettes must be <= {}, is {}.", BPL_MAX_PAL, palettes.len()
            )))
        }
        let nb_pal_old = self.number_palettes;
        self.number_palettes = palettes.len() as u16;
        self.palettes = palettes;
        if self.has_palette_animation {
            if self.number_palettes < nb_pal_old {
                // Remove the extra spec entries
                let specs = take(&mut self.animation_specs);
                self.animation_specs = specs.into_iter().take(self.number_palettes as usize).collect();
            } else if self.number_palettes > nb_pal_old {
                // Add missing spec entries
                for _ in nb_pal_old..self.number_palettes {
                    self.animation_specs.push(Py::new(py, BplAnimationSpec::new(
                        0, 0
                    ))?);
                }
            }
        }
        Ok(())
    }
    /// Returns a modified copy of self.palettes.
    ///
    ///  This copy is modified to have colors swapped out for the current frame of palette animation.
    ///  The information for this is stored in self.animation_specs and the animation palette in
    ///  self.animation_palette.
    ///
    ///  Only available if self.has_palette_animation.
    ///
    ///  The maximum number of frames is the length of self.animation_palette.
    pub fn apply_palette_animations(&self, frame: u16, py: Python) -> Vec<Vec<u8>> {
        // TODO: First frame is missing: No change!
        let mut f_palettes = Vec::with_capacity(self.animation_specs.len());
        for (i, spec) in self.animation_specs.iter().enumerate() {
            let spec = spec.borrow(py);
            if spec.number_of_frames > 0 {
                let actual_frame_for_pal = frame % spec.number_of_frames;
                let pal_for_frame = &self.animation_palette[actual_frame_for_pal as usize];
                f_palettes.push(
                    repeat(0).take(3)
                        .chain(pal_for_frame.into_iter().copied()).collect()
                )
            } else {
                f_palettes.push(self.palettes[i].clone())
            }
        }
        f_palettes
    }
    /// Returns whether or not the palette with that index is affected by animation.
    pub fn is_palette_affected_by_animation(&self, pal_idx: usize, py: Python) -> bool {
        if self.has_palette_animation {
            self.animation_specs[pal_idx].borrow(py).number_of_frames > 0
        } else {
            false
        }
    }
    /// Gets the actual palettes defined (without dummy grayscale entries).
    pub fn get_real_palettes(&self) -> Vec<Vec<u8>> {
        self.get_real_palettes_slices().to_vec()
    }
    /// Sets the palette properly, adding dummy grayscale entries if needed.
    pub fn set_palettes(&mut self, palettes: Vec<Vec<u8>>) {
        self.palettes = palettes;
        self.number_palettes = self.palettes.len() as u16;
        Self::add_dummy_palettes(&mut self.palettes);
    }
}

#[pyclass(module = "skytemple_rust.st_bpl")]
#[derive(Clone)]
pub struct BplWriter;

#[pymethods]
impl BplWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Bpl, py: Python) -> PyResult<StBytes> {
        let animation_size = if model.has_palette_animation {
            model.number_palettes as usize * BPL_COL_INDEX_ENTRY_LEN + model.animation_palette.len() * BPL_PAL_ENTRY_LEN
        } else {
            0
        } as usize;

        let mut data = Vec::with_capacity(
            BPL_HEADER_LEN * model.number_palettes as usize * BPL_PAL_SIZE + animation_size
        );

        // Header
        data.put_u16_le(model.number_palettes);
        data.put_u16_le(model.has_palette_animation as u16);

        for palette in model.get_real_palettes_slices() {
            // Palettes [Starts with transparent color! This is removed!]
            for (i, color) in palette.into_iter().skip(3).enumerate() {
                data.put_u8(*color);
                if i % 3 == 2 {
                    // Insert the fourth color
                    data.put_u8(BPL_FOURTH_COLOR);
                }
            }
        }

        if model.has_palette_animation {
            // Palette Animation Spec
            for spec in model.animation_specs {
                let spec = spec.borrow(py);
                data.put_u16_le(spec.duration_per_frame);
                data.put_u16_le(spec.number_of_frames);
            }

            // Palette Animation Palette
            for frame in model.animation_palette {
                for (i, color) in frame.into_iter().enumerate() {
                    data.put_u8(color);
                    if i % 3 == 2 {
                        // Insert the fourth color
                        data.put_u8(BPL_FOURTH_COLOR);
                    }
                }
            }
        }

        Ok(StBytes::from(data))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bpl_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bpl";
    let m = PyModule::new(py, name)?;
    m.add_class::<BplAnimationSpec>()?;
    m.add_class::<Bpl>()?;
    m.add_class::<BplWriter>()?;

    Ok((name, m))
}
