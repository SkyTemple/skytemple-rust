use bytes::{Bytes, BytesMut};
use std::ops::{Deref, DerefMut};
use bytes::buf::IntoIter;
use pyo3::types::PyList;
use crate::python::{FromPyObject, IntoPy, PyAny, PyByteArray, PyBytes, PyObject, PyResult, Python};

#[derive(Clone, Default, PartialEq)]
pub struct StBytesMut(pub BytesMut);

#[derive(Clone, Default, PartialEq)]
pub struct StBytes(pub Bytes);

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
            unsafe { data = Vec::from(bytearray.as_bytes()); }
            Ok(Self(Bytes::from(data)))
        } else {
            let data: &PyList = ob.downcast()?;
            Ok(Self(Bytes::from(
                data.into_iter().map(|x| x.extract::<u8>()).collect::<PyResult<Vec<u8>>>()?
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
    fn from_iter<T: IntoIterator<Item=u8>>(iter: T) -> Self {
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
            unsafe { data = BytesMut::from(bytearray.as_bytes()); }
            Ok(Self(data))
        } else if let Ok(bytes) = ob.downcast::<PyBytes>() {
            let data = BytesMut::from(bytes.as_bytes());
            Ok(Self(data))
        } else {
            let data: &PyList = ob.downcast()?;
            Ok(Self(BytesMut::from(
                &data.into_iter().map(|x| x.extract::<u8>()).collect::<PyResult<Vec<u8>>>()?[..]
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
    fn from_iter<T: IntoIterator<Item=u8>>(iter: T) -> Self {
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
