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
use crate::err::convert_packing_err;
use crate::python::*;
use packed_struct::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyList;
#[cfg(feature = "python")]
use pyo3::types::PyTuple;
use std::iter::repeat;
use std::ops::Deref;
use std::{mem, vec};

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, Debug, PartialEq)]
pub struct MappaMonsterList {
    pub list: Vec<Py<MappaMonster>>,
}

#[cfg(feature = "python")]
#[pymethods]
impl MappaMonsterList {
    pub fn __iter__(&mut self) -> MappaMonsterListIterator {
        MappaMonsterListIterator::new(self.list.clone().into_iter())
    }
    pub fn __getitem__(&self, idx: SliceOrInt, py: Python) -> PyResult<PyObject> {
        match idx {
            SliceOrInt::Slice(sl) => {
                let pylist = PyList::new(py, self.list.iter().cloned());
                pylist
                    .call_method1("__getitem__", PyTuple::new(py, [sl]))
                    .map(|v| v.into_py(py))
            }
            SliceOrInt::Int(idx) => {
                if idx >= 0 && idx as usize <= self.list.len() {
                    Ok(self.list[idx as usize].clone().into_py(py))
                } else {
                    Err(exceptions::PyIndexError::new_err("list index out of range"))
                }
            }
        }
    }
    pub fn __setitem__(&mut self, idx: SliceOrInt, o: PyObject, py: Python) -> PyResult<()> {
        match idx {
            SliceOrInt::Slice(sl) => {
                let pylist = PyList::new(py, self.list.iter().cloned());
                pylist.call_method1("__setitem__", PyTuple::new(py, [sl.into_py(py), o]))?;
                self.list = pylist
                    .into_iter()
                    .map(|o| o.extract())
                    .collect::<PyResult<Vec<Py<MappaMonster>>>>()?;
                Ok(())
            }
            SliceOrInt::Int(idx) => {
                if idx >= 0 && idx as usize <= self.list.len() {
                    self.list[idx as usize] = o.extract(py)?;
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
                let pylist = PyList::new(py, self.list.iter().cloned());
                pylist.call_method1("__delitem__", PyTuple::new(py, [sl]))?;
                self.list = pylist
                    .into_iter()
                    .map(|o| o.extract())
                    .collect::<PyResult<Vec<Py<MappaMonster>>>>()?;
                Ok(())
            }
            SliceOrInt::Int(idx) => {
                if idx >= 0 && idx as usize <= self.list.len() {
                    self.list.remove(idx as usize);
                    Ok(())
                } else {
                    Err(exceptions::PyIndexError::new_err("list index out of range"))
                }
            }
        }
    }
    pub fn __len__(&self) -> usize {
        self.list.len()
    }
    pub fn index(&self, value: PyObject, py: Python) -> PyResult<usize> {
        if let Ok(value) = value.extract::<Py<MappaMonster>>(py) {
            if let Some(idx) = self.list.iter().position(|x| {
                x.call_method1(py, "__eq__", PyTuple::new(py, [value.clone()]))
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
        if let Ok(value) = value.extract::<Py<MappaMonster>>(py) {
            self.list
                .iter()
                .filter(|x| {
                    x.call_method1(py, "__eq__", PyTuple::new(py, [value.clone()]))
                        .and_then(|x| x.is_true(py))
                        .unwrap_or_default()
                })
                .count()
        } else {
            0
        }
    }
    pub fn insert(&mut self, index: usize, value: Py<MappaMonster>) {
        self.list.insert(index, value)
    }
    pub fn append(&mut self, value: Py<MappaMonster>) {
        self.list.push(value)
    }
    pub fn clear(&mut self) {
        self.list.clear()
    }
    pub fn extend(&mut self, _value: PyObject) -> PyResult<()> {
        Err(exceptions::PyNotImplementedError::new_err("Not supported."))
    }
    #[args(index = "0")]
    pub fn pop(&mut self, idx: isize) -> PyResult<Py<MappaMonster>> {
        if idx == 0 {
            if !self.list.is_empty() {
                Ok(self.list.pop().unwrap())
            } else {
                Err(exceptions::PyIndexError::new_err("pop from empty list"))
            }
        } else if idx >= 0 && idx as usize <= self.list.len() {
            Ok(self.list.remove(idx as usize))
        } else {
            Err(exceptions::PyIndexError::new_err("pop index out of range"))
        }
    }
    pub fn remove(&mut self, value: PyObject, py: Python) -> PyResult<()> {
        if let Ok(value) = value.extract::<Py<MappaMonster>>(py) {
            if let Some(idx) = self.list.iter().position(|x| {
                x.call_method1(py, "__eq__", PyTuple::new(py, [value.clone()]))
                    .and_then(|x| x.is_true(py))
                    .unwrap_or_default()
            }) {
                self.list.remove(idx);
                Ok(())
            } else {
                Err(exceptions::PyValueError::new_err("not in list"))
            }
        } else {
            Err(exceptions::PyValueError::new_err("not in list"))
        }
    }
    pub fn __iadd__(&mut self, value: PyObject) -> PyResult<()> {
        self.extend(value)
    }
}

impl TryFrom<StBytes> for Py<MappaMonsterList> {
    type Error = PyErr;

    fn try_from(value: StBytes) -> Result<Self, Self::Error> {
        static_assert_size!(<MappaMonster as PackedStruct>::ByteArray, 0x08);

        Python::with_gil(|py| {
            let mut monsters = Vec::with_capacity(50);
            loop {
                let monster =
                    MappaMonster::unpack_from_slice(&value[..]).map_err(convert_packing_err)?;
                if monster.md_index == 0 {
                    break;
                }
                monsters.push(Py::new(py, monster)?);
            }
            Py::new(py, MappaMonsterList { list: monsters })
        })
    }
}

impl From<Py<MappaMonsterList>> for StBytes {
    fn from(value: Py<MappaMonsterList>) -> Self {
        Python::with_gil(|py| {
            let value_brw = value.borrow(py);
            value_brw
                .list
                .iter()
                .flat_map(|m| m.borrow(py).pack().unwrap())
                .chain(repeat(0).take(mem::size_of::<<MappaMonster as PackedStruct>::ByteArray>()))
                .collect::<StBytes>()
        })
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
pub struct MappaMonster {
    level_raw: u16,
    #[pyo3(get, set)]
    pub weight: u16,
    #[pyo3(get, set)]
    pub weight2: u16,
    #[pyo3(get, set)]
    pub md_index: u16,
}

#[pymethods]
impl MappaMonster {
    const LEVEL_MULTIPLIER: u16 = 512;

    #[new]
    pub fn new(level: u8, weight: u16, weight2: u16, md_index: u16) -> Self {
        Self {
            level_raw: (level as u16) * Self::LEVEL_MULTIPLIER,
            weight,
            weight2,
            md_index,
        }
    }

    #[getter]
    pub fn level(&self) -> PyResult<u8> {
        u8::try_from(self.level_raw / Self::LEVEL_MULTIPLIER)
            .map_err(|_| exceptions::PyValueError::new_err("Monster has invalid level value."))
    }

    #[setter]
    pub fn set_level(&mut self, level: u8) {
        self.level_raw = (level as u16) * Self::LEVEL_MULTIPLIER;
    }

    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject, py: Python) -> bool {
        if let Ok(other) = other.extract::<Py<Self>>(py) {
            self == other.borrow(py).deref()
        } else {
            false
        }
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
pub struct MappaMonsterListIterator {
    iter: vec::IntoIter<Py<MappaMonster>>,
}

impl MappaMonsterListIterator {
    pub fn new(iter: vec::IntoIter<Py<MappaMonster>>) -> Self {
        Self { iter }
    }
}

#[pymethods]
impl MappaMonsterListIterator {
    pub fn __next__(&mut self) -> Option<Py<MappaMonster>> {
        self.iter.next()
    }
}
