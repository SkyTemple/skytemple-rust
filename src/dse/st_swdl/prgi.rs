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
use crate::gettext::gettext;
use crate::python::PyResult;
use bytes::{Buf, BufMut, BytesMut};
use std::iter::repeat;

pub const PRGI_HEADER: &[u8] = "prgi".as_bytes();
const LEN_LFO: usize = 16;
const LEN_SPLITS: usize = 48;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwdlLfoEntry {
    pub unk34: u8,
    pub unk52: u8,
    pub dest: u8,
    pub wshape: u8,
    pub rate: u16,
    pub unk29: u16,
    pub depth: u16,
    pub delay: u16,
    pub unk32: u16,
    pub unk33: u16,
}

impl From<&mut StBytes> for PyResult<SwdlLfoEntry> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= LEN_LFO,
            gettext("SWDL file too short (LFO EOF).")
        );
        Ok(SwdlLfoEntry {
            unk34: source.get_u8(),
            unk52: source.get_u8(),
            dest: source.get_u8(),
            wshape: source.get_u8(),
            rate: source.get_u16_le(),
            unk29: source.get_u16_le(),
            depth: source.get_u16_le(),
            delay: source.get_u16_le(),
            unk32: source.get_u16_le(),
            unk33: source.get_u16_le(),
        })
    }
}

impl From<SwdlLfoEntry> for StBytes {
    fn from(source: SwdlLfoEntry) -> Self {
        let mut b = BytesMut::with_capacity(LEN_LFO);
        b.put_u8(source.unk34);
        b.put_u8(source.unk52);
        b.put_u8(source.dest);
        b.put_u8(source.wshape);
        b.put_u16_le(source.rate);
        b.put_u16_le(source.unk29);
        b.put_u16_le(source.depth);
        b.put_u16_le(source.delay);
        b.put_u16_le(source.unk32);
        b.put_u16_le(source.unk33);
        debug_assert_eq!(LEN_LFO, b.len());
        b.into()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwdlSplitEntry {
    pub id: u8,
    pub unk11: u8,
    pub unk25: u8,
    pub lowkey: i8,
    pub hikey: i8,
    pub lolevel: i8,
    pub hilevel: i8,
    pub unk16: i32,
    pub unk17: i16,
    pub sample_id: u16,
    pub ftune: i8,
    pub ctune: i8,
    pub rootkey: i8,
    pub ktps: i8,
    pub sample_volume: i8,
    pub sample_pan: i8,
    pub keygroup_id: i8,
    pub unk22: u8,
    pub unk23: u16,
    pub unk24: u16,
    pub envelope: u8,
    pub envelope_multiplier: u8,
    pub unk37: u8,
    pub unk38: u8,
    pub unk39: u16,
    pub unk40: u16,
    pub attack_volume: i8,
    pub attack: i8,
    pub decay: i8,
    pub sustain: i8,
    pub hold: i8,
    pub decay2: i8,
    pub release: i8,
    pub unk53: i8,
}

impl From<&mut StBytes> for PyResult<SwdlSplitEntry> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= LEN_SPLITS,
            gettext("SWDL file too short (Split EOF).")
        );
        // One zero:
        source.advance(1);
        let id = source.get_u8();
        let unk11 = source.get_u8();
        let unk25 = source.get_u8();
        let lowkey = source.get_i8();
        let hikey = source.get_i8();
        pyr_assert!(
            source.get_i8() == lowkey,
            gettext("SWDL file: Invalid lowkey duplicate (Split EOF).")
        );
        pyr_assert!(
            source.get_i8() == hikey,
            gettext("SWDL file: Invalid hikey duplicate (Split EOF).")
        );
        let lolevel = source.get_i8();
        let hilevel = source.get_i8();
        pyr_assert!(
            source.get_i8() == lolevel,
            gettext("SWDL file: Invalid lolevel duplicate (Split EOF).")
        );
        pyr_assert!(
            source.get_i8() == hilevel,
            gettext("SWDL file: Invalid hilevel duplicate (Split EOF).")
        );
        let unk16 = source.get_i32_le();
        let unk17 = source.get_i16_le();
        let sample_id = source.get_u16_le();
        let ftune = source.get_i8();
        let ctune = source.get_i8();
        let rootkey = source.get_i8();
        let ktps = source.get_i8();
        let sample_volume = source.get_i8();
        let sample_pan = source.get_i8();
        let keygroup_id = source.get_i8();
        let unk22 = source.get_u8();
        let unk23 = source.get_u16_le();
        let unk24 = source.get_u16_le();

        let envelope = source.get_u8();
        let envelope_multiplier = source.get_u8();
        let unk37 = source.get_u8();
        let unk38 = source.get_u8();
        let unk39 = source.get_u16_le();
        let unk40 = source.get_u16_le();
        let attack_volume = source.get_i8();
        let attack = source.get_i8();
        let decay = source.get_i8();
        let sustain = source.get_i8();
        let hold = source.get_i8();
        let decay2 = source.get_i8();
        let release = source.get_i8();
        let unk53 = source.get_i8();
        Ok(SwdlSplitEntry {
            id,
            unk11,
            unk25,
            lowkey,
            hikey,
            lolevel,
            hilevel,
            unk16,
            unk17,
            sample_id,
            ftune,
            ctune,
            rootkey,
            ktps,
            sample_volume,
            sample_pan,
            keygroup_id,
            unk22,
            unk23,
            unk24,
            envelope,
            envelope_multiplier,
            unk37,
            unk38,
            unk39,
            unk40,
            attack_volume,
            attack,
            decay,
            sustain,
            hold,
            decay2,
            release,
            unk53,
        })
    }
}

impl From<SwdlSplitEntry> for StBytes {
    fn from(source: SwdlSplitEntry) -> Self {
        let mut b = BytesMut::with_capacity(LEN_SPLITS);
        b.put_u8(0);
        b.put_u8(source.id);
        b.put_u8(source.unk11);
        b.put_u8(source.unk25);
        b.put_i8(source.lowkey);
        b.put_i8(source.hikey);
        b.put_i8(source.lowkey);
        b.put_i8(source.hikey);
        b.put_i8(source.lolevel);
        b.put_i8(source.hilevel);
        b.put_i8(source.lolevel);
        b.put_i8(source.hilevel);
        b.put_i32_le(source.unk16);
        b.put_i16_le(source.unk17);
        b.put_u16_le(source.sample_id);
        b.put_i8(source.ftune);
        b.put_i8(source.ctune);
        b.put_i8(source.rootkey);
        b.put_i8(source.ktps);
        b.put_i8(source.sample_volume);
        b.put_i8(source.sample_pan);
        b.put_i8(source.keygroup_id);
        b.put_u8(source.unk22);
        b.put_u16_le(source.unk23);
        b.put_u16_le(source.unk24);

        b.put_u8(source.envelope);
        b.put_u8(source.envelope_multiplier);
        b.put_u8(source.unk37);
        b.put_u8(source.unk38);
        b.put_u16_le(source.unk39);
        b.put_u16_le(source.unk40);
        b.put_i8(source.attack_volume);
        b.put_i8(source.attack);
        b.put_i8(source.decay);
        b.put_i8(source.sustain);
        b.put_i8(source.hold);
        b.put_i8(source.decay2);
        b.put_i8(source.release);
        b.put_i8(source.unk53);
        debug_assert_eq!(LEN_SPLITS, b.len());
        b.into()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwdlProgram {
    pub id: u16,
    pub prg_volume: i8,
    pub prg_pan: i8,
    pub unk3: u8,
    pub that_f_byte: u8,
    pub unk4: u16,
    pub unk5: u8,
    pub unk7: u8,
    pub unk8: u8,
    pub unk9: u8,
    pub lfos: Vec<SwdlLfoEntry>,
    pub splits: Vec<SwdlSplitEntry>,
    pub(crate) delimiter: u8,
}

impl From<&mut StBytes> for PyResult<SwdlProgram> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 0x10,
            gettext("SWDL file too short (PRG EOF).")
        );
        let id = source.get_u16_le();
        let number_splits = source.get_u16_le();
        let prg_volume = source.get_i8();
        let prg_pan = source.get_i8();
        let unk3 = source.get_u8();
        let that_f_byte = source.get_u8();
        let unk4 = source.get_u16_le();
        let unk5 = source.get_u8();
        let number_lfos = source.get_u8();
        let delimiter: u8 = source.get_u8();
        let unk7 = source.get_u8();
        let unk8 = source.get_u8();
        let unk9 = source.get_u8();
        let lfos = (0..number_lfos)
            .map(|_| <&mut StBytes>::into(source))
            .collect::<PyResult<Vec<SwdlLfoEntry>>>()?;
        source.advance(16); // Delimiter
        let splits = (0..number_splits)
            .map(|_| <&mut StBytes>::into(source))
            .collect::<PyResult<Vec<SwdlSplitEntry>>>()?;
        Ok(SwdlProgram {
            id,
            prg_volume,
            prg_pan,
            unk3,
            that_f_byte,
            unk4,
            unk5,
            unk7,
            unk8,
            unk9,
            lfos,
            splits,
            delimiter,
        })
    }
}

impl From<SwdlProgram> for StBytes {
    fn from(source: SwdlProgram) -> Self {
        let expected_len =
            0x10 + (source.lfos.len() * LEN_LFO) + 0x10 + (source.splits.len() * LEN_SPLITS);
        let mut b = BytesMut::with_capacity(expected_len);
        b.put_u16_le(source.id);
        b.put_u16_le(source.splits.len() as u16);
        b.put_i8(source.prg_volume);
        b.put_i8(source.prg_pan);
        b.put_u8(source.unk3);
        b.put_u8(source.that_f_byte);
        b.put_u16_le(source.unk4);
        b.put_u8(source.unk5);
        b.put_u8(source.lfos.len() as u8);
        b.put_u8(source.delimiter); // TODO: Ok? This is the delimiter.
        b.put_u8(source.unk7);
        b.put_u8(source.unk8);
        b.put_u8(source.unk9);
        b.extend(source.lfos.into_iter().flat_map(|x| StBytes::from(x).0));
        b.extend(repeat(source.delimiter).take(16)); // TODO: Ok? This is the delimiter.
        b.extend(source.splits.into_iter().flat_map(|x| StBytes::from(x).0));
        debug_assert_eq!(expected_len, b.len());
        b.into()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwdlPrgi {
    pub program_table: Vec<Option<SwdlProgram>>,
}

impl SwdlPrgi {
    pub fn from_bytes(source: &mut StBytes, number_slots: u16) -> PyResult<Self> {
        pyr_assert!(
            source.len() >= 16 + (number_slots as usize * 2),
            gettext("SWDL file too short (Prgi EOF).")
        );
        let header = source.copy_to_bytes(4);
        pyr_assert!(PRGI_HEADER == header, gettext("Invalid PRGI/Prgi header."));
        // 0x00, 0x00, 0x15, 0x04, 0x10, 0x00, 0x00, 0x00:
        source.advance(8);
        let len_chunk_data = source.get_u32_le();
        let mut toc = source.clone();
        let program_table = (0..(number_slots))
            .map(|_| {
                let pnt = toc.get_u16_le();
                pyr_assert!(
                    (pnt as u32) < len_chunk_data,
                    gettext("SWDL Prgi length invalid; tried to read past EOF.")
                );
                if pnt > 0 {
                    let mut dst = source.clone();
                    dst.advance(pnt as usize);
                    Ok(Some(<PyResult<SwdlProgram>>::from(&mut dst)?))
                } else {
                    Ok(None)
                }
            })
            .collect::<PyResult<Vec<Option<SwdlProgram>>>>()?;
        source.advance(len_chunk_data as usize);
        Ok(Self { program_table })
    }
}

impl From<SwdlPrgi> for StBytes {
    fn from(source: SwdlPrgi) -> Self {
        let toc_len = source.program_table.len() * 2;
        let mut toc = BytesMut::with_capacity(toc_len);
        let mut content = BytesMut::with_capacity(source.program_table.len() * 64);

        // Padding after TOC
        if toc_len % 16 != 0 {
            content.extend(repeat(0xAA).take(16 - (toc_len % 16)));
        }

        for prg in source.program_table.into_iter() {
            match prg {
                Some(prg) => {
                    toc.put_u16_le((toc_len + content.len()) as u16);
                    content.put(StBytes::from(prg).0)
                }
                None => toc.put_u16_le(0),
            }
        }
        debug_assert_eq!(toc.len(), toc_len);

        let mut data = BytesMut::with_capacity(0x10);
        data.put(&b"prgi\0\0\x15\x04\x10\0\0\0"[..]);
        data.put_u32_le((toc_len + content.len()) as u32);
        data.put(toc);
        debug_assert_eq!(0x10 + toc_len, data.len());
        data.put(content);
        data.into()
    }
}
