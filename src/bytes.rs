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

use std::ops::{Deref, DerefMut};

use bytes::buf::IntoIter;
use bytes::{Bytes, BytesMut};
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes, PyList};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StBytesMut(pub(crate) BytesMut);

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StBytes(pub(crate) Bytes);

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct StU8List(pub(crate) Vec<u8>);

/// Export Bytes as bytes
impl<'py> IntoPyObject<'py> for StBytes {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, &self.0))
    }
}

impl<'source> FromPyObject<'source> for StBytes {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
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
            let data: &Bound<PyList> = ob.downcast()?;
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

impl From<StU8List> for StBytes {
    fn from(v: StU8List) -> Self {
        Self(Bytes::from(v.0))
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

/// Export BytesMut as bytes
impl<'py> IntoPyObject<'py> for StBytesMut {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, &self.0))
    }
}

impl StBytesMut {
    pub fn freeze(self) -> StBytes {
        StBytes::from(self.0)
    }
}

impl<'source> FromPyObject<'source> for StBytesMut {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
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
            let data: &Bound<PyList> = ob.downcast()?;
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

pub trait AsStBytesPyRef {
    fn as_bytes_pyref(&self, py: Python) -> StBytes;
}

impl<T> AsStBytesPyRef for Py<T>
where
    Self: Into<StBytes>,
{
    fn as_bytes_pyref(&self, py: Python) -> StBytes {
        self.clone_ref(py).into()
    }
}

/// Export Vec<u8> as list
impl<'py> IntoPyObject<'py> for StU8List {
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyList::new(py, &self.0)
    }
}

impl<'source> FromPyObject<'source> for StU8List {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(bytes) = ob.downcast::<PyBytes>() {
            // TODO: Maybe we could do without copying?
            let data = Vec::from(bytes.as_bytes());
            Ok(Self(data))
        } else if let Ok(bytearray) = ob.downcast::<PyByteArray>() {
            // TODO: Maybe we could do without copying?
            let data: Vec<u8>;
            unsafe {
                data = Vec::from(bytearray.as_bytes());
            }
            Ok(Self(data))
        } else {
            let data: &Bound<PyList> = ob.downcast()?;
            Ok(Self(
                data.into_iter()
                    .map(|x| x.extract::<u8>())
                    .collect::<PyResult<Vec<u8>>>()?,
            ))
        }
    }
}

impl Deref for StU8List {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StU8List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<u8> for StU8List {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl IntoIterator for StU8List {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<&[u8]> for StU8List {
    fn from(v: &[u8]) -> Self {
        Self(Vec::from(v))
    }
}

impl From<Vec<u8>> for StU8List {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<StBytes> for StU8List {
    fn from(v: StBytes) -> Self {
        Self(v.to_vec())
    }
}

impl From<Bytes> for StU8List {
    fn from(v: Bytes) -> Self {
        Self(v.to_vec())
    }
}

impl From<BytesMut> for StU8List {
    fn from(v: BytesMut) -> Self {
        Self(v.to_vec())
    }
}

impl AsRef<[u8]> for StU8List {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
