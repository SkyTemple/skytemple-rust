/// This crate converts our image models from/into PIL images for Python.
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
use bytes::Bytes;
use pyo3::{exceptions, IntoPy, PyObject, Python};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use crate::image::{InWrappedImage, Raster};
use crate::image::IndexedImage;

fn in_from_py(img: InWrappedImage, py: Python) -> PyResult<(Vec<u8>, Vec<u8>, usize, usize)> {
    if img.0.getattr(py, "mode")?.extract::<&str>(py)? != "P" {
        return Err(exceptions::PyValueError::new_err("Expected an indexed image."))
    }
    let args = PyTuple::new(py, ["raw", "P"]);
    let bytes: Vec<u8> = img.0.getattr(py, "tobytes")?.call1(py, args)?.extract(py)?;
    let pal: Vec<u8> = img.0.getattr(py, "palette")?.getattr(py, "palette")?.extract(py)?;
    Ok((bytes, pal, img.0.getattr(py, "width")?.extract(py)?, img.0.getattr(py, "height")?.extract(py)?))
}

fn out_to_py(img: IndexedImage, py: Python) -> PyResult<PyObject> {
    let args = PyTuple::new(py, [
        "P".into_py(py), PyTuple::new(py, [img.0.1, img.0.2]).into_py(py), img.0.0.into_py(py),
        "raw".into_py(py), "P".into_py(py), 0.into_py(py), 1.into_py(py)
    ]);
    let out_img = PyModule::import(py, "PIL.Image")?
        .getattr("frombuffer")?
        .call1(args)?;
    let args = PyTuple::new(py, [img.1.into_py(py)]);
    out_img.getattr("putpalette")?.call1(args)?;
    Ok(out_img.to_object(py))
}

impl IntoPy<PyObject> for IndexedImage {
    fn into_py(self, py: Python) -> PyObject {
        match out_to_py(self, py) {
            Ok(d) => d,
            Err(e) => {
                println!("skytemple-rust: Critical error during image conversion:");
                e.print(py);
                py.None()
            }
        }
    }
}

impl InWrappedImage {
    pub fn extract(self, py: Python) -> PyResult<IndexedImage> {
        match in_from_py(self, py) {
            Ok((raster, pal, width, height)) => {
                Ok(IndexedImage(Raster(
                    Bytes::from(raster), width, height),
                    Bytes::from(pal)
                ))
            },
            Err(e) => Err(e)
        }
    }
}

