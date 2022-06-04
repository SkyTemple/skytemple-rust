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

const EOC_HEADER: &[u8] = b"eoc\x20";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SmdlEoc {
    pub param1: u32,
    pub param2: u32,
}

impl From<&mut StBytes> for PyResult<SmdlEoc> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 16,
            gettext("SMDL file too short (EOC EOF).")
        );
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
