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

use crate::python::Py;
#[cfg(feature = "python")]
use crate::python::{
    FromPyObject, IntoPy, PyAny, PyByteArray, PyBytes, PyObject, PyResult, Python,
};
use bytes::buf::IntoIter;
use bytes::{Bytes, BytesMut};
#[cfg(feature = "python")]
use pyo3::types::PyList;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StBytesMut(pub(crate) BytesMut);

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StBytes(pub(crate) Bytes);

/** Export Bytes as bytes */
#[cfg(feature = "python")]
impl IntoPy<PyObject> for StBytes {
    fn into_py(self, py: Python) -> PyObject {
        PyBytes::new(py, &self.0).into()
    }
}

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for StBytes {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if let Ok(bytes) = ob.downcast::<PyBytes>() {
            // TODO: Maybe we could do without copying?
            let data = Vec::from(bytes.as_bytes());
            Ok(Self(Bytes::from(data)))
        } else if let Ok(bytearray) = ob.downcast::<PyByteArray>() {
            // TODO: Maybe we could do without copying?
            let data: Vec<u8>;
            unsafe {
                data = Vec::from(bytearray.as_bytes());
            }
            Ok(Self(Bytes::from(data)))
        } else {
            let data: &PyList = ob.downcast()?;
            Ok(Self(Bytes::from(
                data.into_iter()
                    .map(|x| x.extract::<u8>())
                    .collect::<PyResult<Vec<u8>>>()?,
            )))
        }
    }
}

impl Deref for StBytes {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<u8> for StBytes {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(Bytes::from_iter(iter))
    }
}

impl IntoIterator for StBytes {
    type Item = u8;
    type IntoIter = IntoIter<Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<&[u8]> for StBytes {
    fn from(v: &[u8]) -> Self {
        Self(Bytes::copy_from_slice(v))
    }
}

impl From<Vec<u8>> for StBytes {
    fn from(v: Vec<u8>) -> Self {
        Self(Bytes::from(v))
    }
}

impl From<Bytes> for StBytes {
    fn from(v: Bytes) -> Self {
        Self(v)
    }
}

impl From<BytesMut> for StBytes {
    fn from(v: BytesMut) -> Self {
        Self(v.freeze())
    }
}

impl From<StBytes> for Bytes {
    fn from(v: StBytes) -> Self {
        v.0
    }
}

impl AsRef<[u8]> for StBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/** Export Vec<u8> as bytes */
#[cfg(feature = "python")]
impl IntoPy<PyObject> for StBytesMut {
    fn into_py(self, py: Python) -> PyObject {
        PyBytes::new(py, &self.0).into()
    }
}

impl StBytesMut {
    pub fn freeze(self) -> StBytes {
        StBytes::from(self.0)
    }
}

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for StBytesMut {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if let Ok(bytearray) = ob.downcast::<PyByteArray>() {
            let data: BytesMut;
            unsafe {
                data = BytesMut::from(bytearray.as_bytes());
            }
            Ok(Self(data))
        } else if let Ok(bytes) = ob.downcast::<PyBytes>() {
            let data = BytesMut::from(bytes.as_bytes());
            Ok(Self(data))
        } else {
            let data: &PyList = ob.downcast()?;
            Ok(Self(BytesMut::from(
                &data
                    .into_iter()
                    .map(|x| x.extract::<u8>())
                    .collect::<PyResult<Vec<u8>>>()?[..],
            )))
        }
    }
}

impl Deref for StBytesMut {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StBytesMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<u8> for StBytesMut {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(BytesMut::from_iter(iter))
    }
}

impl IntoIterator for StBytesMut {
    type Item = u8;
    type IntoIter = IntoIter<BytesMut>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<u8>> for StBytesMut {
    fn from(v: Vec<u8>) -> Self {
        Self(BytesMut::from(&v[..]))
    }
}

impl From<&[u8]> for StBytesMut {
    fn from(v: &[u8]) -> Self {
        Self(BytesMut::from(v))
    }
}

impl From<Bytes> for StBytesMut {
    fn from(v: Bytes) -> Self {
        Self(BytesMut::from(&v[..]))
    }
}

impl From<BytesMut> for StBytesMut {
    fn from(v: BytesMut) -> Self {
        Self(v)
    }
}

impl From<StBytesMut> for BytesMut {
    fn from(v: StBytesMut) -> Self {
        v.0
    }
}

impl From<StBytesMut> for Bytes {
    fn from(v: StBytesMut) -> Self {
        v.0.freeze()
    }
}

pub trait AsStBytes {
    fn as_bytes(&self) -> StBytes;
}

impl<T> AsStBytes for Py<T>
where
    Self: Into<StBytes> + Clone,
{
    fn as_bytes(&self) -> StBytes {
        self.clone().into()
    }
}

impl<T> AsStBytes for &T
where
    T: AsStBytes,
{
    fn as_bytes(&self) -> StBytes {
        T::as_bytes(self)
    }
}
