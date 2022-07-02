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
use std::collections::BTreeMap;

#[derive(EnumToPy_u8, PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum MappaFloorStructureType {
    MediumLarge = 0,
    Small = 1,
    SingleMonsterHouse = 2,
    Ring = 3,
    Crossroads = 4,
    TwoRoomsOneMonsterHouse = 5,
    Line = 6,
    Cross = 7,
    SmallMedium = 8,
    Beetle = 9,
    OuterRooms = 10,
    Medium = 11,
    MediumLarge12 = 12,
    MediumLarge13 = 13,
    MediumLarge14 = 14,
    MediumLarge15 = 15,
}

#[derive(EnumToPy_u8, PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum MappaFloorWeather {
    Clear = 0,
    Sunny = 1,
    Sandstorm = 2,
    Cloudy = 3,
    Rainy = 4,
    Hail = 5,
    Fog = 6,
    Snow = 7,
    Random = 8,
}

#[derive(EnumToPy_u8, PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum MappaFloorDarknessLevel {
    NoDarkness = 0,
    HeavyDarkness = 1,
    LightDarkness = 2,
    ThreeTile = 3,
    FourTile = 4,
}

#[derive(EnumToPy_u8, PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum MappaTrapType {
    Unused = 0,
    MudTrap = 1,
    StickyTrap = 2,
    GrimyTrap = 3,
    SummonTrap = 4,
    PitfallTrap = 5,
    WarpTrap = 6,
    GustTrap = 7,
    SpinTrap = 8,
    SlumberTrap = 9,
    SlowTrap = 10,
    SealTrap = 11,
    PoisonTrap = 12,
    SelfdestructTrap = 13,
    ExplosionTrap = 14,
    PpZeroTrap = 15,
    ChestnutTrap = 16,
    WonderTile = 17,
    MonsterTrap = 18,
    SpikedTile = 19,
    StealthRock = 20,
    ToxicSpikes = 21,
    TripTrap = 22,
    RandomTrap = 23,
    GrudgeTrap = 24,
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq, Eq)]
pub struct MappaTrapList {
    weights: BTreeMap<MappaTrapType, u16>,
}

impl MappaTrapList {
    pub fn new(weights: BTreeMap<MappaTrapType, u16>) -> Self {
        todo!()
    }
}

#[pymethods]
impl MappaTrapList {
    #[cfg(feature = "python")]
    #[new]
    pub fn _new(weights: PyObject) -> Self {
        // weights: Union[List[u16], Dict[_MappaTrapType, u16]]
        todo!()
    }

    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject) -> bool {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
pub struct MappaMonster {
    pub level: u8,
    pub weight: u16,
    pub weight2: u16,
    pub md_index: u16,
}

#[pymethods]
impl MappaMonster {
    #[new]
    pub fn new(level: u8, weight: u16, weight2: u16, md_index: u16) -> Self {
        todo!()
    }

    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject) -> bool {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq, Eq)]
pub struct MappaItemList {
    // todo probably not usize
    pub categories: BTreeMap<usize, usize>,
    pub items: BTreeMap<usize, usize>,
}

#[pymethods]
impl MappaItemList {
    #[new]
    pub fn new(categories: BTreeMap<usize, usize>, items: BTreeMap<usize, usize>) -> Self {
        todo!()
    }

    #[classmethod]
    pub fn from_bytes(cls: &PyType, item_list: Vec<usize>, pointer: usize) -> Self {
        todo!()
    }

    pub fn to_bytes(&self) -> StBytes {
        todo!()
    }

    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject) -> bool {
        todo!()
    }
}

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

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
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
    pub unusued_chance: u8,
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
        unusued_chance: u8,
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
        py: Python,
    ) -> PyResult<Self> {
        // _max_coin_amount = max_coin_amount // 5
        todo!()
    }

    #[getter]
    pub fn max_coin_amount(&self) -> u16 {
        todo!()
    }

    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject) -> bool {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq)]
pub struct MappaFloor {
    #[pyo3(get, set)]
    pub layout: Py<MappaFloorLayout>,
    #[pyo3(get, set)]
    pub monsters: Vec<Py<MappaMonster>>,
    #[pyo3(get, set)]
    pub traps: Py<MappaTrapList>,
    #[pyo3(get, set)]
    pub floor_items: Py<MappaItemList>,
    #[pyo3(get, set)]
    pub shop_items: Py<MappaItemList>,
    #[pyo3(get, set)]
    pub monster_house_items: Py<MappaItemList>,
    #[pyo3(get, set)]
    pub buried_items: Py<MappaItemList>,
    #[pyo3(get, set)]
    pub unk_items1: Py<MappaItemList>,
    #[pyo3(get, set)]
    pub unk_items2: Py<MappaItemList>,
}

#[pymethods]
impl MappaFloor {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layout: Py<MappaFloorLayout>,
        monsters: Vec<Py<MappaMonster>>,
        traps: Py<MappaTrapList>,
        floor_items: Py<MappaItemList>,
        shop_items: Py<MappaItemList>,
        monster_house_items: Py<MappaItemList>,
        buried_items: Py<MappaItemList>,
        unk_items1: Py<MappaItemList>,
        unk_items2: Py<MappaItemList>,
        py: Python,
    ) -> PyResult<Self> {
        todo!()
    }

    #[cfg(feature = "python")]
    pub fn __eq__(&self, other: PyObject) -> bool {
        todo!()
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
    pub fn new(data: StBytes, pointer_to_pointers: usize, py: Python) -> PyResult<Self> {
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
        data_pointer: usize,
        static_data: InStaticData,
    ) -> PyResult<Self> {
        Ok(Self::sir0_unwrap(content_data, data_pointer, static_data)?)
    }
}

impl Sir0Serializable for MappaBin {
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<usize>, Option<usize>)> {
        todo!()
    }

    fn sir0_unwrap(
        content_data: StBytes,
        data_pointer: usize,
        static_data: InStaticData,
    ) -> Sir0Result<Self> {
        todo!()
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
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_mappa_bin_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_mappa_bin";
    let m = PyModule::new(py, name)?;
    m.add_class::<MappaTrapList>()?;
    m.add_class::<MappaMonster>()?;
    m.add_class::<MappaItemList>()?;
    m.add_class::<MappaFloorTerrainSettings>()?;
    m.add_class::<MappaFloorLayout>()?;
    m.add_class::<MappaFloor>()?;
    m.add_class::<MappaBin>()?;
    m.add_class::<MappaBinWriter>()?;

    Ok((name, m))
}
