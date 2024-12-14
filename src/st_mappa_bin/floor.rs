/*
 * Copyright 2021-2024 Capypara and the SkyTemple Contributors
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
use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use std::ops::Deref;

use crate::st_mappa_bin::{
    MappaFloorLayout, MappaItemList, MappaMonster, MappaMonsterList, MappaTrapList,
};
use crate::util::Lazy;

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
pub struct MappaFloor {
    pub layout: Lazy<Py<MappaFloorLayout>>,
    pub monsters: Lazy<Py<MappaMonsterList>>,
    pub traps: Lazy<Py<MappaTrapList>>,
    pub floor_items: Lazy<Py<MappaItemList>>,
    pub shop_items: Lazy<Py<MappaItemList>>,
    pub monster_house_items: Lazy<Py<MappaItemList>>,
    pub buried_items: Lazy<Py<MappaItemList>>,
    pub unk_items1: Lazy<Py<MappaItemList>>,
    pub unk_items2: Lazy<Py<MappaItemList>>,
}

#[pymethods]
impl MappaFloor {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layout: Py<MappaFloorLayout>,
        monsters: Vec<Py<MappaMonster>>,
        traps: MappaTrapList,
        floor_items: MappaItemList,
        shop_items: MappaItemList,
        monster_house_items: MappaItemList,
        buried_items: MappaItemList,
        unk_items1: MappaItemList,
        unk_items2: MappaItemList,
        py: Python,
    ) -> PyResult<Self> {
        Ok(Self {
            layout: Lazy::Instance(layout),
            monsters: Lazy::Instance(Py::new(py, MappaMonsterList(monsters))?),
            traps: Lazy::Instance(Py::new(py, traps)?),
            floor_items: Lazy::Instance(Py::new(py, floor_items)?),
            shop_items: Lazy::Instance(Py::new(py, shop_items)?),
            monster_house_items: Lazy::Instance(Py::new(py, monster_house_items)?),
            buried_items: Lazy::Instance(Py::new(py, buried_items)?),
            unk_items1: Lazy::Instance(Py::new(py, unk_items1)?),
            unk_items2: Lazy::Instance(Py::new(py, unk_items2)?),
        })
    }

    #[getter]

    pub fn layout(&mut self, py: Python) -> PyResult<Py<MappaFloorLayout>> {
        Ok(self.layout.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_layout(&mut self, value: Py<MappaFloorLayout>) -> PyResult<()> {
        self.layout = Lazy::Instance(value);
        Ok(())
    }

    #[getter]

    pub fn monsters(&mut self, py: Python) -> PyResult<Py<MappaMonsterList>> {
        Ok(self.monsters.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_monsters(&mut self, py: Python, value: PyObject) -> PyResult<()> {
        if let Ok(val) = value.extract::<Py<MappaMonsterList>>(py) {
            self.monsters = Lazy::Instance(val);
            Ok(())
        } else {
            match value.extract::<Vec<Py<MappaMonster>>>(py) {
                Ok(v) => {
                    self.monsters = Lazy::Instance(Py::new(py, MappaMonsterList(v))?);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }

    #[getter]

    pub fn traps(&mut self, py: Python) -> PyResult<Py<MappaTrapList>> {
        Ok(self.traps.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_traps(&mut self, value: Py<MappaTrapList>) -> PyResult<()> {
        self.traps = Lazy::Instance(value);
        Ok(())
    }

    #[getter]

    pub fn floor_items(&mut self, py: Python) -> PyResult<Py<MappaItemList>> {
        Ok(self.floor_items.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_floor_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.floor_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]

    pub fn shop_items(&mut self, py: Python) -> PyResult<Py<MappaItemList>> {
        Ok(self.shop_items.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_shop_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.shop_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]

    pub fn monster_house_items(&mut self, py: Python) -> PyResult<Py<MappaItemList>> {
        Ok(self.monster_house_items.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_monster_house_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.monster_house_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]

    pub fn buried_items(&mut self, py: Python) -> PyResult<Py<MappaItemList>> {
        Ok(self.buried_items.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_buried_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.buried_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]

    pub fn unk_items1(&mut self, py: Python) -> PyResult<Py<MappaItemList>> {
        Ok(self.unk_items1.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_unk_items1(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.unk_items1 = Lazy::Instance(value);
        Ok(())
    }

    #[getter]

    pub fn unk_items2(&mut self, py: Python) -> PyResult<Py<MappaItemList>> {
        Ok(self.unk_items2.instance()?.clone_ref(py))
    }

    #[setter]

    pub fn set_unk_items2(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.unk_items2 = Lazy::Instance(value);
        Ok(())
    }

    fn __richcmp__(&self, other: PyRef<Self>, op: pyo3::basic::CompareOp) -> PyResult<Py<PyAny>> {
        let py = other.py();
        Ok(match op {
            pyo3::basic::CompareOp::Eq => self.eq_pyref(other.deref(), py).into_py_any(py)?,
            pyo3::basic::CompareOp::Ne => { !self.eq_pyref(other.deref(), py) }.into_py_any(py)?,
            _ => py.NotImplemented(),
        })
    }
}

impl MappaFloor {
    pub fn eq_pyref(&self, other: &Self, py: Python) -> bool {
        self.layout.eq_pyref(&other.layout, py)
            && self.monsters.eq_pyref(&other.monsters, py)
            && self.traps.eq_pyref(&other.traps, py)
            && self.floor_items.eq_pyref(&other.floor_items, py)
            && self.shop_items.eq_pyref(&other.shop_items, py)
            && self
                .monster_house_items
                .eq_pyref(&other.monster_house_items, py)
            && self.buried_items.eq_pyref(&other.buried_items, py)
            && self.unk_items1.eq_pyref(&other.unk_items1, py)
            && self.unk_items2.eq_pyref(&other.unk_items2, py)
    }
}
