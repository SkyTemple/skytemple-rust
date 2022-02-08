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
pub struct SwdlLfoEntry {
    pub unk34: u32,
    pub unk52: u32,
    pub dest: u32,
    pub wshape: u32,
    pub rate: u32,
    pub unk29: u32,
    pub depth: u32,
    pub delay: u32,
    pub unk32: u32,
    pub unk33: u32,
}

#[derive(Clone)]
pub struct SwdlSplitEntry {
    pub id: u32,
    pub unk11: u32,
    pub unk25: u32,
    pub lowkey: u32,
    pub hikey: u32,
    pub lolevel: u32,
    pub hilevel: u32,
    pub unk16: u32,
    pub unk17: u32,
    pub sample_id: u32,
    pub ftune: u32,
    pub ctune: u32,
    pub rootkey: u32,
    pub ktps: u32,
    pub sample_volume: u32,
    pub sample_pan: u32,
    pub keygroup_id: u32,
    pub unk22: u32,
    pub unk23: u32,
    pub unk24: u32,
    pub envelope: u32,
    pub envelope_multiplier: u32,
    pub unk37: u32,
    pub unk38: u32,
    pub unk39: u32,
    pub unk40: u32,
    pub attack_volume: u32,
    pub attack: u32,
    pub decay: u32,
    pub sustain: u32,
    pub hold: u32,
    pub decay2: u32,
    pub release: u32,
    pub unk53: u32,
}

#[derive(Clone)]
pub struct SwdlProgramTable {
    pub id: u32,
    pub prg_volume: u32,
    pub prg_pan: u32,
    pub unk3: u32,
    pub that_f_byte: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk7: u32,
    pub unk8: u32,
    pub unk9: u32,
    pub lfos: Vec<SwdlLfoEntry>,
    pub splits: Vec<SwdlSplitEntry>,
}

#[derive(Clone)]
pub struct SwdlPrgi {
    pub program_table: Vec<Option<SwdlProgramTable>>
}
