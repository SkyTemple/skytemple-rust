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

use crate::python::*;
use pmd_wan as lib;
use pmd_wan::WanError;
use std::convert::TryInto;
use std::io::Cursor;

use crate::bytes::StBytes;
use crate::image::{In16ColIndexedImage, InIndexedImage};

/// A PMD2 WAN sprite.
#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
struct WanImage {
    #[pyo3(get)]
    pub fragment_bytes_store: FragmentBytesStore,
    #[pyo3(get)]
    pub frame_store: FrameStore,
    #[pyo3(get)]
    pub animation_store: AnimationStore,
    #[pyo3(get)]
    pub palette: Palette,
    #[pyo3(get)]
    /// true if the picture have 256 color, false if it only have 16
    pub is_256_color: bool,
    #[pyo3(get)]
    pub sprite_type: SpriteType,
    #[pyo3(get)]
    pub unk2: u16,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
struct FragmentBytesStore {
    #[pyo3(get)]
    pub fragment_bytes: Vec<FragmentBytes>,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct FragmentBytes {
    #[pyo3(get)]
    pub mixed_pixels: Vec<u8>,
    #[pyo3(get)]
    pub z_index: u32,
}

#[pymethods]
impl FragmentBytes {
    pub fn decode_fragment(&self, resolution: &FragmentResolution) -> PyResult<Vec<u8>> {
        lib::decode_fragment_pixels(
            &self.mixed_pixels,
            &lib::FragmentResolution::new(resolution.x, resolution.y),
        )
        .map_err(convert_decode_fragment_bytes_error)
    }

    pub fn to_image(&self, palette: &Palette, fragment: &Fragment) -> PyResult<Vec<u8>> {
        let decoded = self.decode_fragment(&fragment.resolution)?;
        let mut target: Vec<u8> =
            Vec::with_capacity(fragment.resolution.x as usize * fragment.resolution.y as usize);
        for pixel in decoded {
            if pixel == 0 {
                target.extend([0, 0, 0, 0])
            } else {
                let color_id = fragment.pal_idx as usize * 16 + pixel as usize;
                match palette.palette.get(color_id) {
                    Some(v) => {
                        let mut v = *v;
                        v[3] = v[3].saturating_mul(2);
                        target.extend(v)
                    }
                    None => {
                        return Err(exceptions::PyValueError::new_err(format!(
                            "An image reference the non-existing color with the id {}",
                            color_id
                        )))
                    }
                }
            }
        }
        Ok(target)
    }
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct FrameStore {
    #[pyo3(get)]
    pub frames: Vec<Frame>,
    #[pyo3(get)]
    pub max_fragment_alloc_count: u16,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct Fragment {
    #[pyo3(get)]
    pub unk1: u16,
    #[pyo3(get)]
    pub unk3_4: Option<(bool, bool)>,
    #[pyo3(get)]
    pub unk5: bool,
    #[pyo3(get)]
    pub fragment_bytes_index: usize,
    #[pyo3(get)]
    pub offset_y: i8,
    #[pyo3(get)]
    pub offset_x: i16,
    #[pyo3(get)]
    pub flip: FragmentFlip,
    #[pyo3(get)]
    pub is_mosaic: bool,
    #[pyo3(get)]
    pub pal_idx: u16,
    #[pyo3(get)]
    pub resolution: FragmentResolution,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct FragmentFlip {
    #[pyo3(get)]
    pub flip_h: bool,
    #[pyo3(get)]
    pub flip_v: bool,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct Frame {
    #[pyo3(get)]
    pub fragments: Vec<Fragment>,
    #[pyo3(get)]
    pub frame_offset: Option<FrameOffset>,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct FrameOffset {
    #[pyo3(get)]
    head: (i16, i16),
    #[pyo3(get)]
    hand_left: (i16, i16),
    #[pyo3(get)]
    hand_right: (i16, i16),
    #[pyo3(get)]
    center: (i16, i16),
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Copy, Clone)]
pub struct FragmentResolution {
    #[pyo3(get)]
    pub x: u8,
    #[pyo3(get)]
    pub y: u8,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct AnimationStore {
    #[pyo3(get)]
    pub copied_on_previous: Option<Vec<bool>>, //indicate if a sprite can copy on the previous. Will always copy if possible if None
    #[pyo3(get)]
    pub anim_groups: Vec<Vec<Animation>>, //usize1 = start, usize2 = length
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(PartialEq, Eq, Clone)]
pub struct Animation {
    #[pyo3(get)]
    pub frames: Vec<AnimationFrame>,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(PartialEq, Eq, Clone)]
pub struct AnimationFrame {
    #[pyo3(get)]
    pub duration: u8,
    #[pyo3(get)]
    pub flag: u8,
    #[pyo3(get)]
    pub frame_id: u16,
    #[pyo3(get)]
    pub offset_x: i16,
    #[pyo3(get)]
    pub offset_y: i16,
    #[pyo3(get)]
    pub shadow_offset_x: i16,
    #[pyo3(get)]
    pub shadow_offset_y: i16,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct Palette {
    #[pyo3(get)]
    pub palette: Vec<[u8; 4]>,
}

#[pyclass(module = "skytemple_rust.pmd_wan")]
#[derive(Clone)]
pub struct SpriteType {
    #[pyo3(get)]
    name: &'static str,
    #[pyo3(get)]
    value: u8,
}

#[allow(non_upper_case_globals)]
#[pymethods]
impl SpriteType {
    #[classattr]
    const PropsUI: SpriteType = SpriteType {
        name: "PropsUI",
        value: 0,
    };

    #[classattr]
    const Chara: SpriteType = SpriteType {
        name: "Chara",
        value: 1,
    };

    #[classattr]
    const Unknown: SpriteType = SpriteType {
        name: "Unknown",
        value: 3,
    };

    #[new]
    fn new(value: u8) -> PyResult<Self> {
        match value {
            0 => Ok(SpriteType::PropsUI),
            1 => Ok(SpriteType::Chara),
            3 => Ok(SpriteType::Unknown),
            _ => Err(convert_error(WanError::TypeOfSpriteUnknown(value as u16))),
        }
    }

    pub fn __int__(&self) -> PyResult<u8> {
        Ok(self.value)
    }

    pub fn __str__(&self) -> PyResult<&'static str> {
        Ok(self.name)
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("SpriteType({})", self.value))
    }
}

fn wrap_vec<T, U>(vector: &[T], convert_cb: fn(&T) -> U) -> Vec<U> {
    vector.iter().map(convert_cb).collect()
}

fn wrap_fragment_bytes_store(lib_ent: &lib::FragmentBytesStore) -> FragmentBytesStore {
    FragmentBytesStore {
        fragment_bytes: wrap_vec(&lib_ent.fragment_bytes, wrap_fragment_bytes),
    }
}

fn wrap_fragment_bytes(lib_ent: &lib::FragmentBytes) -> FragmentBytes {
    FragmentBytes {
        mixed_pixels: lib_ent.mixed_pixels.clone(),
        z_index: lib_ent.z_index,
    }
}

fn wrap_frame_store(lib_ent: &lib::FrameStore) -> FrameStore {
    FrameStore {
        frames: wrap_vec(&lib_ent.frames, wrap_frame),
        max_fragment_alloc_count: lib_ent.compute_fragment_alloc_counter(),
    }
}

fn wrap_fragment(lib_ent: &lib::Fragment) -> Fragment {
    Fragment {
        unk1: lib_ent.unk1,
        unk3_4: lib_ent.unk3_4,
        unk5: lib_ent.unk5,
        fragment_bytes_index: lib_ent.fragment_bytes_index,
        offset_y: lib_ent.offset_y,
        offset_x: lib_ent.offset_x,
        flip: wrap_fragment_flip(lib_ent.flip),
        is_mosaic: lib_ent.is_mosaic,
        pal_idx: lib_ent.pal_idx,
        resolution: wrap_fragment_resolution(&lib_ent.resolution),
    }
}

fn wrap_fragment_flip(lib_ent: lib::FragmentFlip) -> FragmentFlip {
    FragmentFlip {
        flip_h: lib_ent.flip_h,
        flip_v: lib_ent.flip_v,
    }
}

fn wrap_frame(lib_ent: &lib::Frame) -> Frame {
    Frame {
        fragments: wrap_vec(&lib_ent.fragments, wrap_fragment),
        frame_offset: lib_ent.frame_offset.as_ref().map(wrap_frame_offset),
    }
}

fn wrap_frame_offset(lib_ent: &lib::FrameOffset) -> FrameOffset {
    FrameOffset {
        head: lib_ent.head,
        hand_left: lib_ent.hand_left,
        hand_right: lib_ent.hand_right,
        center: lib_ent.center,
    }
}

fn wrap_fragment_resolution(lib_ent: &lib::FragmentResolution) -> FragmentResolution {
    FragmentResolution {
        x: lib_ent.x,
        y: lib_ent.y,
    }
}

fn wrap_animation_store(lib_ent: &lib::AnimationStore) -> AnimationStore {
    AnimationStore {
        anim_groups: wrap_vec(&lib_ent.anim_groups, |x| wrap_vec(x, wrap_animation)),
        copied_on_previous: lib_ent.copied_on_previous.clone(),
    }
}

fn wrap_animation(lib_ent: &lib::Animation) -> Animation {
    Animation {
        frames: wrap_vec(&lib_ent.frames, wrap_animation_frame),
    }
}

fn wrap_animation_frame(lib_ent: &lib::AnimationFrame) -> AnimationFrame {
    AnimationFrame {
        duration: lib_ent.duration,
        flag: lib_ent.flag,
        frame_id: lib_ent.frame_id,
        offset_x: lib_ent.offset_x,
        offset_y: lib_ent.offset_y,
        shadow_offset_x: lib_ent.shadow_offset_x,
        shadow_offset_y: lib_ent.shadow_offset_y,
    }
}

fn wrap_palette(lib_ent: &lib::Palette) -> Palette {
    Palette {
        palette: lib_ent.palette.clone(),
    }
}

fn convert_sprite_type(lib_ent: &lib::SpriteType) -> SpriteType {
    match lib_ent {
        lib::SpriteType::PropsUI => SpriteType::PropsUI,
        lib::SpriteType::Chara => SpriteType::Chara,
        lib::SpriteType::Unknown => SpriteType::Unknown,
    }
}

#[pymethods]
impl WanImage {
    #[new]
    fn new(data: Vec<u8>) -> PyResult<Self> {
        let result = lib::WanImage::decode_wan(Cursor::new(data));
        match result {
            Ok(lib_img) => Ok(WanImage {
                fragment_bytes_store: wrap_fragment_bytes_store(&lib_img.fragment_bytes_store),
                frame_store: wrap_frame_store(&lib_img.frame_store),
                animation_store: wrap_animation_store(&lib_img.animation_store),
                palette: wrap_palette(&lib_img.palette),
                is_256_color: lib_img.is_256_color,
                sprite_type: convert_sprite_type(&lib_img.sprite_type),
                unk2: lib_img.unk2,
            }),
            Err(err) => Err(convert_error(err)),
        }
    }
}

fn convert_error(err: lib::WanError) -> PyErr {
    match err {
        WanError::IOError(_) => exceptions::PyIOError::new_err("an io error happened"),
        err => exceptions::PyValueError::new_err(format!("{}", err)),
    }
}

fn convert_decode_fragment_bytes_error(err: lib::DecodeFragmentBytesError) -> PyErr {
    exceptions::PyValueError::new_err(format!("{}", err))
}

fn convert_anyhow_error(err: anyhow::Error) -> PyErr {
    exceptions::PyValueError::new_err(format!("{:?}", err))
}

fn convert_anyhow_error_to_user(err: anyhow::Error) -> PyErr {
    create_value_user_error(format!("{:?}", err))
}

#[pyfunction]
pub fn encode_image_to_static_wan_file(py: Python, image: PyObject) -> PyResult<StBytes> {
    let indexed_image = In16ColIndexedImage(image).extract(py)?;
    let mut palette = Vec::new();

    // input palette always have 255 colours
    for color in indexed_image.1.chunks_exact(3).take(16) {
        //no panic: color is always of length 3
        palette.push([color[0], color[1], color[2], 128]);
    }

    let mut wanimage = lib::WanImage::new(lib::SpriteType::PropsUI);
    wanimage.palette.palette = palette;

    let frame_id = lib::insert_frame_in_wanimage(
        indexed_image.0 .0 .0.to_vec(),
        indexed_image
            .0
             .1
            .try_into()
            .map_err(|_| create_value_user_error("The image is far too wide"))?,
        indexed_image
            .0
             .2
            .try_into()
            .map_err(|_| create_value_user_error("The image is far too high"))?,
        &mut wanimage,
        0,
    )
    .map_err(convert_anyhow_error_to_user)?;

    if let Some(frame_id) = frame_id {
        wanimage
            .animation_store
            .anim_groups
            .push(vec![lib::Animation {
                frames: vec![lib::AnimationFrame {
                    duration: 1,
                    flag: 0,
                    frame_id: frame_id as u16,
                    offset_x: 0,
                    offset_y: 0,
                    shadow_offset_x: 0,
                    shadow_offset_y: 0,
                }],
            }]);

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        wanimage
            .create_wan(&mut cursor)
            .map_err(convert_anyhow_error)?;
        Ok(StBytes::from(buffer))
    } else {
        Err(create_value_user_error(
            "The image doesn't contain any visible pixel",
        ))
    }
}

pub(crate) fn create_pmd_wan_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.pmd_wan";
    let m = PyModule::new(py, name)?;
    m.add_class::<WanImage>()?;
    m.add_class::<FragmentBytesStore>()?;
    m.add_class::<FragmentBytes>()?;
    m.add_class::<FrameStore>()?;
    m.add_class::<Fragment>()?;
    m.add_class::<FragmentFlip>()?;
    m.add_class::<Frame>()?;
    m.add_class::<FrameOffset>()?;
    m.add_class::<FragmentResolution>()?;
    m.add_class::<AnimationStore>()?;
    m.add_class::<Animation>()?;
    m.add_class::<AnimationFrame>()?;
    m.add_class::<Palette>()?;
    m.add_class::<SpriteType>()?;
    m.add_function(wrap_pyfunction!(encode_image_to_static_wan_file, m)?)?;

    Ok((name, m))
}
