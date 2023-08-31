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
use crate::st_mappa_bin::minimize::MinimizedMappa;
use crate::st_mappa_bin::MappaFloor;
use crate::st_sir0::{Sir0Error, Sir0Result, Sir0Serializable};
use crate::util::Lazy;
use bytes::Buf;
use std::ops::Deref;

struct MappaReader {
    source: StBytes,
    dungeon_list_index_start: usize,
    floor_layout_data_start: usize,
    item_spawn_list_index_start: usize,
    monster_spawn_list_index_start: usize,
    trap_spawn_list_index_start: usize,
}

impl MappaReader {
    pub fn new(source: StBytes, header_pointer: u32) -> PyResult<Self> {
        let mut header = source.clone();
        if source.len() <= header_pointer as usize {
            return Err(exceptions::PyValueError::new_err(
                "Mappa Header pointer out of bounds.",
            ));
        }
        header.advance(header_pointer as usize);
        if header.len() < 20 {
            Err(exceptions::PyValueError::new_err("Mappa Header too short."))
        } else {
            Ok(Self {
                source,
                dungeon_list_index_start: header.get_u32_le() as usize,
                floor_layout_data_start: header.get_u32_le() as usize,
                item_spawn_list_index_start: header.get_u32_le() as usize,
                monster_spawn_list_index_start: header.get_u32_le() as usize,
                trap_spawn_list_index_start: header.get_u32_le() as usize,
            })
        }
    }
}

impl MappaReader {
    const FLOOR_IDX_ENTRY_LEN: usize = 18;

    fn collect_floor_lists(&self) -> PyResult<Vec<Vec<Py<MappaFloor>>>> {
        let mut buf = &self.source[self.dungeon_list_index_start..self.floor_layout_data_start];
        if buf.len() % 4 != 0 {
            return Err(exceptions::PyValueError::new_err(
                "Can't read floor lists: Invalid list data pointer area size.",
            ));
        }
        let mut dungeons = Vec::with_capacity(buf.len() / 4);
        while buf.has_remaining() {
            dungeons.push(self.collect_floor_list(buf.get_u32_le() as usize)?)
        }
        Ok(dungeons)
    }

    fn collect_floor_list(&self, pointer: usize) -> PyResult<Vec<Py<MappaFloor>>> {
        let mut buf = self.source.slice(pointer..self.dungeon_list_index_start);
        // The zeroth floor is just nulls, we omit it.
        let zero = vec![0u8; Self::FLOOR_IDX_ENTRY_LEN];
        let mut tmpbuf = buf.copy_to_bytes(Self::FLOOR_IDX_ENTRY_LEN);
        pyr_assert!(
            zero[..] == tmpbuf[..],
            "The first floor of a dungeon must be a null floor."
        );
        let mut floors = Vec::with_capacity(100);
        if buf.remaining() >= Self::FLOOR_IDX_ENTRY_LEN {
            tmpbuf = buf.copy_to_bytes(Self::FLOOR_IDX_ENTRY_LEN);
            Python::with_gil(|py| -> PyResult<()> {
                while tmpbuf != zero {
                    floors.push(self.collect_floor(StBytes(tmpbuf), py)?);
                    if buf.remaining() < Self::FLOOR_IDX_ENTRY_LEN {
                        break;
                    }
                    tmpbuf = buf.copy_to_bytes(Self::FLOOR_IDX_ENTRY_LEN);
                }
                Ok(())
            })?;
        }
        Ok(floors)
    }

    fn collect_floor(&self, mut floor_data: StBytes, py: Python) -> PyResult<Py<MappaFloor>> {
        let layout = self.resolve_pointer(
            self.floor_layout_data_start + 32 * (floor_data.get_u16_le() as usize),
            Some(32),
        )?;
        let monsters = self.resolve_pointer(
            self.read_floor_data_pnt(self.monster_spawn_list_index_start, floor_data.get_u16_le())?,
            None,
        )?;
        let traps = self.resolve_pointer(
            self.read_floor_data_pnt(self.trap_spawn_list_index_start, floor_data.get_u16_le())?,
            Some(50),
        )?;
        let floor_items = self.resolve_pointer(
            self.read_floor_data_pnt(self.item_spawn_list_index_start, floor_data.get_u16_le())?,
            None,
        )?;
        let shop_items = self.resolve_pointer(
            self.read_floor_data_pnt(self.item_spawn_list_index_start, floor_data.get_u16_le())?,
            None,
        )?;
        let mh_items = self.resolve_pointer(
            self.read_floor_data_pnt(self.item_spawn_list_index_start, floor_data.get_u16_le())?,
            None,
        )?;
        let buried_items = self.resolve_pointer(
            self.read_floor_data_pnt(self.item_spawn_list_index_start, floor_data.get_u16_le())?,
            None,
        )?;
        let unk1_items = self.resolve_pointer(
            self.read_floor_data_pnt(self.item_spawn_list_index_start, floor_data.get_u16_le())?,
            None,
        )?;
        let unk2_items = self.resolve_pointer(
            self.read_floor_data_pnt(self.item_spawn_list_index_start, floor_data.get_u16_le())?,
            None,
        )?;

        // TODO: Can we keep more lazy? The others are dynamically sized, but maybe there's
        //       some tricks we could do.
        Py::new(
            py,
            MappaFloor {
                layout: Lazy::Source(layout),
                monsters: Lazy::instance_from_source(monsters)?,
                traps: Lazy::Source(traps),
                floor_items: Lazy::instance_from_source(floor_items)?,
                shop_items: Lazy::instance_from_source(shop_items)?,
                monster_house_items: Lazy::instance_from_source(mh_items)?,
                buried_items: Lazy::instance_from_source(buried_items)?,
                unk_items1: Lazy::instance_from_source(unk1_items)?,
                unk_items2: Lazy::instance_from_source(unk2_items)?,
            },
        )
    }
    fn read_floor_data_pnt(&self, index_list_start: usize, index: u16) -> PyResult<usize> {
        let idx = index_list_start + 4 * (index as usize);
        if idx + 4 > self.source.len() {
            Err(exceptions::PyIndexError::new_err(format!(
                "Pointer in floor list out of bounds ({} > {}).",
                idx + 4,
                self.source.len()
            )))
        } else {
            Ok((&self.source[idx..]).get_u32_le() as usize)
        }
    }
    fn resolve_pointer(&self, pnt: usize, len: Option<usize>) -> PyResult<StBytes> {
        let mut b = self.source.clone();
        if pnt > b.len() {
            Err(exceptions::PyIndexError::new_err(
                "Pointer in floor list out of bounds.",
            ))
        } else {
            if let Some(len) = len {
                b = StBytes(b.slice(pnt..pnt + len));
            } else {
                b.advance(pnt);
            }
            Ok(b)
        }
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone)]
pub struct MappaBin {
    #[pyo3(get, set)]
    pub floor_lists: Vec<Vec<Py<MappaFloor>>>,
}

#[pymethods]
impl MappaBin {
    #[new]
    pub fn new(floor_lists: Vec<Vec<Py<MappaFloor>>>) -> Self {
        Self { floor_lists }
    }

    pub fn add_floor_list(&mut self, floor_list: Vec<Py<MappaFloor>>) -> PyResult<()> {
        self.floor_lists.push(floor_list);
        Ok(())
    }

    pub fn remove_floor_list(&mut self, index: usize) -> PyResult<()> {
        if index < self.floor_lists.len() {
            self.floor_lists.remove(index);
            Ok(())
        } else {
            Err(exceptions::PyIndexError::new_err(
                "Floor list index out of bounds",
            ))
        }
    }

    pub fn add_floor_to_floor_list(
        &mut self,
        floor_list_index: usize,
        floor: Py<MappaFloor>,
    ) -> PyResult<()> {
        if floor_list_index < self.floor_lists.len() {
            self.floor_lists[floor_list_index].push(floor);
            Ok(())
        } else {
            Err(exceptions::PyIndexError::new_err(
                "Floor list index out of bounds",
            ))
        }
    }

    pub fn insert_floor_in_floor_list(
        &mut self,
        floor_list_index: usize,
        insert_index: usize,
        floor: Py<MappaFloor>,
    ) -> PyResult<()> {
        if floor_list_index < self.floor_lists.len() {
            if insert_index > self.floor_lists[floor_list_index].len() {
                Err(exceptions::PyIndexError::new_err(
                    "Floor insert index out of bounds",
                ))
            } else {
                self.floor_lists[floor_list_index].insert(insert_index, floor);
                Ok(())
            }
        } else {
            Err(exceptions::PyIndexError::new_err(
                "Floor list index out of bounds",
            ))
        }
    }

    pub fn remove_floor_from_floor_list(
        &mut self,
        floor_list_index: usize,
        floor_index: usize,
    ) -> PyResult<()> {
        if floor_list_index < self.floor_lists.len() {
            let floor_list = &mut self.floor_lists[floor_list_index];
            if floor_index < floor_list.len() {
                floor_list.remove(floor_index);
                Ok(())
            } else {
                Err(exceptions::PyIndexError::new_err(
                    "Floor index out of bounds",
                ))
            }
        } else {
            Err(exceptions::PyIndexError::new_err(
                "Floor list index out of bounds",
            ))
        }
    }

    #[cfg(feature = "python")]
    #[pyo3(name = "sir0_serialize_parts")]
    pub fn _sir0_serialize_parts(&self, py: Python) -> PyResult<PyObject> {
        Ok(self.sir0_serialize_parts()?.into_py(py))
    }

    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "sir0_unwrap")]
    pub fn _sir0_unwrap(_cls: &PyType, content_data: StBytes, data_pointer: u32) -> PyResult<Self> {
        Ok(Self::sir0_unwrap(content_data, data_pointer)?)
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

impl Sir0Serializable for MappaBin {
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<u32>, Option<u32>)> {
        MinimizedMappa::from_mappa(self).sir0_serialize_parts()
    }

    fn sir0_unwrap(content_data: StBytes, data_pointer: u32) -> Sir0Result<Self> {
        Ok(Self {
            floor_lists: MappaReader::new(content_data, data_pointer)
                .map_err(|e| Sir0Error::UnwrapFailed(anyhow::Error::from(e)))?
                .collect_floor_lists()
                .map_err(|e| Sir0Error::UnwrapFailed(anyhow::Error::from(e)))?,
        })
    }
}

// Pyo3 does not compare nested Py vectors by value.
impl PartialEq for MappaBin {
    fn eq(&self, other: &Self) -> bool {
        if self.floor_lists.len() != other.floor_lists.len() {
            false
        } else {
            Python::with_gil(|py| {
                for (sfl, ofl) in self.floor_lists.iter().zip(other.floor_lists.iter()) {
                    if sfl.len() != ofl.len() {
                        return false;
                    }
                    for (sf, of) in sfl.iter().zip(ofl.iter()) {
                        if sf.borrow(py).deref() != of.borrow(py).deref() {
                            return false;
                        }
                    }
                }
                true
            })
        }
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, Default)]
pub struct MappaBinWriter;

#[pymethods]
impl MappaBinWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<MappaBin>, py: Python) -> PyResult<StBytes> {
        model
            .borrow(py)
            .sir0_serialize_parts()
            .map(|(c, _, _)| c)
            .map_err(|e| exceptions::PyValueError::new_err(format!("{}", e)))
    }
}
