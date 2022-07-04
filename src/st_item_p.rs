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
use crate::st_sir0::{Sir0Result, Sir0Serializable};
use crate::static_data::InStaticData;
use packed_struct::prelude::*;

#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
#[pyclass(module = "skytemple_rust.st_item_p")]
pub struct ItemPEntry {
    #[pyo3(get, set)]
    pub buy_price: u16,
    #[pyo3(get, set)]
    pub sell_price: u16,
    #[pyo3(get, set)]
    pub category: u8,
    #[pyo3(get, set)]
    pub sprite: u8,
    #[pyo3(get, set)]
    pub item_id: u16,
    #[pyo3(get, set)]
    pub move_id: u16,
    #[pyo3(get, set)]
    pub range_min: u8,
    #[pyo3(get, set)]
    pub range_max: u8,
    #[pyo3(get, set)]
    pub palette: u8,
    #[pyo3(get, set)]
    pub action_name: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub is_valid: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub is_in_td: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub ai_flag_1: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub ai_flag_2: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub ai_flag_3: bool,
}

#[pymethods]
impl ItemPEntry {
    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject) -> bool {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_item_p")]
#[derive(Clone)]
pub struct ItemP {
    #[pyo3(get, set)]
    pub item_list: Vec<Py<ItemPEntry>>,
}

#[pymethods]
impl ItemP {
    #[new]
    pub fn new(data: StBytes, pointer_to_pointers: u32, py: Python) -> PyResult<Self> {
        todo!()
    }

    #[cfg(feature = "python")]
    #[pyo3(name = "sir0_serialize_parts")]
    pub fn _sir0_serialize_parts(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.sir0_serialize_parts()?.into_py(py))
    }

    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "sir0_unwrap")]
    pub fn _sir0_unwrap(
        cls: &PyType,
        content_data: StBytes,
        data_pointer: u32,
        static_data: Option<InStaticData>,
    ) -> PyResult<Self> {
        Ok(Self::sir0_unwrap(content_data, data_pointer, static_data)?)
    }
}

impl Sir0Serializable for ItemP {
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<u32>, Option<u32>)> {
        todo!()
    }

    fn sir0_unwrap(
        content_data: StBytes,
        data_pointer: u32,
        static_data: Option<InStaticData>,
    ) -> Sir0Result<Self> {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_item_p")]
#[derive(Clone, Default)]
pub struct ItemPWriter;

#[pymethods]
impl ItemPWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<ItemP>, py: Python) -> PyResult<StBytes> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_item_p_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_item_p";
    let m = PyModule::new(py, name)?;
    m.add_class::<ItemPEntry>()?;
    m.add_class::<ItemP>()?;
    m.add_class::<ItemPWriter>()?;

    Ok((name, m))
}
