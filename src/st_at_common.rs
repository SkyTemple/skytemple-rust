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
use crate::bytes::StBytesMut;
use crate::python::*;
use crate::st_at3px::At3px;
use crate::st_at4pn::At4pn;
use crate::st_at4px::At4px;
use crate::st_atupx::Atupx;
use crate::st_pkdpx::Pkdpx;
use bytes::Buf;
use std::env;

pub enum CommonAtType {
    AT4PN = 0,
    AT3PX = 1,
    AT4PX = 2,
    ATUPX = 3,
    PKDPX = 4,
}

// Pre-built lists for compression
pub const COMMON_AT_BEST_3: [CommonAtType; 4] = [
    CommonAtType::AT4PN,
    CommonAtType::ATUPX,
    CommonAtType::AT3PX,
    CommonAtType::PKDPX,
];
pub const COMMON_AT_BEST_4: [CommonAtType; 4] = [
    CommonAtType::AT4PN,
    CommonAtType::ATUPX,
    CommonAtType::AT4PX,
    CommonAtType::PKDPX,
];
pub const COMMON_AT_MUST_COMPRESS_3: [CommonAtType; 3] = [
    CommonAtType::ATUPX,
    CommonAtType::AT3PX,
    CommonAtType::PKDPX,
];
pub const COMMON_AT_MUST_COMPRESS_4: [CommonAtType; 3] = [
    CommonAtType::ATUPX,
    CommonAtType::AT4PX,
    CommonAtType::PKDPX,
];
pub const COMMON_AT_PKD: [CommonAtType; 1] = [CommonAtType::PKDPX];

pub trait CompressionContainer {
    fn cont_size<F: Buf>(mut data: F, byte_offset: usize) -> u16 {
        data.advance(byte_offset + 5);
        data.get_u16_le()
    }
}

pub struct CommonAt {}

struct CommonAtCompressor(Option<StBytesMut>, i32);

impl CommonAt {
    pub fn cont_size(data: &[u8], byte_offset: usize) -> Option<u16> {
        let mut data = &data[byte_offset..];
        if At4pn::matches(data) {
            return Some(At4pn::cont_size(&mut data, 0));
        }
        if At4px::matches(data) {
            return Some(At4px::cont_size(&mut data, 0));
        }
        if At3px::matches(data) {
            return Some(At3px::cont_size(&mut data, 0));
        }
        if Pkdpx::matches(data) {
            return Some(Pkdpx::cont_size(&mut data, 0));
        }
        if Atupx::matches(data) {
            return Some(Atupx::cont_size(&mut data, 0));
        }
        None
    }

    /// Compress with all of the compression algorithms yielded by `compression_type`. Returns
    /// the fastest. Additionally ATUPX is only checked if the environment variable `SKYTEMPLE_ALLOW_ATUPX` is set.
    pub fn compress<'a, I>(uncompressed_data: &[u8], compression_type: I) -> PyResult<StBytesMut>
    where
        I: Iterator<Item = &'a CommonAtType>,
    {
        let mut meta = CommonAtCompressor(None, -1);
        for t in compression_type {
            match t {
                CommonAtType::AT4PN => Self::compress_try(
                    At4pn::compress(uncompressed_data).map(|x| x.to_bytes()),
                    &mut meta,
                ),
                CommonAtType::AT3PX => Self::compress_try(
                    At3px::compress(uncompressed_data).map(|x| x.to_bytes()),
                    &mut meta,
                ),
                CommonAtType::AT4PX => Self::compress_try(
                    At4px::compress(uncompressed_data).map(|x| x.to_bytes()),
                    &mut meta,
                ),
                CommonAtType::ATUPX => {
                    if env::var("SKYTEMPLE_ALLOW_ATUPX").is_ok() {
                        Self::compress_try(
                            Atupx::compress(uncompressed_data).map(|x| x.to_bytes()),
                            &mut meta,
                        );
                    }
                }
                CommonAtType::PKDPX => Self::compress_try(
                    Pkdpx::compress(uncompressed_data).map(|x| x.to_bytes()),
                    &mut meta,
                ),
            }
        }
        match meta.0 {
            Some(x) => Ok(x),
            None => Err(exceptions::PyValueError::new_err(
                "No usable compression algorithm.",
            )),
        }
    }
    fn compress_try(in_bytes: PyResult<StBytesMut>, meta: &mut CommonAtCompressor) {
        if let Ok(in_bytes) = in_bytes {
            if meta.0.is_none() || in_bytes.0.len() < meta.1 as usize {
                meta.1 = in_bytes.0.len() as i32;
                meta.0 = Some(in_bytes);
            }
        }
    }

    pub fn decompress(compressed_data: &[u8]) -> PyResult<StBytesMut> {
        if At4pn::matches(compressed_data) {
            return At4pn::new(compressed_data, false)?.decompress();
        }
        if At4px::matches(compressed_data) {
            return At4px::new(compressed_data)?.decompress();
        }
        if At3px::matches(compressed_data) {
            return At3px::new(compressed_data)?.decompress();
        }
        if Pkdpx::matches(compressed_data) {
            return Pkdpx::new(compressed_data)?.decompress();
        }
        if Atupx::matches(compressed_data) {
            return Atupx::new(compressed_data)?.decompress();
        }
        Err(exceptions::PyValueError::new_err(
            "Unknown compression container",
        ))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_at_common_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_at_common";
    let m = PyModule::new(py, name)?;

    Ok((name, m))
}
