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
pub use skytemple_rust_macros::*;

pub type PyResult<T> = Result<T, PyErr>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PyErr {
    type_name: String,
    value: String
}

pub mod exceptions {
    use crate::no_python::PyErr;

    macro_rules! impl_py_exception (
        ($name:ident) => (
            pub struct $name {}
            impl $name {
                pub fn new_err(value: &str) -> PyErr
                {
                    PyErr { type_name: String::from(stringify!($name)), value: String::from(value)}
                }
            }
        );
    );

    impl_py_exception!(PyValueError);
}
