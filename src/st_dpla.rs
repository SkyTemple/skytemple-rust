/*
 * Copyright 2021-2023 Capypara and the SkyTemple Contributors
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
use crate::python::*;
use crate::st_sir0::{Sir0Error, Sir0Result, Sir0Serializable};
use bytes::{Buf, BufMut, BytesMut};
use itertools::Itertools;

pub const DPLA_COLORS_PER_PALETTE: usize = 16;

#[pyclass(module = "skytemple_rust.st_dpla")]
#[derive(Clone)]
pub struct Dpla {
    #[pyo3(get, set)]
    pub colors: Vec<Vec<u8>>,
    #[pyo3(get, set)]
    pub durations_per_frame_for_colors: Vec<u16>,
}

#[pymethods]
impl Dpla {
    #[new]
    pub fn new(data: StBytes, pointer_to_pointers: u32) -> PyResult<Self> {
        let toc_pointers: Vec<_> = ((pointer_to_pointers as usize)..data.len())
            .step_by(4)
            .map(|i| data[i..i + 4].as_ref().get_u32_le())
            .collect();
        //  A list of colors stored in this file. The colors are lists of RGB values: [R, G, B, R, G, B...]
        let mut colors: Vec<Vec<u8>> = Vec::with_capacity(toc_pointers.len());
        let mut durations_per_frame_for_colors: Vec<u16> = Vec::with_capacity(toc_pointers.len());
        for pnt in toc_pointers {
            let mut slice = &data[(pnt as usize)..];
            // 0x0         2           uint16      (NbColors) The amount of colors in this entry.
            let number_colors = slice.get_u16_le() as usize;
            // 0x2         2           uint16      duration per frame
            durations_per_frame_for_colors.push(slice.get_u16_le());
            // 0x4         (NbColors * 4)          A list of colors. Always at least 4 bytes even when empty! Is completely 0 if nb of color == 0 !
            // [
            //   0x0     4           RGBX32      A color.
            //   ...
            // ]
            let mut frame_colors = Vec::with_capacity(number_colors * 3);
            let mut entries = &slice[..(number_colors * 4)];
            while entries.has_remaining() {
                frame_colors.push(entries.get_u8());
                frame_colors.push(entries.get_u8());
                frame_colors.push(entries.get_u8());
                let unk = entries.get_u8();
                debug_assert_eq!(128, unk);
            }
            colors.push(frame_colors);
        }
        Ok(Self {
            colors,
            durations_per_frame_for_colors,
        })
    }

    /// Returns the color palette at the given frame id. Returned is a stream of RGB colors: [R, G, B, R, G, B...].
    /// Returned are always 16 colors. If the palette file has more than 16 colors, the pal_idx specifies what set
    /// of 16 colors to return.
    pub fn get_palette_for_frame(&self, pal_idx: usize, frame_id: usize) -> PyResult<Vec<u8>> {
        let colors = self.colors.get((pal_idx * 16)..((pal_idx + 1) * 16));
        if let Some(colors) = colors {
            let result = colors
                .iter()
                .map(|color| {
                    let color_len = color.len() / 3;
                    if color.is_empty() {
                        Ok(&[0u8, 0, 0][..])
                    } else {
                        // flatten_ok needs a result.
                        color
                            .get(((frame_id % color_len) * 3)..((frame_id % color_len) * 3) + 3)
                            .ok_or(())
                    }
                })
                .flatten_ok()
                .map(Result::<&_, _>::copied)
                .collect::<Result<Vec<u8>, ()>>();
            match result {
                Err(_) => Err(exceptions::PyIndexError::new_err(gettext(
                    "Palette is invalid.",
                ))),
                Ok(palette) => Ok(palette),
            }
        } else {
            Err(exceptions::PyIndexError::new_err(gettext(
                "Palette index out of range.",
            )))
        }
    }

    pub fn has_for_palette(&self, palette_idx: usize) -> bool {
        self.colors
            .get(palette_idx * DPLA_COLORS_PER_PALETTE)
            .map_or(false, |x| !x.is_empty())
    }

    pub fn get_frame_count_for_palette(&self, palette_idx: usize) -> PyResult<usize> {
        self.colors
            .get(palette_idx * DPLA_COLORS_PER_PALETTE)
            .ok_or_else(|| {
                exceptions::PyValueError::new_err(gettext("This palette has no animation."))
            })
            .map(|x| x.len() / 3)
    }

    pub fn enable_for_palette(&mut self, palid: usize) {
        if !self.has_for_palette(palid) {
            // Add one entry, this enables it.
            while self.colors.len() < ((palid + 1) * DPLA_COLORS_PER_PALETTE) {
                self.colors.push(vec![0, 0, 0])
            }
            for entry in self
                .colors
                .iter_mut()
                .skip(palid * DPLA_COLORS_PER_PALETTE)
                .take(DPLA_COLORS_PER_PALETTE)
            {
                if entry.is_empty() {
                    entry.extend_from_slice(&[0, 0, 0])
                }
            }
        }
    }

    pub fn disable_for_palette(&mut self, palid: usize) {
        if self.has_for_palette(palid) {
            // Remove all entries, this disables ist.
            for entry in self
                .colors
                .iter_mut()
                .skip(palid * DPLA_COLORS_PER_PALETTE)
                .take(DPLA_COLORS_PER_PALETTE)
            {
                entry.clear();
            }
        }
    }

    /// Warning: Colors in-game are animated separately. There is no speed for an entire palette.
    /// We are asuming there's one speed for the entire palette.
    /// This could be inaccurate.
    pub fn get_duration_for_palette(&self, palette_idx: usize) -> PyResult<u16> {
        self.durations_per_frame_for_colors
            .get(palette_idx * DPLA_COLORS_PER_PALETTE)
            .ok_or_else(|| {
                exceptions::PyIndexError::new_err(gettext("Palette index out of range."))
            })
            .copied()
    }

    /// Warning: Colors in-game are animated separately. There is no speed for an entire palette.
    /// We are asuming there's one speed for the entire palette.
    /// This could be inaccurate.
    pub fn set_duration_for_palette(&mut self, palid: usize, duration: u16) {
        for entry in self
            .durations_per_frame_for_colors
            .iter_mut()
            .skip(palid * DPLA_COLORS_PER_PALETTE)
            .take(DPLA_COLORS_PER_PALETTE)
        {
            *entry = duration;
        }
    }

    /// Returns a modified copy of `palettes`.
    ///
    /// This copy is modified to have colors swapped out for the current frame of palette animation.
    /// > The first 16 colors of the DPLA model are placed in the palette 11 (if color 0 has at least one frame).
    /// > The second 16 colors of the DPLA model are placed in the palette 12 (if color 16 has at least one frame).
    /// > If the model has more colors, they are ignored.
    ///
    /// Warning: Colors in-game are animated separately. There is no speed for an entire palette.
    /// We are assuming there's one speed for the entire palette.
    /// This could be inaccurate.
    pub fn apply_palette_animations(
        &self,
        mut palettes: Vec<Vec<u8>>,
        frame_idx: usize,
    ) -> PyResult<Vec<Vec<u8>>> {
        if self.has_for_palette(0) {
            if palettes.len() < 11 {
                return Err(exceptions::PyIndexError::new_err(gettext(
                    "Palette index out of range.",
                )));
            }
            palettes[10] = self.get_palette_for_frame(0, frame_idx)?;
        }
        if self.has_for_palette(1) {
            if palettes.len() < 12 {
                return Err(exceptions::PyIndexError::new_err(gettext(
                    "Palette index out of range.",
                )));
            }
            palettes[11] = self.get_palette_for_frame(1, frame_idx)?;
        }
        Ok(palettes)
    }

    #[cfg(feature = "python")]
    #[pyo3(name = "sir0_serialize_parts")]
    pub fn _sir0_serialize_parts(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.sir0_serialize_parts()?.into_py(py))
    }

    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "sir0_unwrap")]
    pub fn _sir0_unwrap(_cls: &PyType, content_data: StBytes, data_pointer: u32) -> PyResult<Self> {
        Ok(Self::sir0_unwrap(content_data, data_pointer)?)
    }
}

impl Sir0Serializable for Dpla {
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<u32>, Option<u32>)> {
        let mut pointers: Vec<u32> = vec![];
        let mut pointer_offsets: Vec<u32> = vec![];
        let mut written_so_far: u32 = 0;
        let mut data = self
            .colors
            .clone() // we clone so dealing with the null color is easier.
            .into_iter()
            .enumerate()
            .map(|(i, mut color_frames)| {
                pointers.push(written_so_far);
                let number_colors = color_frames.len() / 3;

                // Always one null color
                let null_color = color_frames.is_empty();
                if null_color {
                    color_frames = vec![0, 0, 0]
                }

                let mut buffer = Vec::with_capacity(color_frames.len() * 4 + 4);
                buffer.put_u16_le(u16::try_from(number_colors)?);
                buffer.put_u16_le(*self.durations_per_frame_for_colors.get(i).ok_or_else(
                    || {
                        exceptions::PyIndexError::new_err(gettext(
                            "Frame durations out of range while trying to build color.",
                        ))
                    },
                )?);
                for (i, channel) in color_frames.iter().enumerate() {
                    buffer.put_u8(*channel);
                    if i % 3 == 2 {
                        // Insert the fourth color
                        buffer.put_u8(if null_color { 0 } else { 128 });
                    }
                }

                written_so_far += buffer.len() as u32;
                Ok(buffer)
            })
            .collect::<PyResult<Vec<_>>>()
            .map_err(Sir0Error::SerializeFailedPy)?
            .into_iter()
            .flatten()
            .collect::<BytesMut>();

        let data_offset = data.len() as u32;
        for pnt in pointers {
            pointer_offsets.push(data.len() as u32);
            data.put_u32_le(pnt);
        }

        Ok((StBytes::from(data), pointer_offsets, Some(data_offset)))
    }

    fn sir0_unwrap(content_data: StBytes, data_pointer: u32) -> Sir0Result<Self> {
        Self::new(content_data, data_pointer)
            .map_err(|e| Sir0Error::UnwrapFailed(anyhow::Error::from(e)))
    }
}

#[pyclass(module = "skytemple_rust.st_dpla")]
#[derive(Clone, Default)]
pub struct DplaWriter;

#[pymethods]
impl DplaWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<Dpla>, py: Python) -> PyResult<StBytes> {
        model
            .borrow(py)
            .sir0_serialize_parts()
            .map(|(c, _, _)| c)
            .map_err(|e| exceptions::PyValueError::new_err(format!("{}", e)))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_dpla_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_dpla";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dpla>()?;
    m.add_class::<DplaWriter>()?;

    Ok((name, m))
}
