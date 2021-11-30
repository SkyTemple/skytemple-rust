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
/** Definitions of a Pyo3 types without Python or Pyo3 */
extern crate skytemple_rust_macros;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
pub use skytemple_rust_macros::*;

pub type PyResult<T> = Result<T, PyErr>;

/// Dummy. Just pass this when you don't use Python.
#[derive(Copy, Clone)]
pub struct Python;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PyErr {
    type_name: String,
    value: String,
    rust_source: Option<Box<dyn Error>>
}

impl Display for PyErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.type_name, self.value)
    }
}

impl From<io::Error> for PyErr {
    fn from(err: io::Error) -> Self {
        let value = err.to_string();
        Self {
            type_name: "FromRust".to_string(),
            rust_source: Some(Box::new(err)),
            value
        }
    }
}

impl Error for PyErr {}

pub trait PyWrapable {}

/// Wrapper around Vec<u8> used to tell Pyo3 that this is a bytes object for Python
#[derive(Clone)]
pub struct PyBytes(Vec<u8>);

impl PyWrapable for PyBytes {}
impl PyBytes {
    pub fn new(_: Python, from: &[u8]) -> Self {
        Self(from.to_vec())
    }
}

/// This would normally be a reference to an object on the Python heap.
/// If not using Python, this is a container that clones(!) instead.
///
/// TODO: If we could somehow turn this into a "reference generator" that'd be great.
///
/// NOTE FOR CONTRIBUTORS:
///   Make sure your implementations are compatible with this restricted version of Py.
#[derive(Clone)]
pub struct Py<T>(T) where T: Clone;
impl<T> Py<T> where T: Clone {
    pub fn new(_: Python, obj: T) -> PyResult<Self> {
        Ok(Self(obj))
    }
    pub fn extract<U>(&self, _: Python) -> PyResult<T> {  // where T: U (!!)
        Ok(self.0.clone())
    }
    pub fn clone_ref(&self, _: Python) -> Self {
        self.clone()
    }
}
impl <T> Py<T> where T: PyWrapable + Clone {
    pub fn from(obj: T) -> Self {
        Self(obj)
    }
}

pub mod exceptions {
    use crate::no_python::PyErr;

    macro_rules! impl_py_exception (
        ($name:ident) => (
            pub struct $name {}
            impl $name {
                pub fn new_err<S>(value: S) -> PyErr where S: Into<String>
                {
                    PyErr { type_name: String::from(stringify!($name)), value: value.into(), rust_source: None}
                }
            }
        );
    );

    impl_py_exception!(PyValueError);
    impl_py_exception!(PyRuntimeError);
}
