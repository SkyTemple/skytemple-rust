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
use crate::encoding::{BufEncoding, BufMutEncoding};
use bytes::{Buf, BufMut, BytesMut};
use encoding::codec::ascii::ASCIIEncoding;
use encoding::{DecoderTrap, EncoderTrap};
use std::fmt::{Display, Formatter};
use std::iter::repeat;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DseFilename(pub String);

impl DseFilename {
    pub fn from_bytes_fixed(source: &mut StBytes, len: usize) -> Self {
        Self(
            source
                .get_fixed_string(ASCIIEncoding, len, DecoderTrap::Ignore)
                .unwrap(),
        )
    }
}

impl<T: AsRef<[u8]> + Buf> From<&mut T> for DseFilename {
    fn from(source: &mut T) -> Self {
        Self(
            source
                .get_c_string(ASCIIEncoding, DecoderTrap::Ignore)
                .unwrap(),
        )
    }
}

impl From<DseFilename> for StBytes {
    fn from(mut source: DseFilename) -> Self {
        if source.0.len() > 0xF {
            source.0.truncate(0xF)
        }
        let mut target = BytesMut::with_capacity(16);
        target
            .put_c_string(&source.0, ASCIIEncoding, EncoderTrap::Ignore)
            .unwrap();
        if target.len() < 2 {
            // the string only contained non-ascii characters.....
            target = BytesMut::with_capacity(16);
            target.put_u8(b'?');
            target.put_u8(0);
        }
        if target.len() < 16 {
            target.extend(repeat(0xFF).take(16 - target.len()))
        }
        target.into()
    }
}

impl Display for DseFilename {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
