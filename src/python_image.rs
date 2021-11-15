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
use image::{DynamicImage, ImageFormat};
use pyo3::{exceptions, IntoPy, PyObject, Python};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyTuple};
use crate::image::InWrappedImage;
use crate::image::OutWrappedImage;

fn in_from_py<'a>(img: &'a InWrappedImage, py: Python<'a>) -> PyResult<&'a PyBytes> {
    let bytesio = PyModule::import(py, "io")?
        .getattr("BytesIO")?;
    let arr = bytesio
        .getattr("__new__")?
        .call1(PyTuple::new(py, [bytesio]))?;
    let args = PyTuple::new(py, [arr]);
    let kwargs = PyDict::new(py);
    kwargs.set_item("format", "PNG")?;
    img.0.getattr("save")?.call(args, Option::Some(kwargs))?;
    let raw: &PyBytes = arr.getattr("getvalue")?.call0()?.cast_as()?;
    Ok(raw)
}

fn out_to_py<'a>(img: &'a OutWrappedImage, py: Python<'a>) -> PyResult<PyObject> {
    let mut src: Vec<u8> = Vec::new();
    match img.0.write_to(&mut src, ImageFormat::Png) {
        Ok(_) => (),
        Err(e) => return Err(exceptions::PyRuntimeError::new_err(format!("{:?}", e)))
    }
    let bytesio = PyModule::import(py, "io")?.getattr("BytesIO")?;
    let buff = bytesio.getattr("__new__")?.call1(PyTuple::new(py, [bytesio]))?;
    buff.getattr("__init__")?.call1(PyTuple::new(py, [PyBytes::new(py, &*src)]));
    let img = PyModule::import(py, "PIL.Image")?
        .getattr("open")?
        .call1(PyTuple::new(py, [buff]))?;
    return Ok(img.to_object(py));
}

impl IntoPy<PyObject> for OutWrappedImage {
    fn into_py(self, py: Python) -> PyObject {
        match out_to_py(&self, py) {
            Ok(d) => d,
            Err(e) => {
                println!("skytemple-rust: Critical error during image conversion:");
                e.print(py);
                py.None()
            }
        }
    }
}

impl InWrappedImage<'_> {
    pub fn unwrap(&self) -> PyResult<DynamicImage> {
        let mut ret: Option<DynamicImage> = Option::None;
        let mut err: Option<PyErr> = Option::None;
        Python::with_gil(|py| {
            match in_from_py(self, py) {
                Ok(x) => {
                    match image::load_from_memory_with_format(x.as_bytes(), ImageFormat::Png) {
                        Ok(x) => ret = Option::Some(x),
                        Err(e) => err = Option::Some(exceptions::PyRuntimeError::new_err(format!("Internal error converting an image: {:?}", e)))
                    }
                },
                Err(e) => err = Option::Some(e)
            }
        });
        match ret {
            Some(d) => Ok(d),
            None => match err {
                Some(e) => Err(e),
                None => Err(exceptions::PyRuntimeError::new_err("Unexpected image conversion error."))
            }
        }
    }
}
