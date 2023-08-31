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
use crate::bytes::{AsStBytes, StBytes};
use crate::err::convert_packing_err;
use crate::python::*;
use crate::st_md::PokeType;
use crate::st_sir0::{
    decode_sir0_pointer_offsets, encode_sir0_pointer_offsets, Sir0Error, Sir0Result,
    Sir0Serializable,
};
use crate::util::pad;
use bytes::{Buf, BufMut, BytesMut};
use itertools::Itertools;
use packed_struct::prelude::*;
use packed_struct::PackingResult;
use std::num::TryFromIntError;
use std::ops::Deref;

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum WazaMoveCategory {
    Physical = 0,
    Special = 1,
    Status = 2,
}

#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
#[pyclass(module = "skytemple_rust.st_waza_p")]
pub struct LevelUpMove {
    #[pyo3(get, set)]
    pub move_id: u16,
    #[pyo3(get, set)]
    pub level_id: u16,
}

#[pymethods]
impl LevelUpMove {
    #[new]
    pub fn new(move_id: u16, level_id: u16) -> Self {
        Self { move_id, level_id }
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

impl_pylist!("skytemple_rust.st_waza_p", LevelUpMoveList, Py<LevelUpMove>);
impl_pylist_primitive!("skytemple_rust.st_waza_p", U32List, u32);

#[derive(Clone, Debug)]
#[pyclass(module = "skytemple_rust.st_waza_p")]
pub struct MoveLearnset {
    pub level_up_moves: Py<LevelUpMoveList>,
    pub tm_hm_moves: Py<U32List>,
    pub egg_moves: Py<U32List>,
}

#[pymethods]
impl MoveLearnset {
    #[new]
    pub fn new(
        level_up_moves: Vec<Py<LevelUpMove>>,
        tm_hm_moves: Vec<u32>,
        egg_moves: Vec<u32>,
        py: Python,
    ) -> PyResult<Self> {
        Ok(Self {
            level_up_moves: Py::new(py, LevelUpMoveList(level_up_moves))?,
            tm_hm_moves: Py::new(py, U32List(tm_hm_moves))?,
            egg_moves: Py::new(py, U32List(egg_moves))?,
        })
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn level_up_moves(&self) -> Py<LevelUpMoveList> {
        self.level_up_moves.clone()
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_level_up_moves(&mut self, py: Python, value: PyObject) -> PyResult<()> {
        if let Ok(val) = value.extract::<Py<LevelUpMoveList>>(py) {
            self.level_up_moves = val;
            Ok(())
        } else {
            match value.extract::<Vec<Py<LevelUpMove>>>(py) {
                Ok(v) => {
                    self.level_up_moves = Py::new(py, LevelUpMoveList(v))?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn tm_hm_moves(&self) -> Py<U32List> {
        self.tm_hm_moves.clone()
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_tm_hm_moves(&mut self, py: Python, value: PyObject) -> PyResult<()> {
        if let Ok(val) = value.extract::<Py<U32List>>(py) {
            self.tm_hm_moves = val;
            Ok(())
        } else {
            match value.extract::<Vec<u32>>(py) {
                Ok(v) => {
                    self.tm_hm_moves = Py::new(py, U32List(v))?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn egg_moves(&self) -> Py<U32List> {
        self.egg_moves.clone()
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_egg_moves(&mut self, py: Python, value: PyObject) -> PyResult<()> {
        if let Ok(val) = value.extract::<Py<U32List>>(py) {
            self.egg_moves = val;
            Ok(())
        } else {
            match value.extract::<Vec<u32>>(py) {
                Ok(v) => {
                    self.egg_moves = Py::new(py, U32List(v))?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
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

impl PartialEq for MoveLearnset {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            self.level_up_moves.borrow(py).deref() == other.level_up_moves.borrow(py).deref()
                && self.tm_hm_moves.borrow(py).deref() == other.tm_hm_moves.borrow(py).deref()
                && self.egg_moves.borrow(py).deref() == other.egg_moves.borrow(py).deref()
        })
    }
}

impl Eq for MoveLearnset {}

#[derive(Clone, Copy, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "msb")]
#[pyclass(module = "skytemple_rust.st_waza_p")]
pub struct WazaMoveRangeSettings {
    #[pyo3(get, set)]
    #[packed_field(size_bits = "4")]
    pub range: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "4")]
    pub target: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "4")]
    pub unused: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "4")]
    pub condition: u8,
}

#[pymethods]
impl WazaMoveRangeSettings {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        <Self as PackedStruct>::unpack(&data[..2].try_into().unwrap()).map_err(convert_packing_err)
    }

    #[cfg(feature = "python")]
    pub fn __int__(&self) -> u16 {
        self.into()
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

impl From<&WazaMoveRangeSettings> for u16 {
    fn from(v: &WazaMoveRangeSettings) -> Self {
        u16::from_le_bytes(<WazaMoveRangeSettings as PackedStruct>::pack(v).unwrap())
    }
}

impl From<WazaMoveRangeSettings> for u16 {
    fn from(v: WazaMoveRangeSettings) -> Self {
        u16::from_le_bytes(<WazaMoveRangeSettings as PackedStruct>::pack(&v).unwrap())
    }
}

// WazaMoveRangeSettings but on the Python heap
/// (packable wrapper around Py<WazaMoveRangeSettings>
#[cfg_attr(feature = "python", derive(FromPyObject))]
#[derive(Clone, Debug)]
#[pyo3(transparent)]
#[repr(transparent)]
pub struct PyWazaMoveRangeSettings(Py<WazaMoveRangeSettings>);

impl PackedStruct for PyWazaMoveRangeSettings {
    type ByteArray = <WazaMoveRangeSettings as PackedStruct>::ByteArray;

    fn pack(&self) -> PackingResult<Self::ByteArray> {
        Python::with_gil(|py| {
            <WazaMoveRangeSettings as PackedStruct>::pack(self.0.borrow(py).deref())
        })
    }

    fn unpack(src: &Self::ByteArray) -> PackingResult<Self> {
        Python::with_gil(|py| {
            Ok(Self(
                Py::new(py, <WazaMoveRangeSettings as PackedStruct>::unpack(src)?)
                    .map_err(|_| PackingError::InternalError)?,
            ))
        })
    }
}

#[cfg(feature = "python")]
impl IntoPy<PyObject> for PyWazaMoveRangeSettings {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl PartialEq for PyWazaMoveRangeSettings {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| self.0.borrow(py).deref() == other.0.borrow(py).deref())
    }
}

impl Eq for PyWazaMoveRangeSettings {}

#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
#[pyclass(module = "skytemple_rust.st_waza_p")]
pub struct WazaMove {
    #[pyo3(get, set)]
    pub base_power: u16,
    #[packed_field(size_bytes = "1", ty = "enum")]
    #[pyo3(get, set)]
    pub r#type: PokeType,
    #[packed_field(size_bytes = "1", ty = "enum")]
    #[pyo3(get, set)]
    pub category: WazaMoveCategory,
    #[packed_field(size_bytes = "2")]
    #[pyo3(get, set)]
    pub settings_range: PyWazaMoveRangeSettings,
    #[packed_field(size_bytes = "2")]
    #[pyo3(get, set)]
    pub settings_range_ai: PyWazaMoveRangeSettings,
    #[pyo3(get, set)]
    pub base_pp: u8,
    #[pyo3(get, set)]
    pub ai_weight: u8,
    #[pyo3(get, set)]
    pub miss_accuracy: u8,
    #[pyo3(get, set)]
    pub accuracy: u8,
    #[pyo3(get, set)]
    pub ai_condition1_chance: u8,
    #[pyo3(get, set)]
    pub number_chained_hits: u8,
    #[pyo3(get, set)]
    pub max_upgrade_level: u8,
    #[pyo3(get, set)]
    pub crit_chance: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub affected_by_magic_coat: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub is_snatchable: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub uses_mouth: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub ai_frozen_check: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub ignores_taunted: bool,
    #[pyo3(get, set)]
    pub range_check_text: u8,
    #[pyo3(get, set)]
    pub move_id: u16,
    #[pyo3(get, set)]
    pub message_id: u16,
}

#[pymethods]
impl WazaMove {
    const BYTELEN: usize = 26;

    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        static_assert_size!(
            <WazaMove as PackedStruct>::ByteArray,
            /*Self::BYTELEN*/ 26
        );
        if data.len() < Self::BYTELEN {
            Err(exceptions::PyValueError::new_err(
                "Not enough data for WazaMove.",
            ))
        } else {
            <Self as PackedStruct>::unpack(data[..].try_into().unwrap())
                .map_err(convert_packing_err)
        }
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

impl From<Py<WazaMove>> for StBytes {
    fn from(v: Py<WazaMove>) -> Self {
        Python::with_gil(|py| {
            StBytes::from(&<WazaMove as PackedStruct>::pack(v.borrow(py).deref()).unwrap()[..])
        })
    }
}

impl_pylist!("skytemple_rust.st_waza_p", WazaMoveList, Py<WazaMove>);
impl_pylist!(
    "skytemple_rust.st_waza_p",
    MoveLearnsetList,
    Py<MoveLearnset>
);

#[pyclass(module = "skytemple_rust.st_waza_p")]
#[derive(Clone)]
pub struct WazaP {
    pub moves: Py<WazaMoveList>,
    pub learnsets: Py<MoveLearnsetList>,
}

#[pymethods]
impl WazaP {
    // TODO: Consider actually reading until the header later, in case modded games
    //       have added more moves.
    const MOVE_COUNT: usize = 559;
    const LEARNSET_TOCE_BYTELEN: usize = 12;
    const PNT_EOF: u32 = 0xAAAAAAAA;

    #[new]
    pub fn new(data: StBytes, waza_content_pointer: u32, py: Python) -> PyResult<Self> {
        let mut header = data.slice(waza_content_pointer as usize..);
        let move_data_pointer = header.get_u32_le();
        let move_learnset_pointer = header.get_u32_le();
        let move_len = Self::MOVE_COUNT * WazaMove::BYTELEN;

        let moves = data
            .slice(move_data_pointer as usize..move_data_pointer as usize + move_len)
            .chunks_exact(WazaMove::BYTELEN)
            .map(|c| {
                Py::new(
                    py,
                    <WazaMove as PackedStruct>::unpack(c.try_into().unwrap())
                        .map_err(convert_packing_err)?,
                )
            })
            .collect::<PyResult<Vec<Py<WazaMove>>>>()?;

        let mut learnset_slice =
            data.slice(move_learnset_pointer as usize..waza_content_pointer as usize);
        let mut learnsets = Vec::with_capacity(1000);
        loop {
            if learnset_slice.remaining() < Self::LEARNSET_TOCE_BYTELEN {
                break;
            }
            let mut list_pointers = learnset_slice.copy_to_bytes(Self::LEARNSET_TOCE_BYTELEN);
            let mut level_up = Vec::with_capacity(100);
            let mut tm_hm = vec![];
            let mut egg = vec![];

            let pointer_level_up = list_pointers.get_u32_le();
            let pointer_tm_hm = list_pointers.get_u32_le();
            let pointer_egg = list_pointers.get_u32_le();

            if pointer_level_up == Self::PNT_EOF
                || pointer_tm_hm == Self::PNT_EOF
                || pointer_egg == Self::PNT_EOF
            {
                break;
            }

            if pointer_level_up != 0 {
                let level_up_raw =
                    decode_sir0_pointer_offsets(data.clone(), pointer_level_up, false);
                for (move_id, level_id) in level_up_raw.iter().tuples() {
                    level_up.push(Py::new(
                        py,
                        LevelUpMove {
                            move_id: *move_id as u16,
                            level_id: *level_id as u16,
                        },
                    )?)
                }
            }

            if pointer_tm_hm != 0 {
                tm_hm = decode_sir0_pointer_offsets(data.clone(), pointer_tm_hm, false);
            }

            if pointer_egg != 0 {
                egg = decode_sir0_pointer_offsets(data.clone(), pointer_egg, false);
            }

            learnsets.push(Py::new(
                py,
                MoveLearnset {
                    level_up_moves: Py::new(py, LevelUpMoveList(level_up))?,
                    tm_hm_moves: Py::new(py, U32List(tm_hm))?,
                    egg_moves: Py::new(py, U32List(egg))?,
                },
            )?);
        }
        Ok(Self {
            moves: Py::new(py, WazaMoveList(moves))?,
            learnsets: Py::new(py, MoveLearnsetList(learnsets))?,
        })
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn moves(&self) -> Py<WazaMoveList> {
        self.moves.clone()
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_moves(&mut self, py: Python, value: PyObject) -> PyResult<()> {
        if let Ok(val) = value.extract::<Py<WazaMoveList>>(py) {
            self.moves = val;
            Ok(())
        } else {
            match value.extract::<Vec<Py<WazaMove>>>(py) {
                Ok(v) => {
                    self.moves = Py::new(py, WazaMoveList(v))?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn learnsets(&self) -> Py<MoveLearnsetList> {
        self.learnsets.clone()
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_learnsets(&mut self, py: Python, value: PyObject) -> PyResult<()> {
        if let Ok(val) = value.extract::<Py<MoveLearnsetList>>(py) {
            self.learnsets = val;
            Ok(())
        } else {
            match value.extract::<Vec<Py<MoveLearnset>>>(py) {
                Ok(v) => {
                    self.learnsets = Py::new(py, MoveLearnsetList(v))?;
                    Ok(())
                }
                Err(e) => Err(e),
            }
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

impl PartialEq for WazaP {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            self.moves.borrow(py).deref() == other.moves.borrow(py).deref()
                && self.learnsets.borrow(py).deref() == other.learnsets.borrow(py).deref()
        })
    }
}

impl Eq for WazaP {}

impl Sir0Serializable for WazaP {
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<u32>, Option<u32>)> {
        Python::with_gil(|py| {
            let mut data = BytesMut::with_capacity(131072);
            data.put_u32_le(0);

            let mut pointer_offsets = Vec::with_capacity(4096);
            // Learnset
            let self_learnsets: &[Py<MoveLearnset>] = &self.learnsets.borrow(py).0[..];
            let mut learnset_pointers = Vec::with_capacity(self_learnsets.len());
            for pylearnset in self_learnsets {
                let learnset = pylearnset.borrow(py);
                // Level Up
                let lvl_up_move_list: Vec<_> = learnset
                    .level_up_moves
                    .borrow(py)
                    .0
                    .iter()
                    .flat_map(|pye| {
                        let e = pye.borrow(py);
                        [e.move_id as u32, e.level_id as u32]
                    })
                    .collect::<Vec<u32>>();
                let pnt_lvlup = if lvl_up_move_list.is_empty() {
                    0
                } else {
                    data.len() as u32
                };
                data.extend_from_slice(&encode_sir0_pointer_offsets(lvl_up_move_list, false)?[..]);

                // TM/HM
                let tm_hm_brw = learnset.tm_hm_moves.borrow(py);
                let pnt_tm_hm = if tm_hm_brw.0.is_empty() {
                    0
                } else {
                    data.len() as u32
                };
                data.extend_from_slice(
                    &encode_sir0_pointer_offsets(tm_hm_brw.0.iter().copied(), false)?[..],
                );

                // Egg
                let egg_brw = learnset.egg_moves.borrow(py);
                let pnt_egg = if egg_brw.0.is_empty() {
                    0
                } else {
                    data.len() as u32
                };
                data.extend_from_slice(
                    &encode_sir0_pointer_offsets(egg_brw.0.iter().copied(), false)?[..],
                );

                learnset_pointers.push((pnt_lvlup, pnt_tm_hm, pnt_egg))
            }

            // Padding
            pad(&mut data, 16, 0xAA);

            // Move data
            let move_pointer = data.len() as u32;
            let self_moves: &[Py<WazaMove>] = &self.moves.borrow(py).0[..];
            data.extend(self_moves.iter().flat_map(AsStBytes::as_bytes));

            // Padding
            pad(&mut data, 16, 0xAA);

            // Learnset pointer table
            let learnset_pointer_table_pnt = data.len() as u32;
            for (lvlup, tm_hm, egg) in learnset_pointers {
                pointer_offsets.push(data.len() as u32);
                data.put_u32_le(lvlup);
                pointer_offsets.push(data.len() as u32);
                data.put_u32_le(tm_hm);
                pointer_offsets.push(data.len() as u32);
                data.put_u32_le(egg);
            }

            // Padding
            pad(&mut data, 16, 0xAA);

            // Waza Header (<- content pointer)
            let waza_header_start = data.len() as u32;
            pointer_offsets.push(data.len() as u32);
            data.put_u32_le(move_pointer);
            pointer_offsets.push(data.len() as u32);
            data.put_u32_le(learnset_pointer_table_pnt);

            // Padding
            pad(&mut data, 16, 0xAA);

            // Check if data.len() fits into u32. If it doesn't, one of the pointer offsets above
            // and before that will have overflown...
            u32::try_from(data.len()).map_err(convert_try_from_int)?;

            Ok((data.into(), pointer_offsets, Some(waza_header_start)))
        })
    }

    fn sir0_unwrap(content_data: StBytes, data_pointer: u32) -> Sir0Result<Self> {
        Python::with_gil(|py| Self::new(content_data, data_pointer, py))
            .map_err(|e| Sir0Error::UnwrapFailed(anyhow::Error::from(e)))
    }
}

#[pyclass(module = "skytemple_rust.st_waza_p")]
#[derive(Clone, Default)]
pub struct WazaPWriter;

#[pymethods]
impl WazaPWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<WazaP>, py: Python) -> PyResult<StBytes> {
        model
            .borrow(py)
            .sir0_serialize_parts()
            .map(|(c, _, _)| c)
            .map_err(|e| exceptions::PyValueError::new_err(format!("{}", e)))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_waza_p_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_waza_p";
    let m = PyModule::new(py, name)?;
    m.add_class::<LevelUpMove>()?;
    m.add_class::<LevelUpMoveList>()?;
    m.add_class::<U32List>()?;
    m.add_class::<MoveLearnset>()?;
    m.add_class::<MoveLearnsetList>()?;
    m.add_class::<WazaMoveRangeSettings>()?;
    m.add_class::<WazaMove>()?;
    m.add_class::<WazaMoveList>()?;
    m.add_class::<WazaP>()?;

    Ok((name, m))
}

fn convert_try_from_int(e: TryFromIntError) -> Sir0Error {
    Sir0Error::SerializeFailed(e.into())
}
