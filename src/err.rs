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

use crate::python::{exceptions, PyErr};
#[cfg(feature = "packed_struct")]
use packed_struct::PackingError;
use std::borrow::Cow;

#[inline]
#[allow(unused)]
pub fn convert_encoding_err(err: Cow<'static, str>) -> PyErr {
    exceptions::PyValueError::new_err(format!("Failed to decode/encode string for PMD2: {}", err))
}

#[inline]
#[allow(unused)]
pub fn convert_io_err(err: std::io::Error) -> PyErr {
    PyErr::from(err)
}

#[cfg(feature = "packed_struct")]
#[inline]
#[allow(unused)]
pub fn convert_packing_err(err: PackingError) -> PyErr {
    exceptions::PyValueError::new_err(format!("Failed packing/unpacking data: {}", err))
}
