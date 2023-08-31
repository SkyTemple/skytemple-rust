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
/** Definitions of a Pyo3 types without Python or Pyo3 */
pub use skytemple_rust_macros::*;
use std::cell::{Ref, RefCell, RefMut};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::num::TryFromIntError;
use std::ops::Deref;
use std::sync::Arc;

pub(crate) type PyResult<T> = Result<T, PyErr>;

/// Dummy. Just pass this when you don't use Python.
#[derive(Copy, Clone)]
pub struct Python;

impl Python {
    // Dummy. This would acquire the Python GIL if Python were used.
    pub fn with_gil<F, R>(f: F) -> R
    where
        F: FnOnce(Python) -> R,
    {
        f(Python)
    }
}

/// Dummy. Just pass this when you don't use Python.
#[derive(Copy, Clone)]
pub struct PyType;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PyErr {
    type_name: String,
    value: String,
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
            value,
        }
    }
}

impl From<TryFromIntError> for PyErr {
    fn from(err: TryFromIntError) -> Self {
        let value = err.to_string();
        Self {
            type_name: "FromRust".to_string(),
            value,
        }
    }
}

impl Error for PyErr {}

pub trait PyWrapable {}

/// Wrapper around Vec<u8> used to tell Pyo3 that this is a bytes object for Python
#[derive(Clone)]
pub struct PyBytes(pub Vec<u8>);

impl PyWrapable for PyBytes {}
impl PyBytes {
    pub fn new(_: Python, from: &[u8]) -> Self {
        Self(from.to_vec())
    }
}

pub type PyRef<'a, T> = &'a T;
pub type PyRefMut<'a, T> = &'a mut T;

/// This would normally be a reference to an object on the Python heap. Without Python it's
/// basically an Arc<RefCell<T>>.
/// If not using Python, extract always clones, clone clones the inner Arc,
/// and borrow and borrow_mut return normal Rust references to T instead.
#[derive(Clone)]
pub struct Py<T>(Arc<RefCell<T>>);
impl<T> Py<T> {
    pub fn new(_: Python, obj: T) -> PyResult<Self> {
        Ok(Self(Arc::new(RefCell::new(obj))))
    }
    pub fn borrow(&self, _: Python) -> Ref<T> {
        self.0.borrow()
    }
    pub fn borrow_mut(&mut self, _: Python) -> RefMut<T> {
        self.0.borrow_mut()
    }
}

impl<T> Py<T>
where
    T: Clone,
{
    pub fn extract(&self, _: Python) -> PyResult<T> {
        Ok(T::clone(self.0.borrow().deref()))
    }
}

impl<T> Py<T>
where
    T: PyWrapable,
{
    pub fn from(obj: T) -> Self {
        Self(Arc::new(RefCell::new(obj)))
    }
}

impl<T> PartialEq for Py<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Debug for Py<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
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
                    PyErr { type_name: String::from(stringify!($name)), value: value.into()}
                }
            }
        );
    );

    impl_py_exception!(PyValueError);
    impl_py_exception!(PyRuntimeError);
    impl_py_exception!(PyAssertionError);
    impl_py_exception!(PyIndexError);
}
