/*
 * Copyright 2021-2021 Parakoopa and the SkyTemple Contributors
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
pub struct StBytes(pub Vec<u8>);

use bytes::{Bytes, BytesMut};
#[cfg(feature = "python")]
pub use pyo3::exceptions;
#[cfg(feature = "python")]
pub use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyBytes;
#[cfg(feature = "python")]
pub use pyo3::types::PyType;
#[cfg(feature = "python")]
pub use crate::python_image::*;

/** Export Vec<u8> as bytes */
#[cfg(feature = "python")]
impl IntoPy<PyObject> for StBytes {
    fn into_py(self, py: Python) -> PyObject {
        PyBytes::new(py, &self.0).into()
    }
}
impl From<Vec<u8>> for StBytes {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}
impl From<Bytes> for StBytes {
    fn from(v: Bytes) -> Self {
        Self(v.to_vec())
    }
}
impl From<BytesMut> for StBytes {
    fn from(v: BytesMut) -> Self {
        Self(v.to_vec())
    }
}
impl From<StBytes> for Vec<u8> {
    fn from(v: StBytes) -> Self {
        v.0
    }
}

#[cfg(not(feature = "python"))]
pub use crate::no_python::*;
