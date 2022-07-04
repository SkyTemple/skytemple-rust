use crate::bytes::StBytesMut;
use crate::image::InIndexedImage;
use crate::image::IndexedImage;
/// This crate converts our image models from/into PIL images for Python.
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
use log::error;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyIterator, PyTuple};
use pyo3::{exceptions, IntoPy, PyObject, Python};

pub fn in_from_py<'py, T>(
    img: T,
    py: Python<'py>,
) -> PyResult<(StBytesMut, StBytesMut, usize, usize)>
where
    T: InIndexedImage<'py>,
{
    let mut iimg = img.unwrap_py();
    if iimg.getattr(py, "mode")?.extract::<&str>(py)? == "P" {
        if T::MAX_COLORS == 16 {
            // Quantize
            // TODO: This seems to be lossy atm... Just change the mode for now
            //iimg = pil_simple_quant(py, iimg)?;
        }
        // Otherwise we don't support checking further..., we will assume all goes well.
        // TODO: Maybe support in the future via (automatic) Tilequant?
    } else if T::MAX_COLORS == 16 {
        // Quantize
        iimg = pil_simple_quant(py, iimg, T::CAN_HAVE_TRANSPARENCY)?;
    } else {
        // Otherwise we don't support checking further..., input image must be indexed
        // TODO: Maybe support in the future via (automatic) Tilequant?
        return Err(exceptions::PyValueError::new_err(
            "Expected an indexed image.",
        ));
    }
    let args = PyTuple::new(py, ["raw", "P"]);
    let bytes: Vec<u8> = iimg.getattr(py, "tobytes")?.call1(py, args)?.extract(py)?;
    let pal: Vec<u8> = iimg
        .getattr(py, "palette")?
        .getattr(py, "palette")?
        .extract(py)?;
    Ok((
        StBytesMut::from(bytes),
        StBytesMut::from(pal),
        iimg.getattr(py, "width")?.extract(py)?,
        iimg.getattr(py, "height")?.extract(py)?,
    ))
}

pub fn out_to_py(img: IndexedImage, py: Python) -> PyResult<PyObject> {
    let bytes: &PyBytes = PyBytes::new(py, &img.0 .0);
    let args = PyTuple::new(
        py,
        [
            "P".into_py(py),
            PyTuple::new(py, [img.0 .1, img.0 .2]).into_py(py),
            bytes.into_py(py),
            "raw".into_py(py),
            "P".into_py(py),
            0.into_py(py),
            1.into_py(py),
        ],
    );
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
                error!("skytemple-rust: Critical error during image conversion:");
                e.print(py);
                py.None()
            }
        }
    }
}

/// Simple single-palette image quantization. Reduces to 15 colors and adds one transparent color at index 0.
/// The transparent (alpha=0) pixels in the input image are converted to that color (if can_have_transparency=True).
/// If you need to do tiled multi-palette quantization, use Tilequant instead!
fn pil_simple_quant(
    py: Python,
    mut pil_img: PyObject,
    can_have_transparency: bool,
) -> PyResult<PyObject> {
    let args;
    let transparency_map: Vec<bool>;
    if can_have_transparency {
        if pil_img.getattr(py, "mode")?.extract::<&str>(py)? != "RGBA" {
            args = PyTuple::new(py, ["RGBA"]);
            pil_img = pil_img.getattr(py, "convert")?.call1(py, args)?;
        }
        transparency_map =
            PyIterator::from_object(py, &pil_img.getattr(py, "getdata")?.call0(py)?)?
                .map(|x| Ok(x?.extract::<&PyTuple>()?.get_item(3)?.extract::<usize>()? == 0))
                .collect::<PyResult<Vec<bool>>>()?;
    } else {
        if pil_img.getattr(py, "mode")?.extract::<&str>(py)? != "RGB" {
            args = PyTuple::new(py, ["RGB"]);
            pil_img = pil_img.getattr(py, "convert")?.call1(py, args)?;
        }
        transparency_map =
            PyIterator::from_object(py, &pil_img.getattr(py, "getdata")?.call0(py)?)?
                .map(|_| false)
                .collect();
    }
    let args = PyTuple::new(
        py,
        [
            15.into_py(py),
            py.None(),
            0.into_py(py),
            py.None(),
            0.into_py(py),
        ],
    );
    pil_img = pil_img.getattr(py, "quantize")?.call1(py, args)?;
    // Get the original palette and add the transparent color
    let args = PyTuple::new(
        py,
        [[Ok(0), Ok(0), Ok(0)]
            .into_iter()
            .chain(
                PyIterator::from_object(py, &pil_img.getattr(py, "getpalette")?.call0(py)?)?
                    .take(762)
                    .map(|x| x?.extract::<u8>()),
            )
            .collect::<PyResult<Vec<u8>>>()?
            .into_py(py)],
    );
    pil_img.getattr(py, "putpalette")?.call1(py, args)?;
    // Shift up all pixel values by 1 and add the transparent pixels
    let pixels = pil_img.getattr(py, "load")?.call0(py)?;
    let mut k = 0;
    for j in 0..(pil_img.getattr(py, "height")?.extract(py)?) {
        for i in 0..(pil_img.getattr(py, "width")?.extract(py)?) {
            if transparency_map[k] {
                let args = PyTuple::new(py, [PyTuple::new(py, [i, j]).into_py(py), 0.into_py(py)]);
                pixels.getattr(py, "__setitem__")?.call1(py, args)?;
            } else {
                let inner_args = PyTuple::new(py, [PyTuple::new(py, [i, j])]);
                let args = PyTuple::new(
                    py,
                    [
                        PyTuple::new(py, [i, j]).into_py(py),
                        (pixels
                            .getattr(py, "__getitem__")?
                            .call1(py, inner_args)?
                            .extract::<usize>(py)?
                            + 1)
                        .into_py(py),
                    ],
                );
                pixels.getattr(py, "__setitem__")?.call1(py, args)?;
            }
            k += 1
        }
    }
    Ok(pil_img)
}
