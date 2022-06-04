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

const KGRP_HEADER: &[u8] = b"kgrp";
const KEYGROUP_LEN: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwdlKeygroup {
    pub id: u16,
    pub poly: i8,
    pub priority: u8,
    pub vclow: u8,
    pub vchigh: u8,
    pub unk50: u8,
    pub unk51: u8,
}

impl From<&mut StBytes> for PyResult<SwdlKeygroup> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= KEYGROUP_LEN,
            gettext("SWDL file too short (Keygroup EOF).")
        );
        Ok(SwdlKeygroup {
            id: source.get_u16_le(),
            poly: source.get_i8(),
            priority: source.get_u8(),
            vclow: source.get_u8(),
            vchigh: source.get_u8(),
            unk50: source.get_u8(),
            unk51: source.get_u8(),
        })
    }
}

impl From<SwdlKeygroup> for StBytes {
    fn from(source: SwdlKeygroup) -> Self {
        let mut b = BytesMut::with_capacity(KEYGROUP_LEN);
        b.put_u16_le(source.id);
        b.put_i8(source.poly);
        b.put_u8(source.priority);
        b.put_u8(source.vclow);
        b.put_u8(source.vchigh);
        b.put_u8(source.unk50);
        b.put_u8(source.unk51);
        b.into()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwdlKgrp {
    pub keygroups: Vec<SwdlKeygroup>,
}

impl From<&mut StBytes> for PyResult<SwdlKgrp> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 16,
            gettext("SWDL file too short (Kgrp EOF).")
        );
        let header = source.copy_to_bytes(4);
        pyr_assert!(KGRP_HEADER == header, gettext("Invalid SWDL/Kgrp header."));
        // 0x00, 0x00, 0x15, 0x04, 0x10, 0x00, 0x00, 0x00:
        source.advance(8);
        let len_chunk_data = source.get_u32_le() as usize;
        pyr_assert!(
            source.len() >= len_chunk_data,
            gettext("SWDL file too short (Kgrp EOF).")
        );

        let number_slots = len_chunk_data / KEYGROUP_LEN; // TODO: Is this the way to do it?

        let keygroups = (0..number_slots)
            .map(|_| source.into())
            .collect::<PyResult<Vec<SwdlKeygroup>>>()?;
        Ok(SwdlKgrp { keygroups })
    }
}

impl From<SwdlKgrp> for StBytes {
    fn from(source: SwdlKgrp) -> Self {
        let mut content = source
            .keygroups
            .into_iter()
            .flat_map(StBytes::from)
            .collect::<BytesMut>();

        let len_content = content.len();
        if len_content % 16 != 0 {
            // TODO: Unknown what this magic value means
            content.put(&[0xc7, 0xc8, 0x40, 0x00, 0xd0, 0x11, 0xa0, 0x04][..]);
        }

        let mut data = BytesMut::with_capacity(0x10);
        data.put(&b"kgrp\0\0\x15\x04\x10\0\0\0"[..]);
        data.put_u32_le(len_content as u32);
        data.put(content);
        data.into()
    }
}
