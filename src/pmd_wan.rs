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

use pyo3::prelude::*;
use pmd_wan as lib;
use std::io::Cursor;
use pmd_wan::WanError;
use pyo3::exceptions;

/// A PMD2 WAN sprite.
#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
struct WanImage {
    #[pyo3(get)]
    pub image_store: ImageStore,
    #[pyo3(get)]
    pub meta_frame_store: MetaFrameStore,
    #[pyo3(get)]
    pub anim_store: AnimStore,
    #[pyo3(get)]
    pub palette: Palette,
    #[pyo3(get)]
    pub raw_particule_table: Vec<u8>,
    #[pyo3(get)]
    /// true if the picture have 256 color, false if it only have 16
    pub is_256_color: bool,
    #[pyo3(get)]
    pub sprite_type: SpriteType,
    #[pyo3(get)]
    pub unk_1: u32,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
struct ImageStore {
    #[pyo3(get)]
    pub images: Vec<Image>,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct Image {
    #[pyo3(get)]
    pub img: Vec<u8>,
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    #[pyo3(get)]
    pub z_index: u32,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct MetaFrameStore {
    #[pyo3(get)]
    pub meta_frames: Vec<MetaFrame>,
    #[pyo3(get)]
    pub meta_frame_groups: Vec<MetaFrameGroup>,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct MetaFrame {
    #[pyo3(get)]
    pub unk1: u16,
    #[pyo3(get)]
    pub unk2: u16,
    #[pyo3(get)]
    pub unk3: bool,
    #[pyo3(get)]
    pub image_index: usize,
    #[pyo3(get)]
    pub offset_y: i32,
    #[pyo3(get)]
    pub offset_x: i32,
    #[pyo3(get)]
    pub is_last: bool,
    #[pyo3(get)]
    pub v_flip: bool,
    #[pyo3(get)]
    pub h_flip: bool,
    #[pyo3(get)]
    pub is_mosaic: bool,
    #[pyo3(get)]
    pub pal_idx: u16,
    #[pyo3(get)]
    pub resolution: Option<Resolution>,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct MetaFrameGroup {
    #[pyo3(get)]
    pub meta_frames_id: Vec<usize>,
}

#[pyclass(module = "pmd_wan")]
#[derive(Copy, Clone)]
pub struct Resolution {
    #[pyo3(get)]
    pub x: u8,
    #[pyo3(get)]
    pub y: u8,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct AnimStore {
    #[pyo3(get)]
    pub animations: Vec<Animation>,
    #[pyo3(get)]
    pub copied_on_previous: Option<Vec<bool>>, //indicate if a sprite can copy on the previous. Will always copy if possible if None
    #[pyo3(get)]
    pub anim_groups: Vec<Option<(usize, usize)>>, //usize1 = start, usize2 = length
}

#[pyclass(module = "pmd_wan")]
#[derive(PartialEq, Clone)]
pub struct Animation {
    #[pyo3(get)]
    pub frames: Vec<AnimationFrame>,
}

#[pyclass(module = "pmd_wan")]
#[derive(PartialEq, Clone)]
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

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct Palette {
    #[pyo3(get)]
    pub palette: Vec<(u8, u8, u8, u8)>,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct SpriteType {
    #[pyo3(get)]
    name: &'static str,
    #[pyo3(get)]
    value: u8
}

#[allow(non_upper_case_globals)]
#[pymethods]
impl SpriteType {
    #[classattr]
    const PropsUI: SpriteType = SpriteType { name: "PropsUI", value: 0 };

    #[classattr]
    const Chara: SpriteType = SpriteType { name: "Chara", value: 1 };

    #[classattr]
    const Unknown: SpriteType = SpriteType { name: "Unknown", value: 3 };

    #[new]
    fn new(value: u8) -> PyResult<Self> {
        match value {
            0 => Ok(SpriteType::PropsUI),
            1 => Ok(SpriteType::Chara),
            3 => Ok(SpriteType::Unknown),
            _ => Err(convert_error(WanError::TypeOfSpriteUnknown(value as u16)))
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

fn wrap_vec<T, U>(vector: &[T], convert_cb: fn (&T) -> U) -> Vec<U> {
    vector.iter().map(convert_cb).collect()
}

fn wrap_image_store(lib_ent: &lib::ImageStore) -> ImageStore {
    ImageStore {
        images: wrap_vec(&lib_ent.images, |img| { wrap_image(img) })
    }
}

fn wrap_image(lib_ent: &lib::Image) -> Image {
    Image {
        img: lib_ent.img.to_vec(),
        width: lib_ent.img.width(),
        height: lib_ent.img.height(),
        z_index: lib_ent.z_index
    }
}

fn wrap_meta_frame_store(lib_ent: &lib::MetaFrameStore) -> MetaFrameStore {
    MetaFrameStore {
        meta_frames: wrap_vec(&lib_ent.meta_frames, |x| { wrap_meta_frame(x) }),
        meta_frame_groups: wrap_vec(&lib_ent.meta_frame_groups, |x| { wrap_meta_frame_group(x) }),
    }
}

fn wrap_meta_frame(lib_ent: &lib::MetaFrame) -> MetaFrame {
    let resolution;
    if let Some(v) = lib_ent.resolution {
        resolution = Option::Some(wrap_resolution(&v));
    } else {
        resolution = Option::None;
    }
    MetaFrame {
        unk1: lib_ent.unk1,
        unk2: lib_ent.unk2,
        unk3: lib_ent.unk3,
        image_index: lib_ent.image_index,
        offset_y: lib_ent.offset_y,
        offset_x: lib_ent.offset_x,
        is_last: lib_ent.is_last,
        v_flip: lib_ent.v_flip,
        h_flip: lib_ent.h_flip,
        is_mosaic: lib_ent.is_mosaic,
        pal_idx: lib_ent.pal_idx,
        resolution,
    }
}

fn wrap_meta_frame_group(lib_ent: &lib::MetaFrameGroup) -> MetaFrameGroup {
    MetaFrameGroup {
        meta_frames_id: lib_ent.meta_frames_id.clone()
    }
}

fn wrap_resolution(lib_ent: &lib::Resolution<u8>) -> Resolution {
    Resolution {
        x: lib_ent.x,
        y: lib_ent.y
    }
}

fn wrap_anim_store(lib_ent: &lib::AnimStore) -> AnimStore {
    AnimStore {
        animations: wrap_vec(&lib_ent.animations, |x| { wrap_animation(x) }),
        anim_groups: lib_ent.anim_groups.clone(),
        copied_on_previous: lib_ent.copied_on_previous.clone()
    }
}

fn wrap_animation(lib_ent: &lib::Animation) -> Animation {
    Animation {
        frames: wrap_vec(&lib_ent.frames, |x| { wrap_animation_frame(x) })
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
        palette: lib_ent.palette.clone()
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
                image_store: wrap_image_store(&lib_img.image_store),
                meta_frame_store: wrap_meta_frame_store(&lib_img.meta_frame_store),
                anim_store: wrap_anim_store(&lib_img.anim_store),
                palette: wrap_palette(&lib_img.palette),
                raw_particule_table: lib_img.raw_particule_table,
                is_256_color: lib_img.is_256_color,
                sprite_type: convert_sprite_type(&lib_img.sprite_type),
                unk_1: lib_img.unk_1,
            }),
            Err(err) => Err(convert_error(err))
        }

    }
}

fn convert_error(err: lib::WanError) -> PyErr {
    match err {
        WanError::IOError(_) => 
            exceptions::PyIOError::new_err(
                "an io error happened"
            ),
        err =>
            exceptions::PyValueError::new_err(
                format!("{}", err)
            ),
    }
}

#[pymodule]
fn pmd_wan(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<WanImage>()?;
    m.add_class::<ImageStore>()?;
    m.add_class::<Image>()?;
    m.add_class::<MetaFrameStore>()?;
    m.add_class::<MetaFrame>()?;
    m.add_class::<MetaFrameGroup>()?;
    m.add_class::<Resolution>()?;
    m.add_class::<AnimStore>()?;
    m.add_class::<Animation>()?;
    m.add_class::<AnimationFrame>()?;
    m.add_class::<Palette>()?;
    m.add_class::<SpriteType>()?;

    Ok(())
}
