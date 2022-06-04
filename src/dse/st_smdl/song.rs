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

const SONG_HEADER: &[u8] = b"song";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
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
    initial_track_count: u8,
}

impl SmdlSong {
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)] // if python is not enabled.
    pub(crate) fn new(
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u16,
        tpqn: u16,
        unk5: u16,
        nbchans: u8,
        unk6: u32,
        unk7: u32,
        unk8: u32,
        unk9: u32,
        unk10: u16,
        unk11: u16,
        unk12: u32,
    ) -> Self {
        SmdlSong {
            unk1,
            unk2,
            unk3,
            unk4,
            tpqn,
            unk5,
            nbchans,
            unk6,
            unk7,
            unk8,
            unk9,
            unk10,
            unk11,
            unk12,
            initial_track_count: 0,
        }
    }

    pub(crate) fn get_initial_track_count(&self) -> usize {
        self.initial_track_count as usize
    }
}

impl From<&mut StBytes> for PyResult<SmdlSong> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 64,
            gettext("SMDL file too short (Song EOF).")
        );
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
            unk1,
            unk2,
            unk3,
            unk4,
            tpqn,
            unk5,
            nbchans,
            unk6,
            unk7,
            unk8,
            unk9,
            unk10,
            unk11,
            unk12,
            initial_track_count,
        })
    }
}

impl SmdlSong {
    pub fn to_bytes(&self, number_tracks: u8) -> StBytes {
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
