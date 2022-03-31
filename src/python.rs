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
pub use pyo3::exceptions;
#[cfg(feature = "python")]
pub use pyo3::prelude::*;
#[cfg(feature = "python")]
pub use pyo3::PyErr;
#[cfg(feature = "python")]
pub use pyo3::types::PyBytes;
#[cfg(feature = "python")]
pub use pyo3::types::PyByteArray;
#[cfg(feature = "python")]
pub use pyo3::types::PyType;
#[cfg(feature = "python")]
#[cfg(feature = "image")]
pub use crate::python_image::*;

#[cfg(feature = "python")]
const USER_ERROR_MARK: &str = "_skytemple__user_error";

// The Py::clone_ref method returns a copy of the Py container with Python.
// Without Python, it returns a reference instead.
// See no_python module.
#[cfg(feature = "python")]
pub type PyClonedByRef<T> = Py<T>;

#[cfg(not(feature = "python"))]
pub(crate) use crate::no_python::*;
#[cfg(not(feature = "python"))]
pub use crate::no_python::PyErr;
#[cfg(not(feature = "python"))]
pub use crate::no_python::exceptions;

#[inline]
#[allow(unused)]
// Clonability is required for non-Python use cases.
pub(crate) fn return_option<T>(py: Python, opt: &Option<Py<T>>) -> PyResult<Option<PyClonedByRef<T>>> where T: Clone {
    Ok(opt.as_ref().map(|x| x.clone_ref(py)))
}

/// Creates a PyValueError that is marked as an user error for Python contexts (for error reporting purposes).
#[cfg(feature = "python")]
pub fn create_value_user_error<S: Into<String> + IntoPy<PyObject> + Send + Sync + 'static>(msg: S) -> PyErr {
    let exc = exceptions::PyValueError::new_err(msg);
    Python::with_gil(|py | exc.instance(py).setattr(USER_ERROR_MARK, true).ok());
    exc
}

/// Creates a PyValueError that is marked as an user error for Python contexts (for error reporting purposes).
#[cfg(not(feature = "python"))]
pub fn create_value_user_error<S: Into<String> + Send + Sync + 'static>(msg: S) -> PyErr {
    exceptions::PyValueError::new_err(msg)
}
