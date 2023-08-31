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
use crate::st_mappa_bin::{
    MappaFloorLayout, MappaItemList, MappaMonster, MappaMonsterList, MappaTrapList,
};
use crate::util::Lazy;
use std::ops::Deref;

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq)]
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
        layout: MappaFloorLayout,
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
            layout: Lazy::Instance(Py::new(py, layout)?),
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
    #[cfg(feature = "python")]
    pub fn layout(&mut self) -> PyResult<Py<MappaFloorLayout>> {
        Ok(self.layout.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_layout(&mut self, value: Py<MappaFloorLayout>) -> PyResult<()> {
        self.layout = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn monsters(&mut self) -> PyResult<Py<MappaMonsterList>> {
        Ok(self.monsters.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
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
    #[cfg(feature = "python")]
    pub fn traps(&mut self) -> PyResult<Py<MappaTrapList>> {
        Ok(self.traps.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_traps(&mut self, value: Py<MappaTrapList>) -> PyResult<()> {
        self.traps = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn floor_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.floor_items.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_floor_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.floor_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn shop_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.shop_items.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_shop_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.shop_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn monster_house_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.monster_house_items.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_monster_house_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.monster_house_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn buried_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.buried_items.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_buried_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.buried_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk_items1(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.unk_items1.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk_items1(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.unk_items1 = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk_items2(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.unk_items2.instance()?.clone())
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk_items2(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.unk_items2 = Lazy::Instance(value);
        Ok(())
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
