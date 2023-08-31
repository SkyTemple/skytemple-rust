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
use crate::bytes::StBytes;
use crate::python::*;
use crate::st_mappa_bin::MappaTrapType;
use bytes::Buf;
use packed_struct::prelude::*;
use std::collections::BTreeMap;
use std::ops::Deref;

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq, Eq)]
pub struct MappaTrapList {
    #[pyo3(get, set)]
    pub weights: BTreeMap<MappaTrapType, u16>,
}

impl MappaTrapList {
    pub fn new(weights: BTreeMap<MappaTrapType, u16>) -> Self {
        Self { weights }
    }
}

impl TryFrom<StBytes> for Py<MappaTrapList> {
    type Error = PyErr;

    fn try_from(mut value: StBytes) -> Result<Self, Self::Error> {
        if value.len() < 50 {
            Err(exceptions::PyValueError::new_err("Trap list malformed."))
        } else {
            Python::with_gil(|py| {
                Py::new(
                    py,
                    MappaTrapList::new(
                        (0u8..25)
                            .map(|i| {
                                (
                                    MappaTrapType::from_primitive(i).unwrap(),
                                    value.get_u16_le(),
                                )
                            })
                            .collect::<BTreeMap<MappaTrapType, u16>>(),
                    ),
                )
            })
        }
    }
}

impl From<Py<MappaTrapList>> for StBytes {
    fn from(value: Py<MappaTrapList>) -> Self {
        Python::with_gil(|py| {
            let value_brw = value.borrow(py);
            let x = (0u8..25)
                .flat_map(|i| {
                    value_brw
                        .weights
                        .get(&MappaTrapType::from_primitive(i).unwrap())
                        .unwrap()
                        .to_le_bytes()
                })
                .collect::<StBytes>();
            debug_assert_eq!(50, x.len());
            x
        })
    }
}

#[pymethods]
impl MappaTrapList {
    #[cfg(feature = "python")]
    #[new]
    pub fn _new(weights: &PyAny) -> PyResult<Self> {
        // weights: Union[List[u16], Dict[_MappaTrapType, u16]]
        if let Ok(dw) = weights.downcast::<pyo3::types::PyDict>() {
            let weights_c = dw
                .into_iter()
                .map(|(k, v)| {
                    if let Ok(kk) = k.extract::<MappaTrapType>() {
                        if let Ok(vv) = v.extract::<u16>() {
                            return Ok((kk, vv));
                        }
                    }
                    Err(exceptions::PyValueError::new_err(
                        "Invalid key(s) or value(s) for trap dict.",
                    ))
                })
                .collect::<PyResult<BTreeMap<MappaTrapType, u16>>>()?;
            if weights_c.len() != 25 {
                Err(exceptions::PyValueError::new_err(
                    "MappaTrapList constructor needs a weight value for all of the 25 traps.",
                ))
            } else {
                Ok(Self::new(weights_c))
            }
        } else if let Ok(dl) = weights.downcast::<pyo3::types::PyList>() {
            if dl.len() != 25 {
                Err(exceptions::PyValueError::new_err(
                    "MappaTrapList constructor needs a weight value for all of the 25 traps.",
                ))
            } else {
                Ok(Self::new(
                    dl.into_iter()
                        .enumerate()
                        .map(|(i, v)| {
                            if let Ok(vv) = v.extract::<u16>() {
                                Ok((MappaTrapType::from_primitive(i as u8).unwrap(), vv))
                            } else {
                                Err(exceptions::PyValueError::new_err(
                                    "Invalid value(s) for trap list.",
                                ))
                            }
                        })
                        .collect::<PyResult<BTreeMap<MappaTrapType, u16>>>()?,
                ))
            }
        } else {
            Err(exceptions::PyTypeError::new_err(
                "The weights must be a list or dict of probabilities.",
            ))
        }
    }

    #[cfg(feature = "python")]
    fn __richcmp__(&self, other: PyRef<Self>, op: pyo3::basic::CompareOp) -> Py<PyAny> {
        let py = other.py();
        match op {
            pyo3::basic::CompareOp::Eq => (self == other.deref()).into_py(py),
            pyo3::basic::CompareOp::Ne => (self != other.deref()).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}
