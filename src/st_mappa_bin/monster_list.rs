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
use bytes::Buf;
use packed_struct::prelude::*;
use std::iter::repeat;
use std::mem;
use std::ops::Deref;

impl_pylist!(
    "skytemple_rust.st_mappa_bin",
    MappaMonsterList,
    Py<MappaMonster>
);

impl TryFrom<StBytes> for Py<MappaMonsterList> {
    type Error = PyErr;

    fn try_from(mut value: StBytes) -> Result<Self, Self::Error> {
        static_assert_size!(<MappaMonster as PackedStruct>::ByteArray, 0x08);

        Python::with_gil(|py| {
            let mut monsters = Vec::with_capacity(50);
            loop {
                let monster = MappaMonster::unpack_from_slice(&value.copy_to_bytes(0x08)[..])
                    .map_err(convert_packing_err)?;
                if monster.md_index == 0 {
                    break;
                }
                monsters.push(Py::new(py, monster)?);
            }
            Py::new(py, MappaMonsterList(monsters))
        })
    }
}

impl From<Py<MappaMonsterList>> for StBytes {
    fn from(value: Py<MappaMonsterList>) -> Self {
        Python::with_gil(|py| {
            let value_brw = value.borrow(py);
            value_brw
                .0
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
    pub main_spawn_weight: u16,
    #[pyo3(get, set)]
    pub monster_house_spawn_weight: u16,
    #[pyo3(get, set)]
    pub md_index: u16,
}

#[pymethods]
impl MappaMonster {
    const LEVEL_MULTIPLIER: u16 = 512;

    #[new]
    pub fn new(
        level: u8,
        main_spawn_weight: u16,
        monster_house_spawn_weight: u16,
        md_index: u16,
    ) -> Self {
        Self {
            level_raw: (level as u16) * Self::LEVEL_MULTIPLIER,
            main_spawn_weight,
            monster_house_spawn_weight,
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
    fn __richcmp__(&self, other: PyRef<Self>, op: pyo3::basic::CompareOp) -> Py<PyAny> {
        let py = other.py();
        match op {
            pyo3::basic::CompareOp::Eq => (self == other.deref()).into_py(py),
            pyo3::basic::CompareOp::Ne => (self != other.deref()).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}
