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
    #[pyo3(get)]
    pub unk2: u16
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
struct ImageStore {
    #[pyo3(get)]
    pub images: Vec<ImageBytes>,
}

#[pyclass(module = "pmd_wan")]
#[derive(Clone)]
pub struct ImageBytes {
    #[pyo3(get)]
    pub mixed_pixels: Vec<u8>,
    #[pyo3(get)]
    pub z_index: u32,
}

#[pymethods]
impl ImageBytes {
    pub fn decode_image(&self, resolution: &Resolution) -> PyResult<Vec<u8>> {
        lib::decode_image_pixel(&self.mixed_pixels, &lib::Resolution {
            x: resolution.x,
            y: resolution.y
        }).map_err(|err| convert_decode_image_error(err))
    }

    pub fn to_image(&self, palette: &Palette, metaframe: &MetaFrame) -> PyResult<Vec<u8>> {
        let decoded = self.decode_image(&metaframe.resolution)?;
        let mut target: Vec<u8> = Vec::with_capacity(metaframe.resolution.x as usize* metaframe.resolution.y as usize);
        for pixel in decoded {
            if pixel == 0 {
                target.extend(&[0, 0, 0, 0])
            } else {
                let color_id = metaframe.pal_idx as usize * 16 + pixel as usize;
                match palette.palette.get(color_id) {
                    Some(v) => {
                        let mut v = v.clone();
                        v[3] = v[3].saturating_mul(2);
                        target.extend(v)
                    },
                    None => return Err(exceptions::PyValueError::new_err(
                        format!("An image reference the non-existing color with the id {}", color_id)
                    )),
                }
            }
        }
        Ok(target)
    }
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
    pub offset_y: i8,
    #[pyo3(get)]
    pub offset_x: i16,
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
    pub resolution: Resolution,
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
    pub copied_on_previous: Option<Vec<bool>>, //indicate if a sprite can copy on the previous. Will always copy if possible if None
    #[pyo3(get)]
    pub anim_groups: Vec<Vec<Animation>>, //usize1 = start, usize2 = length
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
    pub palette: Vec<[u8; 4]>,
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

fn wrap_image(lib_ent: &lib::ImageBytes) -> ImageBytes {
    ImageBytes {
        mixed_pixels: lib_ent.mixed_pixels.clone(),
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
        resolution: wrap_resolution(&lib_ent.resolution),
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
        anim_groups: wrap_vec(
            &lib_ent.anim_groups,
            |x| wrap_vec(x, |y| wrap_animation(y))
        ),
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
                unk2: lib_img.unk2
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
fn convert_decode_image_error(err: lib::DecodeImageError) -> PyErr {
    exceptions::PyValueError::new_err(
        format!("{}", err)
    )
}

pub(crate) fn create_pmd_wan_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.pmd_wan";
    let m = PyModule::new(py, name)?;
    m.add_class::<WanImage>()?;
    m.add_class::<ImageStore>()?;
    m.add_class::<ImageBytes>()?;
    m.add_class::<MetaFrameStore>()?;
    m.add_class::<MetaFrame>()?;
    m.add_class::<MetaFrameGroup>()?;
    m.add_class::<Resolution>()?;
    m.add_class::<AnimStore>()?;
    m.add_class::<Animation>()?;
    m.add_class::<AnimationFrame>()?;
    m.add_class::<Palette>()?;
    m.add_class::<SpriteType>()?;

    Ok((name, m))
}
