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

use crate::dse::st_swdl::pcmd::SwdlPcmd;

#[derive(Clone)]
pub struct SwdlPcmdReference {
    pub pcmd: SwdlPcmd,
    pub offset: u32,
    pub length: u32
}

#[derive(Clone)]
pub struct SwdlSampleInfoTblEntry {
    pub id: u32,
    pub ftune: u32,
    pub ctune: u32,
    pub rootkey: u32,
    pub ktps: u32,
    pub volume: u32,
    pub pan: u32,
    pub unk5: u32,
    pub unk58: u32,
    pub sample_format: u32,
    pub unk9: u32,
    pub unk10: u32,
    pub unk11: u32,
    pub unk12: u32,
    pub unk13: u32,
    pub sample_rate: u32,
    pub sample: Option<SwdlPcmdReference>,
    pub loop_begin_pos: u32,
    pub loop_length: u32,
    pub envelope: u32,
    pub envelope_multiplier: u32,
    pub unk19: u32,
    pub unk20: u32,
    pub unk21: u32,
    pub unk22: u32,
    pub attack_volume: u32,
    pub attack: u32,
    pub decay: u32,
    pub sustain: u32,
    pub hold: u32,
    pub decay2: u32,
    pub release: u32,
    pub unk57: u32
}

#[derive(Clone)]
pub struct SwdlWavi {
    pub sample_info_table: Vec<Option<SwdlSampleInfoTblEntry>>
}
