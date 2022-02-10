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

use std::io::Cursor;
use std::iter::{repeat};
use bytes::{Buf, BufMut, BytesMut};
use gettextrs::gettext;
use pyo3::{exceptions, PyResult};
use crate::bytes::{StBytes, StBytesMut};
use crate::dse::date::DseDate;
use crate::dse::filename::DseFilename;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////

const SMDL_HEADER: &[u8] = "smdl".as_bytes();

#[derive(Clone, Debug)]
pub struct SmdlHeader {
    pub version: u16,
    pub unk1: u8,
    pub unk2: u8,
    pub modified_date: DseDate,
    pub file_name: DseFilename,
    pub unk5: u32,
    pub unk6: u32,
    pub unk8: u32,
    pub unk9: u32,
}

impl From<&mut StBytes> for PyResult<SmdlHeader> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(source.len() >= 64, gettext("SMDL file too short (Header EOF)."));
        let header = source.copy_to_bytes(4);
        pyr_assert!(SMDL_HEADER == header, gettext("Invalid SMDL/Header header."));
        // 4 zero bytes;
        source.advance(4);
        // We don't validate the length (next 4 bytes):
        source.advance(4);
        let version = source.get_u16_le();
        let unk1 = source.get_u8();
        let unk2 = source.get_u8();
        // 8 zero bytes:
        source.advance(8);
        Ok(SmdlHeader {
            version,
            unk1,
            unk2,
            modified_date: source.into(),
            file_name: DseFilename::from_bytes_fixed(source, 16),
            unk5: source.get_u32_le(),
            unk6: source.get_u32_le(),
            unk8: source.get_u32_le(),
            unk9: source.get_u32_le()
        })
    }
}

impl SmdlHeader {
    fn to_bytes(&self, byte_len_smdl: u32) -> StBytes {
        let mut b = BytesMut::with_capacity(64);
        b.put_slice(SMDL_HEADER);
        b.put_u32_le(0);
        b.put_u32_le(byte_len_smdl);
        b.put_u16_le(self.version);
        b.put_u8(self.unk1);
        b.put_u8(self.unk2);
        b.put_u64(0);
        b.put(StBytes::from(self.modified_date.clone()).0);
        b.put(StBytes::from(self.file_name.clone()).0);
        b.put_u32_le(self.unk5);
        b.put_u32_le(self.unk6);
        b.put_u32_le(self.unk8);
        b.put_u32_le(self.unk9);
        debug_assert_eq!(64, b.len());
        b.into()
    }
}

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////

const SONG_HEADER: &[u8] = "song".as_bytes();

#[derive(Clone, Debug)]
pub struct SmdlSong {
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u16,
    pub tpqn: u16,
    pub unk5: u16,
    pub nbchans: u8,
    pub unk6: u32,
    pub unk7: u32,
    pub unk8: u32,
    pub unk9: u32,
    pub unk10: u16,
    pub unk11: u16,
    pub unk12: u32,
    initial_track_count: u8
}

impl SmdlSong {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        unk1: u32, unk2: u32, unk3: u32, unk4: u16, tpqn: u16, unk5: u16, nbchans: u8,
        unk6: u32, unk7: u32, unk8: u32, unk9: u32, unk10: u16, unk11: u16, unk12: u32
    ) -> Self {
        SmdlSong {
            unk1, unk2, unk3, unk4, tpqn, unk5, nbchans,
            unk6, unk7, unk8, unk9, unk10, unk11, unk12,
            initial_track_count: 0
        }
    }

    fn get_initial_track_count(&self) -> usize {
        self.initial_track_count as usize
    }
}

impl From<&mut StBytes> for PyResult<SmdlSong> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(source.len() >= 64, gettext("SMDL file too short (Song EOF)."));
        let header = source.copy_to_bytes(4);
        pyr_assert!(SONG_HEADER == header, gettext("Invalid SMDL/Song header."));
        let unk1 = source.get_u32_le();
        let unk2 = source.get_u32_le();
        let unk3 = source.get_u32_le();
        let unk4 = source.get_u16_le();
        let tpqn = source.get_u16_le();
        let unk5 = source.get_u16_le();
        let initial_track_count = source.get_u8();
        let nbchans = source.get_u8();
        let unk6 = source.get_u32_le();
        let unk7 = source.get_u32_le();
        let unk8 = source.get_u32_le();
        let unk9 = source.get_u32_le();
        let unk10 = source.get_u16_le();
        let unk11 = source.get_u16_le();
        let unk12 = source.get_u32_le();
        // 16 0xFF bytes:
        source.advance(16);
        Ok(SmdlSong {
            unk1, unk2, unk3, unk4, tpqn, unk5, nbchans, unk6,
            unk7, unk8, unk9, unk10, unk11, unk12, initial_track_count
        })
    }
}

impl SmdlSong {
    fn to_bytes(&self, number_tracks: u8) -> StBytes {
        let mut b = BytesMut::with_capacity(64);
        b.put_slice(SONG_HEADER);
        b.put_u32_le(self.unk1);
        b.put_u32_le(self.unk2);
        b.put_u32_le(self.unk3);
        b.put_u16_le(self.unk4);
        b.put_u16_le(self.tpqn);
        b.put_u16_le(self.unk5);
        b.put_u8(number_tracks);
        b.put_u8(self.nbchans);
        b.put_u32_le(self.unk6);
        b.put_u32_le(self.unk7);
        b.put_u32_le(self.unk8);
        b.put_u32_le(self.unk9);
        b.put_u16_le(self.unk10);
        b.put_u16_le(self.unk11);
        b.put_u32_le(self.unk12);
        b.extend(repeat(0xFF).take(16));
        debug_assert_eq!(64, b.len());
        b.into()
    }
}

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////

const EOC_HEADER: &[u8] = "eoc\x20".as_bytes();

#[derive(Clone, Debug)]
pub struct SmdlEoc {
    pub param1: u32,
    pub param2: u32,
}

impl From<&mut StBytes> for PyResult<SmdlEoc> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(source.len() >= 16, gettext("SMDL file too short (EOC EOF)."));
        let header = source.copy_to_bytes(4);
        pyr_assert!(EOC_HEADER == header, gettext("Invalid SMDL/EOC header."));
        let param1 = source.get_u32_le();
        let param2 = source.get_u32_le();
        source.advance(4);
        Ok(SmdlEoc { param1, param2 })
    }
}

impl From<SmdlEoc> for StBytes {
    fn from(source: SmdlEoc) -> Self {
        let mut b = BytesMut::with_capacity(16);
        b.put_slice(EOC_HEADER);
        b.put_u32_le(source.param1);
        b.put_u32_le(source.param2);
        b.put_u32_le(0);
        debug_assert_eq!(16, b.len());
        b.into()
    }
}

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////

const TRK_HEADER: &[u8] = "trk\x20".as_bytes();

#[derive(Clone, Debug)]
pub struct SmdlTrackHeader {
    pub param1: u32,
    pub param2: u32,
    len: u32
}

impl SmdlTrackHeader {
    pub(crate) fn new(param1: u32, param2: u32) -> Self {
        Self {param1, param2, len: 0}
    }

    fn get_initial_length(&self) -> usize {
        self.len as usize
    }
}

impl From<&mut StBytes> for PyResult<SmdlTrackHeader> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(source.len() >= 16, gettext("SMDL file too short (Track EOF)."));
        let header = source.copy_to_bytes(4);
        pyr_assert!(TRK_HEADER == header, gettext("Invalid SMDL/Track header."));
        let param1 = source.get_u32_le();
        let param2 = source.get_u32_le();
        let len = source.get_u32_le();
        Ok(SmdlTrackHeader { param1, param2, len })
    }
}

impl SmdlTrackHeader {
    fn to_bytes(&self, length: u32) -> StBytes {
        let mut b = BytesMut::with_capacity(16);
        b.put_slice(TRK_HEADER);
        b.put_u32_le(self.param1);
        b.put_u32_le(self.param2);
        b.put_u32_le(length);
        debug_assert_eq!(16, b.len());
        b.into()
    }
}

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct SmdlTrackPreamble {
    pub track_id: u8,
    pub channel_id: u8,
    pub unk1: u8,
    pub unk2: u8,
}

impl From<&mut StBytes> for PyResult<SmdlTrackPreamble> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(source.len() >= 4, gettext("SMDL file too short (EOC EOF)."));
        Ok(SmdlTrackPreamble {
            track_id: source.get_u8(),
            channel_id: source.get_u8(),
            unk1: source.get_u8(),
            unk2: source.get_u8(),
        })
    }
}

impl From<SmdlTrackPreamble> for StBytes {
    fn from(source: SmdlTrackPreamble) -> Self {
        let mut b = BytesMut::with_capacity(4);
        b.put_u8(source.track_id);
        b.put_u8(source.channel_id);
        b.put_u8(source.unk1);
        b.put_u8(source.unk2);
        debug_assert_eq!(4, b.len());
        b.into()
    }
}

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////
const PLAY_NOTE_MAX: u8 = 0x7F;
const PAUSE_NOTE_MAX: u8 = 0x8F;

#[repr(u8)]
#[derive(Clone, PartialEq, PartialOrd, FromPrimitive, Debug)]
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

#[repr(u8)]
#[derive(Clone, PartialEq, PartialOrd, FromPrimitive, Debug)]
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
    SixtyforthNote = 0x8F
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
            SmdlPause::SixtyforthNote => 2
        }
    }
}

#[repr(u8)]
#[derive(Clone, PartialEq, PartialOrd, FromPrimitive, Debug)]
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
            SmdlSpecialOpCode::Wait2Byte => 2,  // LE
            SmdlSpecialOpCode::Wait3Byte => 2,  // LE
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

#[derive(Clone, Debug)]
pub enum SmdlEvent {
    Special { op: SmdlSpecialOpCode, params: Vec<u8> },
    Pause { value: SmdlPause },
    Note { velocity: u8, octave_mod: i8 /* MIN: -2 */, note: SmdlNote, key_down_duration: Option<u32> },
}

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////

const TRACK_EOF_MESSAGE: &str = "Reached EOF while reading tracks from SMDL.";

#[derive(Clone, Debug)]
pub struct SmdlTrack {
    pub header: SmdlTrackHeader,
    pub preamble: SmdlTrackPreamble,
    pub events: Vec<SmdlEvent>,
}

impl From<&mut StBytes> for PyResult<SmdlTrack> {
    fn from(source: &mut StBytes) -> Self {
        let header_err: PyResult<SmdlTrackHeader> = source.into();
        let header = header_err?;

        let mut cursor = Cursor::new(source.clone());
        cursor.advance(4); // preamble; see now.
        let preamble_err: PyResult<SmdlTrackPreamble> = source.into();
        let preamble = preamble_err?;
        let length = header.get_initial_length();
        pyr_assert!(length <= cursor.remaining() as usize, TRACK_EOF_MESSAGE);

        let mut events = Vec::with_capacity(100);
        while (cursor.position() as usize) < length {
            pyr_assert!(cursor.remaining() >= 1, TRACK_EOF_MESSAGE);
            let op_code = cursor.get_u8();
            if op_code <= PLAY_NOTE_MAX {
                let velocity = op_code;
                pyr_assert!(cursor.remaining() >= 1, TRACK_EOF_MESSAGE);
                let param1 = cursor.get_u8();
                let number_params = (param1 >> 6) & 0x3;
                let octave_mod: i8 = ((param1 as i8 >> 4) & 0x3) - 2;
                let note = SmdlNote::from_u8(param1 & 0xF).unwrap();
                pyr_assert!(number_params < 4, "Invalid amount of parameters for note event in SMDL.");
                let key_down_duration = if number_params == 1 {
                    pyr_assert!(cursor.remaining() >= 1, TRACK_EOF_MESSAGE);
                    Some(cursor.get_u8() as u32)
                } else if number_params == 2 {
                    pyr_assert!(cursor.remaining() >= 2, TRACK_EOF_MESSAGE);
                    Some(cursor.get_u16() as u32)  // big endian?? really??
                } else if number_params == 3 {
                    pyr_assert!(cursor.remaining() >= 3, TRACK_EOF_MESSAGE);
                    Some(((cursor.get_u16() as u32) << 8) + cursor.get_u8() as u32)  // big endian?? really??
                } else {
                    None
                };
                events.push(SmdlEvent::Note {
                    note, velocity, octave_mod, key_down_duration
                });
            } else if op_code <= PAUSE_NOTE_MAX {
                events.push(SmdlEvent::Pause { value: SmdlPause::from_u8(op_code).unwrap() });
            } else if op_code == 0xAB {
                // skip byte
                pyr_assert!(cursor.remaining() >= 1, TRACK_EOF_MESSAGE);
                cursor.advance(1);
            } else if op_code == 0xCB || op_code == 0xF8 {
                // skip 2 bytes
                pyr_assert!(cursor.remaining() >= 2, TRACK_EOF_MESSAGE);
                cursor.advance(2);
            } else {
                let op = SmdlSpecialOpCode::from_u8(op_code).ok_or_else(|| exceptions::PyAssertionError::new_err("Invalid SMDL track event."))?;
                let param_len = op.parameter_length();
                pyr_assert!(cursor.remaining() >= param_len, TRACK_EOF_MESSAGE);
                let params = (0..param_len).map(|_| cursor.get_u8()).collect::<Vec<u8>>();
                events.push(SmdlEvent::Special { op, params });
            }
        }

        source.advance((cursor.position() - 4) as usize);
        //Padding
        let padding_needed = 4 - (length % 4);
        if padding_needed > 0 && padding_needed < 4 {
            source.advance(padding_needed);
        }
        Ok(SmdlTrack { header, preamble, events })
    }
}

impl From<SmdlTrack> for StBytesMut {
    fn from(source: SmdlTrack) -> Self {
        //         <- events = self._events_to_bytes(track.events)
        let mut events = BytesMut::with_capacity(source.events.len());
        for event in source.events {
            match event {
                SmdlEvent::Note { note, velocity, key_down_duration, octave_mod } => {
                    events.put_u8(velocity);
                    let n_p = match key_down_duration {
                        None => 0,
                        Some(x) if x > 0xFFFFFF => panic!("Too big of a value for key_down_duration in event."),
                        Some(x) if x > 0xFFFF => 3,
                        Some(x) if x > 0xFF => 2,
                        Some(x) => 1,
                    };
                    events.put_u8((note as u8 & 0xF) + (((octave_mod + 2) as u8 & 0x3) << 4) + ((n_p & 0x3) << 6));
                    if let Some(key_down_duration) = key_down_duration {
                        match n_p {
                            1 => events.put_u8(key_down_duration as u8),
                            2 => events.put_u16(key_down_duration as u16),
                            3 => {
                                events.put_u16((key_down_duration >> 8) as u16);
                                events.put_u8((key_down_duration & 0xF) as u8);
                            },
                            _ => {}
                        }
                    }
                }
                SmdlEvent::Pause { value } => {
                    events.put_u8(value as u8);
                }
                SmdlEvent::Special { op, params } => {
                    events.put_u8(op as u8);
                    events.put_slice(&params);
                }
            }
        }
        let preamble = StBytes::from(source.preamble).0;
        let mut data: BytesMut = source.header.to_bytes((preamble.len() + events.len()) as u32).into_iter()
            .chain(preamble.into_iter())
            .chain(events.into_iter())
            .collect();
        if data.len() % 4 != 0 {
            data.extend(repeat(0x98).take(4 - data.len() % 4));
        }
        data.into()
    }
}

impl From<SmdlTrack> for StBytes {
    fn from(source: SmdlTrack) -> Self {
        StBytesMut::from(source).0.into()
    }
}

////////////////////////////////////////////////
////////////////////////////////////////////////
////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Smdl {
    pub header: SmdlHeader,
    pub song: SmdlSong,
    pub tracks: Vec<SmdlTrack>,
    pub eoc: SmdlEoc
}

impl From<StBytes> for PyResult<Smdl> {
    fn from(mut source: StBytes) -> Self {
        let header = <PyResult<SmdlHeader>>::from(&mut source)?;
        let song = <PyResult<SmdlSong>>::from(&mut source)?;
        let tracks = (0..song.get_initial_track_count())
            .map(|_| <&mut StBytes>::into(&mut source))
            .collect::<PyResult<Vec<SmdlTrack>>>()?;
        Ok(Smdl {
            header,
            song,
            tracks,
            eoc: <PyResult<SmdlEoc>>::from(&mut source)?
        })
    }
}

impl From<Smdl> for StBytes {
    fn from(source: Smdl) -> Self {
        let track_len = source.tracks.len();
        let track_data: StBytes = source.tracks.into_iter().flat_map(|track| {
            let mut data = StBytesMut::from(track);
            let data_len = data.len();
            if data_len % 4 != 0 {
                data.extend(repeat(0x98).take(4 - data_len % 4));
            }
            data.freeze()
        }).collect();
        let track_data_len = track_data.len();
        let res: StBytes = source.header.to_bytes((track_data_len + 144) as u32).into_iter()
            .chain(source.song.to_bytes(track_len as u8).into_iter())
            .chain(track_data.into_iter())
            .chain(StBytes::from(source.eoc).into_iter())
            .collect();
        debug_assert_eq!(track_data_len + 144, res.len());
        res
    }
}
