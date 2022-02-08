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

#[derive(Clone)]
pub struct SwdlKeygroup {
    pub id: u32,
    pub poly: u32,
    pub priority: u32,
    pub vclow: u32,
    pub vchigh: u32,
    pub unk50: u32,
    pub unk51: u32,
}

#[derive(Clone)]
pub struct SwdlKgrp {
    pub keygroups: Vec<SwdlKeygroup>
}
