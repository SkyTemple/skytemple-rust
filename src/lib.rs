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

pub(crate) mod util;
pub mod python;
pub mod image;
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

#[cfg(feature = "python")]
pub mod pmd_wan;
