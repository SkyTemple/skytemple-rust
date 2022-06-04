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
use crate::dse::st_swdl::pcmd::SwdlPcmd;
use crate::gettext::gettext;
use crate::python::PyResult;
use bytes::{Buf, BufMut, BytesMut};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::iter::repeat;

const WAVI_HEADER: &[u8] = b"wavi";

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, FromPrimitive, Debug)]
pub enum SampleFormatConsts {
    Pcm8bit = 0x0000,
    Pcm16bit = 0x0100,
    Adpcm4bit = 0x0200,
    Psg = 0x0300, // possibly
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwdlPcmdReference {
    pub offset: u32,
    pub length: u32,
}

impl SwdlPcmdReference {
    pub fn new(offset: u32, length: u32) -> Self {
        SwdlPcmdReference { offset, length }
    }

    pub fn get_sample<'swdl>(&self, pcmd: &'swdl SwdlPcmd) -> &'swdl [u8] {
        &pcmd.chunk_data[(self.offset as usize)..(self.offset + self.length) as usize]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwdlSampleInfoTblEntry {
    pub id: u16,
    pub ftune: i8,
    pub ctune: i8,
    pub rootkey: i8,
    pub ktps: i8,
    pub volume: i8,
    pub pan: i8,
    pub unk5: u8,
    pub unk58: u8,
    pub sample_format: Option<SampleFormatConsts>,
    pub unk9: u8,
    pub loops: bool,
    pub unk10: u16,
    pub unk11: u16,
    pub unk12: u16,
    pub unk13: u32,
    pub sample_rate: u32,
    pub sample: Option<SwdlPcmdReference>,
    pub loop_begin_pos: u32,
    pub loop_length: u32,
    pub envelope: u8,
    pub envelope_multiplier: u8,
    pub unk19: u8,
    pub unk20: u8,
    pub unk21: u16,
    pub unk22: u16,
    pub attack_volume: i8,
    pub attack: i8,
    pub decay: i8,
    pub sustain: i8,
    pub hold: i8,
    pub decay2: i8,
    pub release: i8,
    pub unk57: i8,
    pub(crate) sample_pos: u32,
}

impl SwdlSampleInfoTblEntry {
    pub(crate) fn get_initial_sample_pos(&self) -> u32 {
        self.sample_pos
    }
}

impl From<&mut StBytes> for PyResult<SwdlSampleInfoTblEntry> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 64,
            gettext("SWDL file too short (Sample Table EOF).")
        );
        // 2 padding/unknown bytes;
        source.advance(2);
        let id = source.get_u16_le();
        let ftune = source.get_i8();
        let ctune = source.get_i8();
        let rootkey = source.get_i8();
        let ktps = source.get_i8();
        let volume = source.get_i8();
        let pan = source.get_i8();
        let unk5 = source.get_u8();
        let unk58 = source.get_u8();
        // 6 unknown / static bytes:
        source.advance(6);
        let sample_format = SampleFormatConsts::from_u16(source.get_u16_le());
        let unk9 = source.get_u8();
        let loops = source.get_u8() > 0;
        let unk10 = source.get_u16_le();
        let unk11 = source.get_u16_le();
        let unk12 = source.get_u16_le();
        let unk13 = source.get_u32_le();
        let sample_rate = source.get_u32_le();
        // Read sample data later into this model
        let sample = None;
        let sample_pos = source.get_u32_le();
        // (For ADPCM samples, the 4 bytes preamble is counted in the loopbeg!)
        let loop_begin_pos = source.get_u32_le();
        let loop_length = source.get_u32_le();
        let envelope = source.get_u8();
        let envelope_multiplier = source.get_u8();
        let unk19 = source.get_u8();
        let unk20 = source.get_u8();
        let unk21 = source.get_u16_le();
        let unk22 = source.get_u16_le();
        let attack_volume = source.get_i8();
        let attack = source.get_i8();
        let decay = source.get_i8();
        let sustain = source.get_i8();
        let hold = source.get_i8();
        let decay2 = source.get_i8();
        let release = source.get_i8();
        let unk57 = source.get_i8();
        Ok(SwdlSampleInfoTblEntry {
            id,
            ftune,
            ctune,
            rootkey,
            ktps,
            volume,
            pan,
            unk5,
            unk58,
            sample_format,
            unk9,
            loops,
            unk10,
            unk11,
            unk12,
            unk13,
            sample_rate,
            sample,
            sample_pos,
            loop_begin_pos,
            loop_length,
            envelope,
            envelope_multiplier,
            unk19,
            unk20,
            unk21,
            unk22,
            attack_volume,
            attack,
            decay,
            sustain,
            hold,
            decay2,
            release,
            unk57,
        })
    }
}

impl SwdlSampleInfoTblEntry {
    pub fn get_sample_length(&self) -> u32 {
        (self.loop_begin_pos + self.loop_length) * 4
    }
}

impl From<SwdlSampleInfoTblEntry> for StBytes {
    fn from(source: SwdlSampleInfoTblEntry) -> Self {
        let mut b = BytesMut::with_capacity(64);
        b.put(&[0x01, 0xAA][..]);
        b.put_u16_le(source.id);
        b.put_i8(source.ftune);
        b.put_i8(source.ctune);
        b.put_i8(source.rootkey);
        b.put_i8(source.ktps);
        b.put_i8(source.volume);
        b.put_i8(source.pan);
        b.put_u8(source.unk5);
        b.put_u8(source.unk58);
        b.put(&[0x00, 0x00, 0xAA, 0xAA, 0x15, 0x04][..]);
        b.put_u16_le(source.sample_format.map_or(0, |x| x as u16));
        b.put_u8(source.unk9);
        b.put_u8(source.loops as u8);
        b.put_u16_le(source.unk10);
        b.put_u16_le(source.unk11);
        b.put_u16_le(source.unk12);
        b.put_u32_le(source.unk13);
        b.put_u32_le(source.sample_rate);
        // TODO: is this safe / correct? Do we need to set this properly??
        b.put_u32_le(source.sample_pos);
        b.put_u32_le(source.loop_begin_pos);
        b.put_u32_le(source.loop_length);
        b.put_u8(source.envelope);
        b.put_u8(source.envelope_multiplier);
        b.put_u8(source.unk19);
        b.put_u8(source.unk20);
        b.put_u16_le(source.unk21);
        b.put_u16_le(source.unk22);
        b.put_i8(source.attack_volume);
        b.put_i8(source.attack);
        b.put_i8(source.decay);
        b.put_i8(source.sustain);
        b.put_i8(source.hold);
        b.put_i8(source.decay2);
        b.put_i8(source.release);
        b.put_i8(source.unk57);
        debug_assert_eq!(64, b.len());
        b.into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwdlWavi {
    pub sample_info_table: Vec<Option<SwdlSampleInfoTblEntry>>,
    initial_length: usize,
}

impl SwdlWavi {
    pub fn new(sample_info_table: Vec<Option<SwdlSampleInfoTblEntry>>) -> Self {
        SwdlWavi {
            sample_info_table,
            initial_length: 0,
        }
    }
}

impl SwdlWavi {
    pub(crate) fn get_initial_length(&self) -> usize {
        self.initial_length
    }

    pub fn from_bytes(source: &mut StBytes, number_slots: u16) -> PyResult<Self> {
        pyr_assert!(
            source.len() >= 16 + (number_slots as usize * 2),
            gettext("SWDL file too short (Wavi EOF).")
        );
        let header = source.copy_to_bytes(4);
        pyr_assert!(WAVI_HEADER == header, gettext("Invalid SWDL/Wavi header."));
        // 0x00, 0x00, 0x15, 0x04, 0x10, 0x00, 0x00, 0x00:
        source.advance(8);
        let len_chunk_data = source.get_u32_le();
        let mut toc = source.clone();
        let sample_info_table = (0..(number_slots))
            .map(|_| {
                let pnt = toc.get_u16_le();
                pyr_assert!(
                    (pnt as u32) < len_chunk_data,
                    gettext("SWDL Wavi length invalid; tried to read past EOF.")
                );
                if pnt > 0 {
                    let mut dst = source.clone();
                    pyr_assert!(
                        dst.len() >= pnt as usize,
                        gettext("SWDL file too short (Wavi EOF).")
                    );
                    dst.advance(pnt as usize);
                    Ok(Some(<PyResult<SwdlSampleInfoTblEntry>>::from(&mut dst)?))
                } else {
                    Ok(None)
                }
            })
            .collect::<PyResult<Vec<Option<SwdlSampleInfoTblEntry>>>>()?;
        source.advance(len_chunk_data as usize);
        Ok(Self {
            sample_info_table,
            initial_length: (len_chunk_data + 0x10) as usize,
        })
    }
}

impl From<SwdlWavi> for StBytes {
    fn from(source: SwdlWavi) -> Self {
        let toc_len = source.sample_info_table.len() * 2;
        let mut toc = BytesMut::with_capacity(toc_len);
        let mut content = BytesMut::with_capacity(source.sample_info_table.len() * 64);

        // Padding after TOC
        if toc_len % 16 != 0 {
            content.extend(repeat(0xAA).take(16 - (toc_len % 16)));
        }

        for wav in source.sample_info_table.into_iter() {
            match wav {
                Some(wav) => {
                    toc.put_u16_le((toc_len + content.len()) as u16);
                    content.put(StBytes::from(wav).0)
                }
                None => toc.put_u16_le(0),
            }
        }
        debug_assert_eq!(toc.len(), toc_len);

        let mut data = BytesMut::with_capacity(0x10);
        data.put(&b"wavi\0\0\x15\x04\x10\0\0\0"[..]);
        data.put_u32_le((toc_len + content.len()) as u32);
        data.put(toc);
        debug_assert_eq!(0x10 + toc_len, data.len());
        data.put(content);
        data.into()
    }
}
