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

use crate::python::*;
use std::cell::Ref;

#[derive(PartialEq, Eq, Debug, Clone, Default)]
#[pyclass(module = "skytemple_rust")]
pub struct TilemapEntry(pub usize, pub bool, pub bool, pub u8); // idx, flip_x, flip_y, pal_idx

#[cfg(feature = "python")]
#[pymethods]
impl TilemapEntry {
    #[new]
    #[pyo3(signature = (idx, flip_x, flip_y, pal_idx, ignore_too_large = false))]
    #[allow(unused_variables)]
    pub fn new(
        idx: usize,
        flip_x: bool,
        flip_y: bool,
        pal_idx: u8,
        ignore_too_large: bool,
    ) -> Self {
        Self(idx, flip_x, flip_y, pal_idx)
    }
    #[classmethod]
    pub fn from_int(_cls: &PyType, i: usize) -> Self {
        Self::from(i)
    }
    pub fn to_int(&self) -> usize {
        self._to_int()
    }
    #[getter]
    fn get_idx(&self) -> PyResult<usize> {
        Ok(self.0)
    }
    #[setter]
    fn set_idx(&mut self, value: usize) -> PyResult<()> {
        self.0 = value;
        Ok(())
    }
    #[getter]
    fn get_flip_x(&self) -> PyResult<bool> {
        Ok(self.1)
    }
    #[setter]
    fn set_flip_x(&mut self, value: bool) -> PyResult<()> {
        self.1 = value;
        Ok(())
    }
    #[getter]
    fn get_flip_y(&self) -> PyResult<bool> {
        Ok(self.2)
    }
    #[setter]
    fn set_flip_y(&mut self, value: bool) -> PyResult<()> {
        self.2 = value;
        Ok(())
    }
    #[getter]
    fn get_pal_idx(&self) -> PyResult<u8> {
        Ok(self.3)
    }
    #[setter]
    fn set_pal_idx(&mut self, value: u8) -> PyResult<()> {
        self.3 = value;
        Ok(())
    }
}

impl From<usize> for TilemapEntry {
    fn from(entry: usize) -> Self {
        TilemapEntry(
            // 0000 0011 1111 1111, tile index
            entry & 0x3FF,
            // 0000 0100 0000 0000, hflip
            (entry & 0x400) > 0,
            // 0000 1000 0000 0000, vflip
            (entry & 0x800) > 0,
            // 1111 0000 0000 0000, pal index
            ((entry & 0xF000) >> 12) as u8,
        )
    }
}

impl From<TilemapEntry> for usize {
    fn from(entry: TilemapEntry) -> Self {
        entry.into()
    }
}

impl TilemapEntry {
    pub(crate) fn _to_int(&self) -> usize {
        (self.0 & 0x3FF)
            + (if self.1 { 1 } else { 0 } << 10)
            + (if self.2 { 1 } else { 0 } << 11)
            + ((self.3 as usize & 0x3F) << 12)
    }
}

impl From<&TilemapEntry> for usize {
    fn from(entry: &TilemapEntry) -> Self {
        entry._to_int()
    }
}

#[derive(Clone)]
pub struct InputTilemapEntry(pub Py<TilemapEntry>);

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for InputTilemapEntry {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        if let Ok(obj) = ob.extract::<Py<TilemapEntry>>() {
            Ok(Self(obj))
        } else if ob.hasattr("to_int")? {
            let val: usize = ob.call_method0("to_int")?.extract()?;
            let tm: TilemapEntry = val.into();
            Ok(Self(Py::new(ob.py(), tm)?))
        } else {
            Err(exceptions::PyTypeError::new_err(
                "Could not convert into TilemapEntry.",
            ))
        }
    }
}

#[cfg(feature = "python")]
impl IntoPy<Py<TilemapEntry>> for InputTilemapEntry {
    fn into_py(self, _py: Python) -> Py<TilemapEntry> {
        self.0
    }
}

#[cfg(feature = "python")]
impl From<InputTilemapEntry> for TilemapEntry {
    fn from(obj: InputTilemapEntry) -> Self {
        Python::with_gil(|py| obj.0.extract(py).unwrap())
    }
}

#[cfg(not(feature = "python"))]
impl From<InputTilemapEntry> for TilemapEntry {
    fn from(obj: InputTilemapEntry) -> Self {
        obj.0.extract(Python).unwrap()
    }
}

impl From<InputTilemapEntry> for Py<TilemapEntry> {
    fn from(obj: InputTilemapEntry) -> Self {
        obj.0
    }
}

pub trait ProvidesTilemapEntry {
    fn idx(&self) -> usize;
    fn flip_x(&self) -> bool;
    fn flip_y(&self) -> bool;
    fn pal_idx(&self) -> u8;
    fn to_int(&self) -> usize;
}

impl ProvidesTilemapEntry for TilemapEntry {
    fn idx(&self) -> usize {
        self.0
    }

    fn flip_x(&self) -> bool {
        self.1
    }

    fn flip_y(&self) -> bool {
        self.2
    }

    fn pal_idx(&self) -> u8 {
        self.3
    }

    fn to_int(&self) -> usize {
        TilemapEntry::_to_int(self)
    }
}

impl ProvidesTilemapEntry for &TilemapEntry {
    fn idx(&self) -> usize {
        self.0
    }

    fn flip_x(&self) -> bool {
        self.1
    }

    fn flip_y(&self) -> bool {
        self.2
    }

    fn pal_idx(&self) -> u8 {
        self.3
    }

    fn to_int(&self) -> usize {
        TilemapEntry::_to_int(self)
    }
}

impl<'a> ProvidesTilemapEntry for Ref<'a, TilemapEntry> {
    fn idx(&self) -> usize {
        self.0
    }

    fn flip_x(&self) -> bool {
        self.1
    }

    fn flip_y(&self) -> bool {
        self.2
    }

    fn pal_idx(&self) -> u8 {
        self.3
    }

    fn to_int(&self) -> usize {
        TilemapEntry::_to_int(self)
    }
}

#[cfg(feature = "python")]
impl<'py> ProvidesTilemapEntry for PyRef<'py, TilemapEntry> {
    fn idx(&self) -> usize {
        self.0
    }

    fn flip_x(&self) -> bool {
        self.1
    }

    fn flip_y(&self) -> bool {
        self.2
    }

    fn pal_idx(&self) -> u8 {
        self.3
    }

    fn to_int(&self) -> usize {
        TilemapEntry::_to_int(self)
    }
}

#[cfg(feature = "python")]
impl ProvidesTilemapEntry for InputTilemapEntry {
    fn idx(&self) -> usize {
        Python::with_gil(|py| self.0.borrow(py).0)
    }

    fn flip_x(&self) -> bool {
        Python::with_gil(|py| self.0.borrow(py).1)
    }

    fn flip_y(&self) -> bool {
        Python::with_gil(|py| self.0.borrow(py).2)
    }

    fn pal_idx(&self) -> u8 {
        Python::with_gil(|py| self.0.borrow(py).3)
    }

    fn to_int(&self) -> usize {
        Python::with_gil(|py| self.0.borrow(py)._to_int())
    }
}
