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

extern crate core;

pub(crate) mod util;
#[macro_use] pub(crate) mod encoding_utils;
pub mod bytes;
pub mod python;
pub mod image;
pub mod encoding;
pub mod rom_source;
#[cfg(not(feature = "python"))]
pub mod no_python;
#[cfg(feature = "python")]
mod python_image;
#[cfg(feature = "python")]
mod python_module;

pub mod compression;
pub mod st_at_common;
pub mod st_at3px;
pub mod st_at4pn;
pub mod st_at4px;
pub mod st_atupx;
pub mod st_pkdpx;
pub mod st_kao;
pub mod st_bg_list_dat;
//pub mod st_bgp;
pub mod st_bma;
pub mod st_bpa;
pub mod st_bpc;
pub mod st_bpl;
//pub mod st_dbg;
//pub mod st_dma;
//pub mod st_dpc;
//pub mod st_dpci;
//pub mod st_dpl;
//pub mod st_dpla;
pub mod st_string;

#[cfg(feature = "python")]
pub mod pmd_wan;
pub(crate) mod err;
