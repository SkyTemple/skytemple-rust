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

#[cfg_attr(not(feature = "python"), allow(unused_imports))]
#[macro_use]
extern crate skytemple_rust_macros;

#[cfg(not(feature = "python"))]
#[macro_use]
extern crate skytemple_rust_macros_no_py;
extern crate core;

#[macro_use]
pub(crate) mod encoding_utils;
#[macro_use]
pub(crate) mod macros;
#[macro_use]
pub(crate) mod gettext;

pub mod bytes;
pub mod encoding;
pub(crate) mod err;
#[cfg(feature = "image")]
pub mod image;
#[cfg(not(feature = "python"))]
pub mod no_python;
pub mod python;
#[cfg(feature = "python")]
#[cfg(feature = "image")]
mod python_image;
#[cfg(feature = "python")]
mod python_module;
pub mod rom_source;
pub(crate) mod util;

#[cfg(feature = "compression")]
pub mod compression;
#[cfg(feature = "dse")]
pub mod dse;
#[cfg(feature = "compression")]
pub mod st_at3px;
#[cfg(feature = "compression")]
pub mod st_at4pn;
#[cfg(feature = "compression")]
pub mod st_at4px;
#[cfg(feature = "compression")]
pub mod st_at_common;
#[cfg(feature = "compression")]
pub mod st_atupx;
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
#[cfg(feature = "item_p")]
pub mod st_item_p;
#[cfg(feature = "kao")]
pub mod st_kao;
#[cfg(feature = "mappa_bin")]
pub mod st_mappa_bin;
#[cfg(feature = "md")]
pub mod st_md;
#[cfg(feature = "compression")]
pub mod st_pkdpx;
#[cfg(feature = "script_var_table")]
pub mod st_script_var_table;
#[cfg(feature = "sir0")]
pub mod st_sir0;
#[cfg(feature = "strings")]
pub mod st_string;
#[cfg(feature = "waza_p")]
pub mod st_waza_p;

#[cfg(feature = "with_pmd_wan")]
pub mod pmd_wan;
#[cfg(feature = "romfs")]
pub mod romfs;

pub type PyResult<T> = crate::python::PyResult<T>;
