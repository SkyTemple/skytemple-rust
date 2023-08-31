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
use crate::st_mappa_bin::{MappaFloorDarknessLevel, MappaFloorStructureType, MappaFloorWeather};
use packed_struct::prelude::*;
use packed_struct::PackingResult;
use std::ops::Deref;

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
pub struct MappaFloorTerrainSettings {
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk7: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk6: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk5: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk4: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk3: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub generate_imperfect_rooms: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk1: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub has_secondary_terrain: bool,
}

#[pymethods]
impl MappaFloorTerrainSettings {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        has_secondary_terrain: bool,
        unk1: bool,
        generate_imperfect_rooms: bool,
        unk3: bool,
        unk4: bool,
        unk5: bool,
        unk6: bool,
        unk7: bool,
    ) -> Self {
        Self {
            has_secondary_terrain,
            unk1,
            generate_imperfect_rooms,
            unk3,
            unk4,
            unk5,
            unk6,
            unk7,
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

/// MappaFloorTerrainSettings but on the Python heap
/// (packable wrapper around Py<MappaFloorTerrainSettings>
#[cfg_attr(feature = "python", derive(FromPyObject))]
#[derive(Clone, Debug)]
#[pyo3(transparent)]
#[repr(transparent)]
pub struct PyMappaFloorTerrainSettings(Py<MappaFloorTerrainSettings>);

impl PackedStruct for PyMappaFloorTerrainSettings {
    type ByteArray = <MappaFloorTerrainSettings as PackedStruct>::ByteArray;

    fn pack(&self) -> PackingResult<Self::ByteArray> {
        Python::with_gil(|py| {
            <MappaFloorTerrainSettings as PackedStruct>::pack(self.0.borrow(py).deref())
        })
    }

    fn unpack(src: &Self::ByteArray) -> PackingResult<Self> {
        Python::with_gil(|py| {
            Ok(Self(
                Py::new(
                    py,
                    <MappaFloorTerrainSettings as PackedStruct>::unpack(src)?,
                )
                .map_err(|_| PackingError::InternalError)?,
            ))
        })
    }
}

#[cfg(feature = "python")]
impl IntoPy<PyObject> for PyMappaFloorTerrainSettings {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

impl PartialEq for PyMappaFloorTerrainSettings {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| self.0.borrow(py).deref() == other.0.borrow(py).deref())
    }
}

impl Eq for PyMappaFloorTerrainSettings {}

#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
#[pyclass(module = "skytemple_rust.st_mappa_bin")]
pub struct MappaFloorLayout {
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub structure: MappaFloorStructureType,
    #[pyo3(get, set)]
    pub room_density: i8,
    #[pyo3(get, set)]
    pub tileset_id: u8,
    #[pyo3(get, set)]
    pub music_id: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub weather: MappaFloorWeather,
    #[pyo3(get, set)]
    pub floor_connectivity: u8,
    #[pyo3(get, set)]
    pub initial_enemy_density: i8,
    #[pyo3(get, set)]
    pub kecleon_shop_chance: u8,
    #[pyo3(get, set)]
    pub monster_house_chance: u8,
    #[pyo3(get, set)]
    pub unused_chance: u8,
    #[pyo3(get, set)]
    pub sticky_item_chance: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub dead_ends: bool,
    #[pyo3(get, set)]
    pub secondary_terrain: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub terrain_settings: PyMappaFloorTerrainSettings,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1")]
    pub unk_e: bool,
    #[pyo3(get, set)]
    pub item_density: u8,
    #[pyo3(get, set)]
    pub trap_density: u8,
    #[pyo3(get, set)]
    pub floor_number: u8,
    #[pyo3(get, set)]
    pub fixed_floor_id: u8,
    #[pyo3(get, set)]
    pub extra_hallway_density: u8,
    #[pyo3(get, set)]
    pub buried_item_density: u8,
    #[pyo3(get, set)]
    pub water_density: u8,
    #[pyo3(get, set)]
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub darkness_level: MappaFloorDarknessLevel,
    _max_coin_amount_raw: u8,
    #[pyo3(get, set)]
    pub kecleon_shop_item_positions: u8,
    #[pyo3(get, set)]
    pub empty_monster_house_chance: u8,
    #[pyo3(get, set)]
    pub unk_hidden_stairs: u8,
    #[pyo3(get, set)]
    pub hidden_stairs_spawn_chance: u8,
    #[pyo3(get, set)]
    pub enemy_iq: u16,
    #[pyo3(get, set)]
    pub iq_booster_boost: i16,
}

#[pymethods]
impl MappaFloorLayout {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        structure: MappaFloorStructureType,
        room_density: i8,
        tileset_id: u8,
        music_id: u8,
        weather: MappaFloorWeather,
        floor_connectivity: u8,
        initial_enemy_density: i8,
        kecleon_shop_chance: u8,
        monster_house_chance: u8,
        unused_chance: u8,
        sticky_item_chance: u8,
        dead_ends: bool,
        secondary_terrain: u8,
        terrain_settings: Py<MappaFloorTerrainSettings>,
        unk_e: bool,
        item_density: u8,
        trap_density: u8,
        floor_number: u8,
        fixed_floor_id: u8,
        extra_hallway_density: u8,
        buried_item_density: u8,
        water_density: u8,
        darkness_level: MappaFloorDarknessLevel,
        max_coin_amount: u16,
        kecleon_shop_item_positions: u8,
        empty_monster_house_chance: u8,
        unk_hidden_stairs: u8,
        hidden_stairs_spawn_chance: u8,
        enemy_iq: u16,
        iq_booster_boost: i16,
    ) -> PyResult<Self> {
        Ok(Self {
            structure,
            room_density,
            tileset_id,
            music_id,
            weather,
            floor_connectivity,
            initial_enemy_density,
            kecleon_shop_chance,
            monster_house_chance,
            unused_chance,
            sticky_item_chance,
            dead_ends,
            secondary_terrain,
            terrain_settings: PyMappaFloorTerrainSettings(terrain_settings),
            unk_e,
            item_density,
            trap_density,
            floor_number,
            fixed_floor_id,
            extra_hallway_density,
            buried_item_density,
            water_density,
            darkness_level,
            _max_coin_amount_raw: (max_coin_amount / 5)
                .try_into()
                .map_err(|_| exceptions::PyValueError::new_err("Coin amount too big."))?,
            kecleon_shop_item_positions,
            empty_monster_house_chance,
            unk_hidden_stairs,
            hidden_stairs_spawn_chance,
            enemy_iq,
            iq_booster_boost,
        })
    }

    #[getter]
    pub fn max_coin_amount(&self) -> u16 {
        self._max_coin_amount_raw as u16 * 5
    }

    #[setter]
    pub fn set_max_coin_amount(&mut self, value: u16) -> PyResult<()> {
        self._max_coin_amount_raw = u8::try_from(value / 5)
            .map_err(|_| exceptions::PyValueError::new_err("Coin amount too big."))?;
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

impl TryFrom<StBytes> for Py<MappaFloorLayout> {
    type Error = PyErr;

    fn try_from(value: StBytes) -> Result<Self, Self::Error> {
        static_assert_size!(<MappaFloorLayout as PackedStruct>::ByteArray, 32);
        Python::with_gil(|py| {
            Py::new(
                py,
                MappaFloorLayout::unpack_from_slice(&value[..32]).map_err(convert_packing_err)?,
            )
        })
    }
}

impl From<Py<MappaFloorLayout>> for StBytes {
    fn from(value: Py<MappaFloorLayout>) -> Self {
        Python::with_gil(|py| StBytes::from(&value.borrow(py).pack().unwrap()[..]))
    }
}
