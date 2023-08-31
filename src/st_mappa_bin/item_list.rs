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
use crate::st_mappa_bin::Probability;
use bytes::{Buf, BufMut, BytesMut};
use packed_struct::PrimitiveEnum;
use std::collections::BTreeMap;
use std::ops::Deref;

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq, Eq)]
pub struct MappaItemList {
    #[pyo3(get, set)]
    pub categories: BTreeMap<u16, Probability>,
    #[pyo3(get, set)]
    pub items: BTreeMap<u16, Probability>,
}

impl MappaItemList {
    const CMD_SKIP: u16 = 0x7530;
    const MAX_ITEM_ID: u16 = 363;
}

#[pymethods]
impl MappaItemList {
    #[new]
    pub fn new(categories: BTreeMap<u16, Probability>, items: BTreeMap<u16, Probability>) -> Self {
        Self { categories, items }
    }

    #[classmethod]
    #[cfg(feature = "python")]
    pub fn from_bytes(
        _cls: &PyType,
        mut bytes: StBytes,
        pointer: usize,
    ) -> PyResult<Py<MappaItemList>> {
        bytes.advance(pointer);
        bytes.try_into()
    }

    //noinspection RsSelfConvention
    #[cfg(feature = "python")]
    pub fn to_bytes(slf: Py<Self>) -> StBytes {
        slf.into()
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

impl TryFrom<StBytes> for Py<MappaItemList> {
    type Error = PyErr;

    fn try_from(mut value: StBytes) -> Result<Self, Self::Error> {
        let mut processing_categories = true;
        let mut item_or_cat_id: i32 = 0;
        #[cfg(debug_assertions)]
        let mut orig_value = value.clone();

        let mut items: BTreeMap<u16, Probability> = BTreeMap::new();
        let mut categories: BTreeMap<u16, Probability> = BTreeMap::new();

        while item_or_cat_id <= MappaItemList::MAX_ITEM_ID as i32 {
            let val_prim = value.get_u16_le();
            let val = Probability::from_primitive(val_prim).unwrap();

            let weight = match val {
                Probability::Percentage(rval) if rval > MappaItemList::CMD_SKIP => None,
                _ => Some(val),
            };

            if let Some(weight) = weight {
                if processing_categories {
                    categories.insert(
                        item_or_cat_id.try_into().map_err(|_| {
                            exceptions::PyValueError::new_err(
                                "Overflow while trying to load item list.",
                            )
                        })?,
                        weight,
                    );
                } else {
                    items.insert(
                        item_or_cat_id.try_into().map_err(|_| {
                            exceptions::PyValueError::new_err(
                                "Overflow while trying to load item list.",
                            )
                        })?,
                        weight,
                    );
                }
                item_or_cat_id += 1;
            } else {
                // Skip
                item_or_cat_id += val_prim as i32 - MappaItemList::CMD_SKIP as i32;
            }

            if processing_categories && item_or_cat_id >= 0xF {
                processing_categories = false;
                item_or_cat_id -= 0x10;
            }
        }

        let mil = Python::with_gil(|py| Py::new(py, MappaItemList::new(categories, items)))?;

        #[cfg(debug_assertions)]
        {
            let orig_value_len = orig_value.len();
            debug_assert_eq!(
                StBytes(orig_value.copy_to_bytes(orig_value_len - value.len())),
                crate::bytes::AsStBytes::as_bytes(&mil)
            );
        }

        Ok(mil)
    }
}

impl From<Py<MappaItemList>> for StBytes {
    fn from(value: Py<MappaItemList>) -> Self {
        Python::with_gil(|py| {
            let value_brw = value.borrow(py);
            let mut data = BytesMut::with_capacity(512);
            let mut current_id = 0;
            // Start with the categories
            for (&cat, &val) in &value_brw.categories {
                if current_id != cat {
                    current_id = write_skip(&mut data, current_id, cat)
                }
                data.put_u16_le(val.to_primitive());
                current_id += 1;
            }
            // Continue with the items
            let first_item_id = value_brw.items.keys().copied().next().unwrap_or_default();
            write_skip(&mut data, current_id, 0x10 + first_item_id);
            current_id = first_item_id;
            for (&item, &val) in &value_brw.items {
                if current_id != item {
                    current_id = write_skip(&mut data, current_id, item);
                }
                data.put_u16_le(val.to_primitive());
                current_id += 1;
            }
            // Fill up to MAX_ITEM_ID + 1
            write_skip(&mut data, current_id, MappaItemList::MAX_ITEM_ID + 1);
            data.freeze().into()
        })
    }
}

fn write_skip(data: &mut BytesMut, current_id: u16, target_id: u16) -> u16 {
    if current_id != target_id {
        data.put_u16_le(target_id - current_id + MappaItemList::CMD_SKIP);
    }
    target_id
}
