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

use crate::bytes::{StBytes, StBytesMut};
use crate::dse::date::DseDate;
use crate::dse::filename::DseFilename;
use crate::dse::st_smdl::eoc::SmdlEoc;
use crate::dse::st_smdl::event::{SmdlEvent, SmdlSpecialOpCode};
use crate::dse::st_smdl::song::SmdlSong;
use crate::dse::st_smdl::trk::SmdlTrack;
use crate::gettext::gettext;
use crate::python::PyResult;
use bytes::{Buf, BufMut, BytesMut};
use std::iter::repeat;

const SMDL_HEADER: &[u8] = b"smdl";
const FRAMES_PER_SECOND: u32 = 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Until {
    End,
    Loop,
    Event(SmdlEvent),
    Special(SmdlSpecialOpCode),
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
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

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Smdl {
    pub header: SmdlHeader,
    pub song: SmdlSong,
    pub tracks: Vec<SmdlTrack>,
    pub eoc: SmdlEoc,
}

impl Smdl {
    /// Returns the inner name of a SWDL file (stored in the header), without
    /// the overhead of reading in the entire file.
    /// This won't do any checks, so if an invalid / non-SWDL file is passed in,
    /// this will likely panic.
    pub fn name_for<T: AsRef<[u8]> + Buf>(raw: &T) -> DseFilename {
        DseFilename::from(&mut (&raw.as_ref()[0x20..0x30]))
    }

    /// Returns the length of the track (until `until`) in beats.
    /// This is the raw unadjusted length of events, without taking the BPM into account.
    pub fn len_in_beats(&self, until: Until) -> u32 {
        //self.tracks.iter().map(|t| Self::single_track_length_in_beats(t, until.clone())).max().unwrap()
        Self::single_track_length_in_beats(&self.tracks[0], until)
    }

    /// Returns the length of the track (until `until`) in microseconds.
    /// This is the length of events, scaled by 60000000 microseconds / (current tempo in bpm * self.song.tpqn).
    /// All events for which no tempo was defined yet are discarded.
    pub fn len_in_microseconds(&self, until: Until) -> u64 {
        //self.tracks.iter().map(|t| Self::single_track_length_in_ticks(t, self.song.tpqn, until.clone())).max().unwrap()
        Self::single_track_length_in_microseconds(&self.tracks[0], self.song.tpqn, until)
    }

    /// Returns the length of the track (until `until`) in ticks.
    pub fn len_in_ticks(&self, until: Until) -> u64 {
        (self.len_in_microseconds(until) as f64 / 1000000.0 * FRAMES_PER_SECOND as f64).ceil()
            as u64
    }

    fn single_track_length_in_beats(trk: &SmdlTrack, until: Until) -> u32 {
        let mut highest = 0u32;
        for (beat, e) in trk.iter_events_timed() {
            highest = beat;
            match &until {
                Until::End => {}
                Until::Loop => {
                    if let SmdlEvent::Special {
                        op: SmdlSpecialOpCode::LoopPoint,
                        ..
                    } = e
                    {
                        return highest;
                    }
                }
                Until::Special(opc) => {
                    if let SmdlEvent::Special { op, .. } = e {
                        if op == opc {
                            return highest;
                        }
                    }
                }
                Until::Event(em) => {
                    if e == em {
                        return highest;
                    }
                }
            }
        }
        highest
    }

    fn single_track_length_in_microseconds(trk: &SmdlTrack, tpqn: u16, until: Until) -> u64 {
        let mut current_tempo = 0;
        let mut previous = 0usize;
        let mut sum = 0u64;
        for e in trk.events.iter() {
            match e {
                SmdlEvent::Special {
                    op: SmdlSpecialOpCode::SetTempo,
                    params,
                } => {
                    current_tempo = 60000000 / (params[0] as u32 * tpqn as u32);
                }
                _ => {
                    if current_tempo > 0 {
                        let previous_c = e.length(previous);
                        if previous_c > 0 {
                            previous = previous_c;
                        }
                        sum += (previous_c as u64) * current_tempo as u64;
                    }
                }
            }
            match &until {
                Until::End => {}
                Until::Loop => {
                    if let SmdlEvent::Special {
                        op: SmdlSpecialOpCode::LoopPoint,
                        ..
                    } = e
                    {
                        return sum;
                    }
                }
                Until::Special(opc) => {
                    if let SmdlEvent::Special { op, .. } = e {
                        if op == opc {
                            return sum;
                        }
                    }
                }
                Until::Event(em) => {
                    if e == em {
                        return sum;
                    }
                }
            }
        }
        sum
    }
}

impl From<&mut StBytes> for PyResult<SmdlHeader> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 64,
            gettext("SMDL file too short (Header EOF).")
        );
        let header = source.copy_to_bytes(4);
        pyr_assert!(
            SMDL_HEADER == header,
            gettext("Invalid SMDL/Header header.")
        );
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
            unk9: source.get_u32_le(),
        })
    }
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
            eoc: <PyResult<SmdlEoc>>::from(&mut source)?,
        })
    }
}

impl From<Smdl> for StBytes {
    fn from(source: Smdl) -> Self {
        let track_len = source.tracks.len();
        let track_data: StBytes = source
            .tracks
            .into_iter()
            .flat_map(|track| {
                let mut data = StBytesMut::from(track);
                let data_len = data.len();
                if data_len % 4 != 0 {
                    data.extend(repeat(0x98).take(4 - data_len % 4));
                }
                data.freeze()
            })
            .collect();
        let track_data_len = track_data.len();
        let res: StBytes = source
            .header
            .to_bytes((track_data_len + 144) as u32)
            .into_iter()
            .chain(source.song.to_bytes(track_len as u8))
            .chain(track_data)
            .chain(StBytes::from(source.eoc))
            .collect();
        debug_assert_eq!(track_data_len + 144, res.len());
        res
    }
}

impl AsRef<Smdl> for Smdl {
    fn as_ref(&self) -> &Smdl {
        self
    }
}
