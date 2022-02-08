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

#[cfg(feature = "strings")]
pub mod pmd2_encoder;

use std::cmp::Ordering;
use bytes::{Buf, BufMut, Bytes};
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use crate::err::convert_encoding_err;
use crate::python::{exceptions, PyResult};

/// Extensions for Buf for dealing with encoded strings.
pub trait BufEncoding {
    fn get_fixed_string<E>(&mut self, enc: E, len: usize, trap: DecoderTrap) -> PyResult<String> where E: Encoding;
    fn get_fixed_string_or_null<E>(&mut self, enc: E, len: usize, trap: DecoderTrap) -> PyResult<Option<String>> where E: Encoding;
}

pub trait BufMutEncoding {
    fn put_fixed_string<E>(&mut self, string: &str, enc: E, len: usize, trap: EncoderTrap) -> PyResult<()> where E: Encoding;
}

impl<T> BufEncoding for T where T: Buf {
    fn get_fixed_string<E>(&mut self, enc: E, len: usize, trap: DecoderTrap) -> PyResult<String> where E: Encoding {
        Ok(self.get_fixed_string_or_null(enc, len, trap)?.unwrap_or_else(|| "".to_string()))
    }

    fn get_fixed_string_or_null<E>(&mut self, enc: E, len: usize, trap: DecoderTrap) -> PyResult<Option<String>> where E: Encoding {
        let c: Bytes = self.copy_to_bytes(len).into_iter().take_while(|x| *x != 0).collect();
        if c.is_empty() {
            return Ok(None);
        }
        enc.decode(&c, trap).map(Some).map_err(convert_encoding_err)
    }
}

impl<T> BufMutEncoding for T where T: BufMut {
    fn put_fixed_string<E>(&mut self, string: &str, enc: E, len: usize, trap: EncoderTrap) -> PyResult<()> where E: Encoding {
        let mut target = Vec::with_capacity(len);
        enc.encode_to(string, trap, &mut target).map_err(convert_encoding_err)?;
        match target.len().cmp(&len) {
            Ordering::Less => {
                target.resize(len, 0)
            }
            Ordering::Greater => {
                return Err(exceptions::PyValueError::new_err(format!(
                    "The string '{}' does not fit into {} bytes.", string, len
                )))
            }
            _ => {}
        }
        self.put(&target[..]);
        Ok(())
    }
}