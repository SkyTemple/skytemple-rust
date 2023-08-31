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

#[cfg(feature = "python")]
#[cfg(feature = "image")]
pub use crate::python_image::*;
#[cfg(feature = "python")]
pub use pyo3::exceptions;
#[cfg(feature = "python")]
pub use pyo3::prelude::*;
#[cfg(feature = "python")]
pub use pyo3::types::PyByteArray;
#[cfg(feature = "python")]
pub use pyo3::types::PyBytes;
#[cfg(feature = "python")]
pub use pyo3::types::PyType;
#[cfg(feature = "python")]
pub use pyo3::PyErr;

#[cfg(feature = "python")]
const USER_ERROR_MARK: &str = "_skytemple__user_error";

#[cfg(not(feature = "python"))]
pub use crate::no_python::exceptions;
#[cfg(not(feature = "python"))]
pub use crate::no_python::PyErr;
#[cfg(not(feature = "python"))]
pub(crate) use crate::no_python::*;

/// Creates a PyValueError that is marked as an user error for Python contexts (for error reporting purposes).
#[cfg(feature = "python")]
pub fn create_value_user_error<S: Into<String> + IntoPy<PyObject> + Send + Sync + 'static>(
    msg: S,
) -> PyErr {
    let exc = exceptions::PyValueError::new_err(msg);
    Python::with_gil(|py| exc.value(py).setattr(USER_ERROR_MARK, true).ok());
    exc
}

/// Creates a PyValueError that is marked as an user error for Python contexts (for error reporting purposes).
#[cfg(not(feature = "python"))]
pub fn create_value_user_error<S: Into<String> + Send + Sync + 'static>(msg: S) -> PyErr {
    exceptions::PyValueError::new_err(msg)
}

#[cfg(feature = "python")]
#[derive(FromPyObject)]
pub enum SliceOrInt<'a> {
    Slice(&'a pyo3::types::PySlice),
    Int(isize),
}
