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
#![cfg_attr(not(feature = "python"), allow(unused_macros))]

macro_rules! pyr_assert {
    ($cond:expr $(,)?) => {{
        if !$cond {
            return Err(crate::python::exceptions::PyAssertionError::new_err(
                format!("{} [{}:{}]", stringify!($cond), file!(), line!()),
            ));
        }
    }};
    ($cond:expr, $msg:expr) => {{
        if !$cond {
            return Err(crate::python::exceptions::PyAssertionError::new_err(
                format!("{} | {} [{}:{}]", $msg, stringify!($cond), file!(), line!()),
            ));
        }
    }};
    ($cond:expr, $msg:expr, $exc:path) => {{
        if !$cond {
            return Err(<$exc>::new_err(format!(
                "{} | {} [{}:{}]",
                $msg,
                stringify!($cond),
                file!(),
                line!()
            )));
        }
    }};
}

macro_rules! static_assert_size {
    ($ty:ty, $size:expr) => {
        const _: [(); $size] = [(); ::std::mem::size_of::<$ty>()];
    };
}

/// Implements "generic" Vec-wrappers that implement Python's MutableSequence protocol.
macro_rules! impl_pylist {
    ($module:expr, $name:expr, $itemty:ty) => {
        __do_impl_pylist!(
            $module,
            $name,
            $itemty,
            {
                pub fn index(&self, value: PyObject, py: Python) -> PyResult<usize> {
                    if let Ok(value) = value.extract::<$itemty>(py) {
                        if let Some(idx) = self.0.iter().position(|x| {
                            x.call_method1(
                                py,
                                "__eq__",
                                ::pyo3::types::PyTuple::new(py, [value.clone()]),
                            )
                            .and_then(|x| x.is_true(py))
                            .unwrap_or_default()
                        }) {
                            Ok(idx)
                        } else {
                            Err(exceptions::PyValueError::new_err("not in list"))
                        }
                    } else {
                        Err(exceptions::PyValueError::new_err("not in list"))
                    }
                }
                pub fn count(&self, value: PyObject, py: Python) -> usize {
                    if let Ok(value) = value.extract::<$itemty>(py) {
                        self.0
                            .iter()
                            .filter(|x| {
                                x.call_method1(
                                    py,
                                    "__eq__",
                                    ::pyo3::types::PyTuple::new(py, [value.clone()]),
                                )
                                .and_then(|x| x.is_true(py))
                                .unwrap_or_default()
                            })
                            .count()
                    } else {
                        0
                    }
                }
                pub fn remove(&mut self, value: PyObject, py: Python) -> PyResult<()> {
                    if let Ok(value) = value.extract::<$itemty>(py) {
                        if let Some(idx) = self.0.iter().position(|x| {
                            x.call_method1(
                                py,
                                "__eq__",
                                ::pyo3::types::PyTuple::new(py, [value.clone()]),
                            )
                            .and_then(|x| x.is_true(py))
                            .unwrap_or_default()
                        }) {
                            self.0.remove(idx);
                            Ok(())
                        } else {
                            Err(exceptions::PyValueError::new_err("not in list"))
                        }
                    } else {
                        Err(exceptions::PyValueError::new_err("not in list"))
                    }
                }
            },
            {
                fn eq(&self, other: &Self) -> bool {
                    Python::with_gil(|py| {
                        if self.0.len() != other.0.len() {
                            false
                        } else {
                            self.0
                                .iter()
                                .zip(other.0.iter())
                                .all(|(a, b)| a.borrow(py).deref() == b.borrow(py).deref())
                        }
                    })
                }
            }
        );
    };
}

macro_rules! impl_pylist_primitive {
    ($module:expr, $name:expr, $itemty:ty) => {
        __do_impl_pylist!(
            $module,
            $name,
            $itemty,
            {
                pub fn index(&self, value: PyObject, py: Python) -> PyResult<usize> {
                    if let Ok(value) = value.extract::<$itemty>(py) {
                        if let Some(idx) = self.0.iter().position(|x| *x == value) {
                            Ok(idx)
                        } else {
                            Err(exceptions::PyValueError::new_err("not in list"))
                        }
                    } else {
                        Err(exceptions::PyValueError::new_err("not in list"))
                    }
                }
                pub fn count(&self, value: PyObject, py: Python) -> usize {
                    if let Ok(value) = value.extract::<$itemty>(py) {
                        self.0.iter().filter(|x| **x == value).count()
                    } else {
                        0
                    }
                }
                pub fn remove(&mut self, value: PyObject, py: Python) -> PyResult<()> {
                    if let Ok(value) = value.extract::<$itemty>(py) {
                        if let Some(idx) = self.0.iter().position(|x| *x == value) {
                            self.0.remove(idx);
                            Ok(())
                        } else {
                            Err(exceptions::PyValueError::new_err("not in list"))
                        }
                    } else {
                        Err(exceptions::PyValueError::new_err("not in list"))
                    }
                }
            },
            {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }
        );
    };
}

macro_rules! __do_impl_pylist {
    ($module:expr, $name:expr, $itemty:ty, { $($implDetails:tt)* }, { $($pyEqImpl:tt)* }) => {
        ::paste::paste! {
            #[pyclass(module = $module)]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[derive(Clone, Debug)]
            pub struct $name(pub Vec<$itemty>);

            #[cfg(feature = "python")]
            #[pymethods]
            impl $name {
                pub fn __iter__(&mut self) -> [<$name Iterator>] {
                    [<$name Iterator>]::new(self.0.clone().into_iter())
                }
                pub fn __getitem__(&self, idx: SliceOrInt, py: Python) -> PyResult<PyObject> {
                    match idx {
                        SliceOrInt::Slice(sl) => {
                            let pylist = ::pyo3::types::PyList::new(py, self.0.iter().cloned());
                            pylist
                                .call_method1("__getitem__", ::pyo3::types::PyTuple::new(py, [sl]))
                                .map(|v| v.into_py(py))
                        }
                        SliceOrInt::Int(idx) => {
                            if idx >= 0 && idx as usize <= self.0.len() {
                                Ok(self.0[idx as usize].clone().into_py(py))
                            } else {
                                Err(exceptions::PyIndexError::new_err("list index out of range"))
                            }
                        }
                    }
                }
                pub fn __setitem__(&mut self, idx: SliceOrInt, o: PyObject, py: Python) -> PyResult<()> {
                    match idx {
                        SliceOrInt::Slice(sl) => {
                            let pylist = ::pyo3::types::PyList::new(py, self.0.iter().cloned());
                            pylist.call_method1("__setitem__", ::pyo3::types::PyTuple::new(py, [sl.into_py(py), o]))?;
                            self.0 = pylist
                                .into_iter()
                                .map(|o| o.extract())
                                .collect::<PyResult<Vec<$itemty>>>()?;
                            Ok(())
                        }
                        SliceOrInt::Int(idx) => {
                            if idx >= 0 && idx as usize <= self.0.len() {
                                self.0[idx as usize] = o.extract(py)?;
                                Ok(())
                            } else {
                                Err(exceptions::PyIndexError::new_err("list index out of range"))
                            }
                        }
                    }
                }
                pub fn __delitem__(&mut self, idx: SliceOrInt, py: Python) -> PyResult<()> {
                    match idx {
                        SliceOrInt::Slice(sl) => {
                            let pylist = ::pyo3::types::PyList::new(py, self.0.iter().cloned());
                            pylist.call_method1("__delitem__", ::pyo3::types::PyTuple::new(py, [sl]))?;
                            self.0 = pylist
                                .into_iter()
                                .map(|o| o.extract())
                                .collect::<PyResult<Vec<$itemty>>>()?;
                            Ok(())
                        }
                        SliceOrInt::Int(idx) => {
                            if idx >= 0 && idx as usize <= self.0.len() {
                                self.0.remove(idx as usize);
                                Ok(())
                            } else {
                                Err(exceptions::PyIndexError::new_err("list index out of range"))
                            }
                        }
                    }
                }
                pub fn __len__(&self) -> usize {
                    self.0.len()
                }
                pub fn insert(&mut self, index: usize, value: $itemty) {
                    self.0.insert(index, value)
                }
                pub fn append(&mut self, value: $itemty) {
                    self.0.push(value)
                }
                pub fn clear(&mut self) {
                    self.0.clear()
                }
                pub fn extend(&mut self, _value: PyObject) -> PyResult<()> {
                    Err(exceptions::PyNotImplementedError::new_err("Not supported."))
                }
                #[pyo3(signature = (idx = 0))]
                pub fn pop(&mut self, idx: isize) -> PyResult<$itemty> {
                    if idx == 0 {
                        if !self.0.is_empty() {
                            Ok(self.0.pop().unwrap())
                        } else {
                            Err(exceptions::PyIndexError::new_err("pop from empty list"))
                        }
                    } else if idx >= 0 && idx as usize <= self.0.len() {
                        Ok(self.0.remove(idx as usize))
                    } else {
                        Err(exceptions::PyIndexError::new_err("pop index out of range"))
                    }
                }
                pub fn __iadd__(&mut self, value: PyObject) -> PyResult<()> {
                    self.extend(value)
                }
                fn __richcmp__(&self, other: PyRef<Self>, op: ::pyo3::basic::CompareOp) -> Py<PyAny> {
                    let py = other.py();
                    match op {
                        ::pyo3::basic::CompareOp::Eq => (self == other.deref()).into_py(py),
                        ::pyo3::basic::CompareOp::Ne => (self != other.deref()).into_py(py),
                        _ => py.NotImplemented(),
                    }
                }
                $($implDetails)*
            }

            impl PartialEq for $name {
                $($pyEqImpl)*
            }

            impl Eq for $name {}

            impl FromIterator<$itemty> for $name {
                fn from_iter<T: IntoIterator<Item = $itemty>>(iter: T) -> Self {
                    Self(Vec::from_iter(iter))
                }
            }

            #[pyclass(module = $module)]
            pub struct [<$name Iterator>] {
                iter: ::std::vec::IntoIter<$itemty>,
            }

            impl [<$name Iterator>] {
                pub fn new(iter: ::std::vec::IntoIter<$itemty>) -> Self {
                    Self { iter }
                }
            }

            #[pymethods]
            impl [<$name Iterator>] {
                pub fn __next__(&mut self) -> Option<$itemty> {
                    self.iter.next()
                }
            }

        }
    };
}
