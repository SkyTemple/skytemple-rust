/*
 * Copyright 2021-2024 Capypara and the SkyTemple Contributors
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
use pyo3::prelude::*;
use pyo3::PyErr;
use pyo3::{exceptions, PyErrArguments};

const USER_ERROR_MARK: &str = "_skytemple__user_error";

/// Creates a PyValueError that is marked as an user error for Python contexts (for error reporting purposes).
pub fn create_value_user_error<S: PyErrArguments + 'static>(msg: S) -> PyErr {
    let exc = exceptions::PyValueError::new_err(msg);
    Python::with_gil(|py| exc.value(py).setattr(USER_ERROR_MARK, true).ok());
    exc
}

#[derive(FromPyObject)]
pub enum SliceOrInt<'a> {
    Slice(Bound<'a, pyo3::types::PySlice>),
    Int(isize),
}
