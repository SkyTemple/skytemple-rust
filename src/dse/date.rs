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
use bytes::{Buf, BufMut, BytesMut};
use time::OffsetDateTime;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DseDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub centisecond: u8,
}

impl DseDate {
    pub fn new(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        centisecond: u8,
    ) -> Self {
        DseDate {
            year,
            month,
            day,
            hour,
            minute,
            second,
            centisecond,
        }
    }
    pub fn now() -> Self {
        let now = OffsetDateTime::now_utc();
        DseDate {
            year: now.year() as u16,
            month: now.month() as u8,
            day: now.day(),
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
            centisecond: 0,
        }
    }
}

impl From<&mut StBytes> for DseDate {
    fn from(source: &mut StBytes) -> Self {
        Self::new(
            source.get_u16_le(),
            source.get_u8(),
            source.get_u8(),
            source.get_u8(),
            source.get_u8(),
            source.get_u8(),
            source.get_u8(),
        )
    }
}

impl From<DseDate> for StBytes {
    fn from(source: DseDate) -> Self {
        let mut buff = BytesMut::with_capacity(8);
        buff.put_u16_le(source.year);
        buff.put_u8(source.month);
        buff.put_u8(source.day);
        buff.put_u8(source.hour);
        buff.put_u8(source.minute);
        buff.put_u8(source.second);
        buff.put_u8(source.centisecond);
        buff.into()
    }
}
