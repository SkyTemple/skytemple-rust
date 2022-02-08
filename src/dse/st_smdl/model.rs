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

use crate::bytes::StBytes;
use crate::dse::date::DseDate;
use crate::dse::filename::DseFilename;

#[derive(Clone)]
pub struct SmdlHeader {
    pub version: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub modified_date: DseDate,
    pub file_name: DseFilename,
    pub unk5: u32,
    pub unk6: u32,
    pub unk8: u32,
    pub unk9: u32,
}

#[derive(Clone)]
pub struct SmdlSong {
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub tpqn: u32,
    pub unk5: u32,
    pub nbchans: u32,
    pub unk6: u32,
    pub unk7: u32,
    pub unk8: u32,
    pub unk9: u32,
    pub unk10: u32,
    pub unk11: u32,
    pub unk12: u32,
}

#[derive(Clone)]
pub struct SmdlEoc {
    pub param1: u32,
    pub param2: u32,
}

#[derive(Clone)]
pub struct SmdlTrackHeader {
    pub param1: u32,
    pub param2: u32,
}

#[derive(Clone)]
pub struct SmdlTrackPreamble {
    pub track_id: u32,
    pub channel_id: u32,
    pub unk1: u32,
    pub unk2: u32,
}

#[derive(Clone)]
pub struct SmdlEventPlayNote {
    pub velocity: u32,
    pub octave_mod: u32,
    pub note: u32,
    pub key_down_duration: Option<u32>,
}

#[derive(Clone)]
pub struct SmdlEventPause {
    pub value: u32,
}

#[derive(Clone)]
pub struct SmdlEventSpecial {
    pub op: u32,
    pub params: Vec<u32>,
}

#[derive(Clone)]
pub enum SmdlEvent {
    Special(SmdlEventSpecial),
    Pause(SmdlEventPause),
    Note(SmdlEventPlayNote),
}

#[derive(Clone)]
pub struct SmdlTrack {
    pub header: SmdlTrackHeader,
    pub preamble: SmdlTrackPreamble,
    pub events: Vec<SmdlEvent>,
}

#[derive(Clone)]
pub struct Smdl {
    pub header: SmdlHeader,
    pub song: SmdlSong,
    pub tracks: Vec<SmdlTrack>,
    pub eoc: SmdlEoc
}

impl From<StBytes> for Smdl {
    fn from(source: StBytes) -> Self {
        todo!()
    }
}

impl From<Smdl> for StBytes {
    fn from(source: Smdl) -> Self {
        todo!()
    }
}
