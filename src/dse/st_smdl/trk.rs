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
use crate::dse::st_smdl::event::{
    SmdlEvent, SmdlNote, SmdlPause, SmdlSpecialOpCode, PAUSE_NOTE_MAX, PLAY_NOTE_MAX,
};
use crate::gettext::gettext;
use crate::python::{exceptions, PyResult};
use bytes::{Buf, BufMut, BytesMut};
use core::iter::repeat;
use num_traits::FromPrimitive;
use std::io::Cursor;
use std::slice;

const TRK_HEADER: &[u8] = b"trk\x20";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmdlTrackHeader {
    pub param1: u32,
    pub param2: u32,
    len: u32,
}

impl SmdlTrackHeader {
    #[allow(dead_code)] // if python is not enabled.
    pub(crate) fn new(param1: u32, param2: u32) -> Self {
        Self {
            param1,
            param2,
            len: 0,
        }
    }

    fn empty() -> Self {
        Self {
            param1: 16777216, // UNKNOWN!! Value often used.
            param2: 65284,    // UNKNOWN!! Value often used.
            len: 0,
        }
    }

    fn get_initial_length(&self) -> usize {
        self.len as usize
    }
}

impl From<&mut StBytes> for PyResult<SmdlTrackHeader> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 16,
            gettext("SMDL file too short (Track EOF).")
        );
        let header = source.copy_to_bytes(4);
        pyr_assert!(TRK_HEADER == header, gettext("Invalid SMDL/Track header."));
        let param1 = source.get_u32_le();
        let param2 = source.get_u32_le();
        let len = source.get_u32_le();
        Ok(SmdlTrackHeader {
            param1,
            param2,
            len,
        })
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmdlTrackPreamble {
    pub track_id: u8,
    pub channel_id: u8,
    pub unk1: u8,
    pub unk2: u8,
}

impl SmdlTrackPreamble {
    fn new(track_id: u8, channel_id: u8) -> Self {
        Self {
            track_id,
            channel_id,
            unk1: 0, // Unknown!! Value often used.
            unk2: 0, // Unknown!! Value often used.
        }
    }
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

const TRACK_EOF_MESSAGE: &str = "Reached EOF while reading tracks from SMDL.";

#[derive(Debug, Clone)]
pub struct SmdlTrackIter<'a> {
    event_iter: slice::Iter<'a, SmdlEvent>,
    previous: usize,
    sum: u32,
}

impl<'a> SmdlTrackIter<'a> {
    fn new(event_iter: slice::Iter<'a, SmdlEvent>) -> Self {
        Self {
            event_iter,
            previous: 0,
            sum: 0,
        }
    }
}

impl<'a> Iterator for SmdlTrackIter<'a> {
    type Item = (u32, &'a SmdlEvent);

    fn next(&mut self) -> Option<Self::Item> {
        self.event_iter.next().map(|e| {
            let previous_c = e.length(self.previous);
            if previous_c > 0 {
                self.previous = previous_c;
            }
            self.sum += previous_c as u32;
            (self.sum, e)
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmdlTrack {
    pub header: SmdlTrackHeader,
    pub preamble: SmdlTrackPreamble,
    pub events: Vec<SmdlEvent>,
}

impl SmdlTrack {
    pub fn new(track_id: u8, channel_id: u8) -> Self {
        Self {
            header: SmdlTrackHeader::empty(),
            preamble: SmdlTrackPreamble::new(track_id, channel_id),
            events: vec![],
        }
    }
    /// Iterates over all events as tuples of their tick/beat and the event itself.
    pub fn iter_events_timed(&self) -> SmdlTrackIter {
        SmdlTrackIter::new(self.events.iter())
    }
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
        pyr_assert!(length <= cursor.remaining(), TRACK_EOF_MESSAGE);

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
                pyr_assert!(
                    number_params < 4,
                    "Invalid amount of parameters for note event in SMDL."
                );
                let key_down_duration = if number_params == 1 {
                    pyr_assert!(cursor.remaining() >= 1, TRACK_EOF_MESSAGE);
                    Some(cursor.get_u8() as u32)
                } else if number_params == 2 {
                    pyr_assert!(cursor.remaining() >= 2, TRACK_EOF_MESSAGE);
                    #[allow(clippy::disallowed_methods)]
                    Some(cursor.get_u16() as u32) // big endian?? really??
                } else if number_params == 3 {
                    pyr_assert!(cursor.remaining() >= 3, TRACK_EOF_MESSAGE);
                    #[allow(clippy::disallowed_methods)] // big endian?? really??
                    Some(((cursor.get_u16() as u32) << 8) + cursor.get_u8() as u32)
                } else {
                    None
                };
                events.push(SmdlEvent::Note {
                    note,
                    velocity,
                    octave_mod,
                    key_down_duration,
                });
            } else if op_code <= PAUSE_NOTE_MAX {
                events.push(SmdlEvent::Pause {
                    value: SmdlPause::from_u8(op_code).unwrap(),
                });
            } else if op_code == 0xAB {
                // skip byte
                pyr_assert!(cursor.remaining() >= 1, TRACK_EOF_MESSAGE);
                cursor.advance(1);
            } else if op_code == 0xCB || op_code == 0xF8 {
                // skip 2 bytes
                pyr_assert!(cursor.remaining() >= 2, TRACK_EOF_MESSAGE);
                cursor.advance(2);
            } else {
                let op = SmdlSpecialOpCode::from_u8(op_code).ok_or_else(|| {
                    exceptions::PyAssertionError::new_err("Invalid SMDL track event.")
                })?;
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
        Ok(SmdlTrack {
            header,
            preamble,
            events,
        })
    }
}

impl From<SmdlTrack> for StBytesMut {
    fn from(source: SmdlTrack) -> Self {
        //         <- events = self._events_to_bytes(track.events)
        let mut events = BytesMut::with_capacity(source.events.len());
        for event in source.events {
            match event {
                SmdlEvent::Note {
                    note,
                    velocity,
                    key_down_duration,
                    octave_mod,
                } => {
                    events.put_u8(velocity);
                    let n_p = match key_down_duration {
                        None => 0,
                        Some(x) if x > 0xFFFFFF => {
                            panic!("Too big of a value for key_down_duration in event.")
                        }
                        Some(x) if x > 0xFFFF => 3,
                        Some(x) if x > 0xFF => 2,
                        Some(_) => 1,
                    };
                    events.put_u8(
                        (note as u8 & 0xF)
                            + (((octave_mod + 2) as u8 & 0x3) << 4)
                            + ((n_p & 0x3) << 6),
                    );
                    if let Some(key_down_duration) = key_down_duration {
                        match n_p {
                            1 => events.put_u8(key_down_duration as u8),
                            2 => events.put_u16(key_down_duration as u16),
                            3 => {
                                events.put_u16((key_down_duration >> 8) as u16);
                                events.put_u8((key_down_duration & 0xF) as u8);
                            }
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
        let mut data: BytesMut = source
            .header
            .to_bytes((preamble.len() + events.len()) as u32)
            .into_iter()
            .chain(preamble)
            .chain(events)
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
