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

const PCMD_HEADER: &[u8] = b"pcmd";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwdlPcmd {
    pub chunk_data: StBytes,
}

impl From<&mut StBytes> for PyResult<SwdlPcmd> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 16,
            gettext("SWDL file too short (Pcmd EOF).")
        );
        let header = source.copy_to_bytes(4);
        pyr_assert!(PCMD_HEADER == header, gettext("Invalid SWDL/Pcmd header."));
        // 0x00, 0x00, 0x15, 0x04, 0x10, 0x00, 0x00, 0x00:
        source.advance(8);
        let len_chunk_data = source.get_u32_le() as usize;
        pyr_assert!(
            source.len() >= len_chunk_data,
            gettext("SWDL file too short (Pcmd EOF).")
        );
        let chunk_data = source.copy_to_bytes(len_chunk_data);
        Ok(SwdlPcmd {
            chunk_data: StBytes(chunk_data),
        })
    }
}

impl From<SwdlPcmd> for StBytes {
    fn from(source: SwdlPcmd) -> Self {
        let mut padding = if source.chunk_data.len() % 16 != 0 {
            // TODO: Unknown what this magic value means
            BytesMut::from(&[0xb4, 0x03, 0, 0, 0x68, 0x01, 0x51, 0x04][..])
        } else {
            BytesMut::new()
        };
        if (source.chunk_data.len() + padding.len()) % 16 != 0 {
            // TODO: is this ok???
            padding.extend(repeat(0).take(16 - ((source.chunk_data.len() + padding.len()) % 16)))
        }
        let mut data = BytesMut::with_capacity(0x10 + source.chunk_data.len() + padding.len());
        data.put(&b"pcmd\0\0\x15\x04\x10\0\0\0"[..]);
        data.put_u32_le((source.chunk_data.len()) as u32);
        data.put(source.chunk_data.0);
        data.put(padding);
        data.into()
    }
}
