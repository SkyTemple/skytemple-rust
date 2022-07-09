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
use std::ops::Deref;

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
pub struct MappaFloorTerrainSettings {
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub has_secondary_terrain: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk1: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub generate_imperfect_rooms: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk3: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk4: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk5: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk6: bool,
    #[pyo3(get, set)]
    #[packed_field(size_bits = "1")]
    pub unk7: bool,
}

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
    pub terrain_settings: MappaFloorTerrainSettings,
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
        terrain_settings: MappaFloorTerrainSettings,
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
            terrain_settings,
            unk_e,
            item_density,
            trap_density,
            floor_number,
            fixed_floor_id,
            extra_hallway_density,
            buried_item_density,
            water_density,
            darkness_level,
            _max_coin_amount_raw: (max_coin_amount / 5).try_into()?,
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

    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject, py: Python) -> bool {
        if let Ok(other) = other.extract::<Py<Self>>(py) {
            self == other.borrow(py).deref()
        } else {
            false
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
                MappaFloorLayout::unpack_from_slice(&value[..]).map_err(convert_packing_err)?,
            )
        })
    }
}

impl From<Py<MappaFloorLayout>> for StBytes {
    fn from(value: Py<MappaFloorLayout>) -> Self {
        Python::with_gil(|py| StBytes::from(&value.borrow(py).pack().unwrap()[..]))
    }
}
