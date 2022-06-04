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

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub const PLAY_NOTE_MAX: u8 = 0x7F;
pub const PAUSE_NOTE_MAX: u8 = 0x8F;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, FromPrimitive, Debug)]
pub enum SmdlNote {
    C = 0x0,
    CS = 0x1,
    D = 0x2,
    DS = 0x3,
    E = 0x4,
    F = 0x5,
    FS = 0x6,
    G = 0x7,
    GS = 0x8,
    A = 0x9,
    AS = 0xA,
    B = 0xB,
    InvalidC = 0xC,
    InvalidD = 0xD,
    InvalidE = 0xE,
    Unknown = 0xF,
}

impl SmdlNote {
    pub fn valid(&self) -> bool {
        self != &Self::InvalidC
            && self != &Self::InvalidD
            && self != &Self::InvalidE
            && self != &Self::Unknown
    }
}

impl From<u8> for SmdlNote {
    fn from(v: u8) -> Self {
        Self::from_u8(v).expect("Only numbers from 0-15 can be converted to SmdlNote.")
    }
}

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, PartialOrd, FromPrimitive, Debug)]
pub enum SmdlPause {
    HalfNote = 0x80,
    DottedQuarterNote = 0x81,
    TwoThirdsOfHalfNote = 0x82,
    QuarterNote = 0x83,
    DottedEightNote = 0x84,
    TwoThirdsOfQuarterNote = 0x85,
    EightNote = 0x86,
    DottedSixteenthNote = 0x87,
    TwoThirdsOfEightNote = 0x88,
    SixteenthNote = 0x89,
    DottedThirtysecondNote = 0x8A,
    TwoThirdsOfSixteenthNote = 0x8B,
    ThirtysecondNote = 0x8C,
    DottedSixtyforthNote = 0x8D,
    TwoThirdsOfThirtysecondNote = 0x8E,
    SixtyforthNote = 0x8F,
}

impl SmdlPause {
    pub fn length(&self) -> usize {
        match self {
            SmdlPause::HalfNote => 96,
            SmdlPause::DottedQuarterNote => 72,
            SmdlPause::TwoThirdsOfHalfNote => 64,
            SmdlPause::QuarterNote => 48,
            SmdlPause::DottedEightNote => 36,
            SmdlPause::TwoThirdsOfQuarterNote => 32,
            SmdlPause::EightNote => 24,
            SmdlPause::DottedSixteenthNote => 18,
            SmdlPause::TwoThirdsOfEightNote => 16,
            SmdlPause::SixteenthNote => 12,
            SmdlPause::DottedThirtysecondNote => 9,
            SmdlPause::TwoThirdsOfSixteenthNote => 8,
            SmdlPause::ThirtysecondNote => 6,
            SmdlPause::DottedSixtyforthNote => 4,
            SmdlPause::TwoThirdsOfThirtysecondNote => 3,
            SmdlPause::SixtyforthNote => 2,
        }
    }
}

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, PartialOrd, FromPrimitive, Debug)]
pub enum SmdlSpecialOpCode {
    WaitAgain = 0x90,
    WaitAdd = 0x91,
    Wait1Byte = 0x92,
    Wait2Byte = 0x93,
    Wait3Byte = 0x94,
    TrackEnd = 0x98,
    LoopPoint = 0x99,
    SetOctave = 0xA0,
    SetTempo = 0xA4,
    SetHeader1 = 0xA9,
    SetHeader2 = 0xAA,
    SetSample = 0xAC,
    SetModu = 0xBE,
    SetBend = 0xD7,
    SetVolume = 0xE0,
    SetXpress = 0xE3,
    SetPan = 0xE8,
    //     NA_NOTE = 0x00, -1
    //     NA_DELTATIME = 0x80, 1
    Unk9C = 0x9C,
    Unk9D = 0x9D,
    UnkA8 = 0xA8,
    UnkB2 = 0xB2,
    UnkB4 = 0xB4,
    UnkB5 = 0xB5,
    UnkBF = 0xBF,
    UnkC0 = 0xC0,
    UnkD0 = 0xD0,
    UnkD1 = 0xD1,
    UnkD2 = 0xD2,
    UnkD4 = 0xD4,
    UnkD6 = 0xD6,
    UnkDB = 0xDB,
    UnkDC = 0xDC,
    UnkE2 = 0xE2,
    UnkEA = 0xEA,
    UnkF6 = 0xF6,
}

impl SmdlSpecialOpCode {
    pub fn parameter_length(&self) -> usize {
        match self {
            SmdlSpecialOpCode::WaitAgain => 0,
            SmdlSpecialOpCode::WaitAdd => 1,
            SmdlSpecialOpCode::Wait1Byte => 1,
            SmdlSpecialOpCode::Wait2Byte => 2, // LE
            SmdlSpecialOpCode::Wait3Byte => 2, // LE
            SmdlSpecialOpCode::TrackEnd => 0,
            SmdlSpecialOpCode::LoopPoint => 0,
            SmdlSpecialOpCode::SetOctave => 1,
            SmdlSpecialOpCode::SetTempo => 1,
            SmdlSpecialOpCode::SetHeader1 => 1,
            SmdlSpecialOpCode::SetHeader2 => 1,
            SmdlSpecialOpCode::SetSample => 1,
            SmdlSpecialOpCode::SetModu => 1,
            SmdlSpecialOpCode::SetBend => 2,
            SmdlSpecialOpCode::SetVolume => 1,
            SmdlSpecialOpCode::SetXpress => 1,
            SmdlSpecialOpCode::SetPan => 1,
            SmdlSpecialOpCode::Unk9C => 1,
            SmdlSpecialOpCode::Unk9D => 0,
            SmdlSpecialOpCode::UnkA8 => 2,
            SmdlSpecialOpCode::UnkB2 => 1,
            SmdlSpecialOpCode::UnkB4 => 2,
            SmdlSpecialOpCode::UnkB5 => 1,
            SmdlSpecialOpCode::UnkBF => 1,
            SmdlSpecialOpCode::UnkC0 => 0,
            SmdlSpecialOpCode::UnkD0 => 1,
            SmdlSpecialOpCode::UnkD1 => 1,
            SmdlSpecialOpCode::UnkD2 => 1,
            SmdlSpecialOpCode::UnkD4 => 3,
            SmdlSpecialOpCode::UnkD6 => 2,
            SmdlSpecialOpCode::UnkDB => 1,
            SmdlSpecialOpCode::UnkDC => 5,
            SmdlSpecialOpCode::UnkE2 => 3,
            SmdlSpecialOpCode::UnkEA => 3,
            SmdlSpecialOpCode::UnkF6 => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SmdlEvent {
    Special {
        op: SmdlSpecialOpCode,
        params: Vec<u8>,
    },
    Pause {
        value: SmdlPause,
    },
    Note {
        velocity: u8,
        octave_mod: i8, /* MIN: -2 */
        note: SmdlNote,
        key_down_duration: Option<u32>,
    },
}

impl SmdlEvent {
    /// Length of the event in ticks
    pub fn length(&self, previous_wait_time: usize) -> usize {
        match self {
            SmdlEvent::Special { op, params } => match op {
                SmdlSpecialOpCode::WaitAgain => previous_wait_time,
                SmdlSpecialOpCode::WaitAdd => previous_wait_time + params[0] as usize,
                SmdlSpecialOpCode::Wait1Byte => params[0] as usize,
                SmdlSpecialOpCode::Wait2Byte => ((params[1] as usize) << 8) + (params[0] as usize),
                SmdlSpecialOpCode::Wait3Byte => {
                    ((params[2] as usize) << 16)
                        + ((params[1] as usize) << 8)
                        + (params[0] as usize)
                }
                _ => 0,
            },
            SmdlEvent::Pause { value } => value.length(),
            SmdlEvent::Note { .. } => {
                // Playing notes doesn't add to the tick count.
                0
            }
        }
    }
}
