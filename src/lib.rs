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

// We allow this, because it currently incorrectly flags code expanded by the gettextrs crate.
#![allow(clippy::format_push_string)]

#[macro_use] pub(crate) mod encoding_utils;
#[macro_use] pub(crate) mod macros;
#[macro_use] pub(crate) mod gettext;

pub(crate) mod util;
pub(crate) mod err;
pub mod bytes;
pub mod python;
#[cfg(feature = "image")]
pub mod image;
pub mod encoding;
pub mod rom_source;
#[cfg(not(feature = "python"))]
pub mod no_python;
#[cfg(feature = "python")]
#[cfg(feature = "image")]
mod python_image;
#[cfg(feature = "python")]
mod python_module;

#[cfg(feature = "compression")]
pub mod compression;
#[cfg(feature = "compression")]
pub mod st_at_common;
#[cfg(feature = "compression")]
pub mod st_at3px;
#[cfg(feature = "compression")]
pub mod st_at4pn;
#[cfg(feature = "compression")]
pub mod st_at4px;
#[cfg(feature = "compression")]
pub mod st_atupx;
#[cfg(feature = "compression")]
pub mod st_pkdpx;
#[cfg(feature = "kao")]
pub mod st_kao;
#[cfg(feature = "map_bg")]
pub mod st_bg_list_dat;
#[cfg(feature = "misc_graphics")]
pub mod st_bgp;
#[cfg(feature = "map_bg")]
pub mod st_bma;
#[cfg(feature = "map_bg")]
pub mod st_bpa;
#[cfg(feature = "map_bg")]
pub mod st_bpc;
#[cfg(feature = "map_bg")]
pub mod st_bpl;
#[cfg(feature = "dungeon_graphics")]
pub mod st_dbg;
#[cfg(feature = "dungeon_graphics")]
pub mod st_dma;
#[cfg(feature = "dungeon_graphics")]
pub mod st_dpc;
#[cfg(feature = "dungeon_graphics")]
pub mod st_dpci;
#[cfg(feature = "dungeon_graphics")]
pub mod st_dpl;
#[cfg(feature = "dungeon_graphics")]
pub mod st_dpla;
#[cfg(feature = "strings")]
pub mod st_string;
#[cfg(feature = "dse")]
pub mod dse;

#[cfg(feature = "with_pmd_wan")]
pub mod pmd_wan;
#[cfg(feature = "romfs")]
pub mod romfs;

pub type PyResult<T> = crate::python::PyResult<T>;
