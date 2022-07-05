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
use crate::st_sir0::{Sir0Error, Sir0Result, Sir0Serializable};
use crate::util::Lazy;
use bytes::Buf;
use packed_struct::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;

const GUARANTEED: u16 = 0xFFFF;

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "python", derive(EnumToPy_u16))]
pub enum Probability {
    Percentage(u16), // as fixed int.
    Guaranteed,
}

impl From<Probability> for u16 {
    fn from(prob: Probability) -> Self {
        match prob {
            Probability::Percentage(v) => v,
            Probability::Guaranteed => GUARANTEED,
        }
    }
}

impl From<u16> for Probability {
    fn from(v: u16) -> Self {
        match v {
            GUARANTEED => Probability::Guaranteed,
            vv => Probability::Percentage(vv),
        }
    }
}

impl PrimitiveEnum for Probability {
    type Primitive = u16;

    fn from_primitive(val: Self::Primitive) -> Option<Self> {
        Some(val.into())
    }

    fn to_primitive(&self) -> Self::Primitive {
        (*self).into()
    }

    fn from_str(_s: &str) -> Option<Self> {
        // Not available for Probability.
        None
    }

    fn from_str_lower(_s: &str) -> Option<Self> {
        // Not available for Probability.
        None
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
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

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
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

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum MappaFloorDarknessLevel {
    NoDarkness = 0,
    HeavyDarkness = 1,
    LightDarkness = 2,
    ThreeTile = 3,
    FourTile = 4,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
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
    fn collect_floor_lists(&self) -> PyResult<Vec<Vec<Py<MappaFloor>>>> {
        //         start = read.dungeon_list_index_start
        //         end = read.floor_layout_data_start
        //         dungeons = []
        //         for i in range(start, end, 4):
        //             dungeons.append(cls._read_floors(read, read_u32(read.data, i)))
        //         return dungeons
        todo!()
    }

    fn collect_floor_list(&self, pointer: usize) -> PyResult<Vec<Py<MappaFloor>>> {
        //         # The zeroth floor is just nulls, we omit it.
        //         empty = bytes(FLOOR_IDX_ENTRY_LEN)
        //         assert (
        //             read.data[pointer : pointer + FLOOR_IDX_ENTRY_LEN] == empty
        //         ), "The first floor of a dungeon must be a null floor."
        //         floors = []
        //         pointer += FLOOR_IDX_ENTRY_LEN
        //         floor_data = read.data[pointer : pointer + FLOOR_IDX_ENTRY_LEN]
        //         while floor_data != empty:
        //             floors.append(MappaFloor.from_mappa(read, floor_data))
        //             pointer += FLOOR_IDX_ENTRY_LEN
        //             floor_data = read.data[pointer : pointer + FLOOR_IDX_ENTRY_LEN]
        //             if pointer > read.dungeon_list_index_start - FLOOR_IDX_ENTRY_LEN:
        //                 break
        //         return floors
        todo!()
    }

    fn collect_floor(&self, floor_data: StBytes) -> PyResult<Py<MappaFloor>> {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq, Eq)]
pub struct MappaTrapList {
    #[pyo3(get, set)]
    pub weights: HashMap<MappaTrapType, u16>,
}

impl MappaTrapList {
    pub fn new(weights: HashMap<MappaTrapType, u16>) -> Self {
        Self { weights }
    }
}

impl TryFrom<StBytes> for MappaTrapList {
    type Error = PyErr;

    fn try_from(value: StBytes) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<MappaTrapList> for StBytes {
    fn from(value: MappaTrapList) -> Self {
        todo!()
    }
}

#[pymethods]
impl MappaTrapList {
    #[cfg(feature = "python")]
    #[new]
    pub fn _new(weights: &PyAny) -> PyResult<Self> {
        // weights: Union[List[u16], Dict[_MappaTrapType, u16]]
        if let Ok(dw) = weights.downcast::<pyo3::types::PyDict>() {
            let weights_c = dw
                .into_iter()
                .map(|(k, v)| {
                    if let Ok(kk) = k.extract::<MappaTrapType>() {
                        if let Ok(vv) = v.extract::<u16>() {
                            return Ok((kk, vv));
                        }
                    }
                    Err(exceptions::PyValueError::new_err(
                        "Invalid key(s) or value(s) for trap dict.",
                    ))
                })
                .collect::<PyResult<HashMap<MappaTrapType, u16>>>()?;
            if weights_c.len() != 25 {
                Err(exceptions::PyValueError::new_err(
                    "MappaTrapList constructor needs a weight value for all of the 25 traps.",
                ))
            } else {
                Ok(Self::new(weights_c))
            }
        } else if let Ok(dl) = weights.downcast::<pyo3::types::PyList>() {
            if dl.len() != 25 {
                Err(exceptions::PyValueError::new_err(
                    "MappaTrapList constructor needs a weight value for all of the 25 traps.",
                ))
            } else {
                Ok(Self::new(
                    dl.into_iter()
                        .enumerate()
                        .map(|(i, v)| {
                            if let Ok(vv) = v.extract::<u16>() {
                                Ok((MappaTrapType::from_primitive(i as u8).unwrap(), vv))
                            } else {
                                Err(exceptions::PyValueError::new_err(
                                    "Invalid value(s) for trap list.",
                                ))
                            }
                        })
                        .collect::<PyResult<HashMap<MappaTrapType, u16>>>()?,
                ))
            }
        } else {
            Err(exceptions::PyTypeError::new_err(
                "The weights must be a list or dict of probabilities.",
            ))
        }
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

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PackedStruct, Debug, PartialEq, Eq)]
#[packed_struct(endian = "lsb")]
pub struct MappaMonster {
    pub level: u8,
    pub weight: u16,
    pub weight2: u16,
    pub md_index: u16,
}

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, Debug, PartialEq)]
pub struct MappaMonsterList {
    pub list: Vec<Py<MappaMonster>>,
}
// TODO: Python MutableSequence impl.

impl TryFrom<StBytes> for MappaMonsterList {
    type Error = PyErr;

    fn try_from(value: StBytes) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<MappaMonsterList> for StBytes {
    fn from(value: MappaMonsterList) -> Self {
        todo!()
    }
}

#[pymethods]
impl MappaMonster {
    #[new]
    pub fn new(level: u8, weight: u16, weight2: u16, md_index: u16) -> Self {
        Self {
            level,
            weight,
            weight2,
            md_index,
        }
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

#[pyclass(module = "skytemple_rust.st_mappa_bin")]
#[derive(Clone, PartialEq, Eq)]
pub struct MappaItemList {
    #[pyo3(get, set)]
    pub categories: HashMap<usize, u16>,
    #[pyo3(get, set)]
    pub items: HashMap<usize, u16>,
}

#[pymethods]
impl MappaItemList {
    #[new]
    pub fn new(categories: HashMap<usize, u16>, items: HashMap<usize, u16>) -> Self {
        Self { categories, items }
    }

    #[classmethod]
    pub fn from_bytes(cls: &PyType, pointer: usize) -> Self {
        // self-> try from
        todo!()
    }

    pub fn to_bytes(&self) -> StBytes {
        // from <- self
        todo!()
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

impl TryFrom<StBytes> for MappaItemList {
    type Error = PyErr;

    fn try_from(value: StBytes) -> Result<Self, Self::Error> {
        //         processing_categories = True
        //         item_or_cat_id = 0
        //         orig_pointer = pointer
        //         len_read = 0
        //
        //         items = {}
        //         categories = {}
        //
        //         while item_or_cat_id <= MAX_ITEM_ID:
        //             val = read_u16(data, pointer)
        //             len_read += 2
        //             skip = val > CMD_SKIP and val != GUARANTEED
        //
        //             if skip:
        //                 item_or_cat_id += val - CMD_SKIP
        //             else:
        //                 if val == GUARANTEED:
        //                     weight = GUARANTEED
        //                 else:
        //                     weight = val
        //                 if processing_categories:
        //                     # TODO: Switch to Pmd2DungeonItemCategory
        //                     categories[item_or_cat_id] = weight
        //                 else:
        //                     items[item_list[item_or_cat_id]] = weight
        //                 item_or_cat_id += 1
        //             if item_or_cat_id >= 0xF and processing_categories:
        //                 processing_categories = False
        //                 item_or_cat_id -= 0x10
        //             pointer += 2
        //
        //         assert (
        //             data[orig_pointer : orig_pointer + len_read]
        //             == MappaItemList(categories, items).to_mappa()
        //         )
        //
        //         return MappaItemList(categories, items)
        todo!()
    }
}

impl From<MappaItemList> for StBytes {
    fn from(value: MappaItemList) -> Self {
        //         data = bytearray()
        //         current_id = 0
        //         # Start with the categories
        //         for cat, val in sorted(self.categories.items(), key=lambda it: it[0]):
        //             id_cat = cat
        //             if current_id != id_cat:
        //                 current_id = self._write_skip(data, current_id, id_cat)
        //             self._write_probability(data, val)
        //             current_id += 1
        //         # Continue with the items
        //         sorted_items = sorted(self.items.items(), key=lambda it: it[0])
        //         first_item_id = sorted_items[0][0] if len(sorted_items) > 0 else 0
        //         self._write_skip(data, current_id, 0x10 + first_item_id)
        //         current_id = first_item_id
        //         for item, val in sorted_items:
        //             if current_id != item:
        //                 current_id = self._write_skip(data, current_id, item)
        //             self._write_probability(data, val)
        //             current_id += 1
        //         # Fill up to MAX_ITEM_ID + 1
        //         self._write_skip(data, current_id, MAX_ITEM_ID + 1)
        //         return data
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

impl TryFrom<StBytes> for MappaFloorLayout {
    type Error = PyErr;

    fn try_from(value: StBytes) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<MappaFloorLayout> for StBytes {
    fn from(value: MappaFloorLayout) -> Self {
        todo!()
    }
}

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
            monsters: Lazy::Instance(Py::new(py, MappaMonsterList { list: monsters })?),
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
    pub fn layout(&mut self) -> PyResult<Py<MappaFloorLayout>> {
        Ok(self.layout.instance()?.clone())
    }

    #[setter]
    pub fn set_layout(&mut self, value: Py<MappaFloorLayout>) -> PyResult<()> {
        self.layout = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    pub fn monsters(&mut self) -> PyResult<Py<MappaMonsterList>> {
        Ok(self.monsters.instance()?.clone())
    }

    #[setter]
    pub fn set_monsters(&mut self, py: Python, value: Vec<Py<MappaMonster>>) -> PyResult<()> {
        self.monsters = Lazy::Instance(Py::new(py, MappaMonsterList { list: value })?);
        Ok(())
    }

    #[getter]
    pub fn traps(&mut self) -> PyResult<Py<MappaTrapList>> {
        Ok(self.traps.instance()?.clone())
    }

    #[setter]
    pub fn set_traps(&mut self, value: Py<MappaTrapList>) -> PyResult<()> {
        self.traps = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    pub fn floor_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.floor_items.instance()?.clone())
    }

    #[setter]
    pub fn set_floor_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.floor_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    pub fn shop_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.shop_items.instance()?.clone())
    }

    #[setter]
    pub fn set_shop_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.shop_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    pub fn monster_house_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.monster_house_items.instance()?.clone())
    }

    #[setter]
    pub fn set_monster_house_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.monster_house_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    pub fn buried_items(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.buried_items.instance()?.clone())
    }

    #[setter]
    pub fn set_buried_items(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.buried_items = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    pub fn unk_items1(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.unk_items1.instance()?.clone())
    }

    #[setter]
    pub fn set_unk_items1(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.unk_items1 = Lazy::Instance(value);
        Ok(())
    }

    #[getter]
    pub fn unk_items2(&mut self) -> PyResult<Py<MappaItemList>> {
        Ok(self.unk_items2.instance()?.clone())
    }

    #[setter]
    pub fn set_unk_items2(&mut self, value: Py<MappaItemList>) -> PyResult<()> {
        self.unk_items2 = Lazy::Instance(value);
        Ok(())
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
}

impl Sir0Serializable for MappaBin {
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<u32>, Option<u32>)> {
        //         """Returns the content and the offsets to the pointers and the sub-header pointer, for Sir0 serialization."""
        //         pointer_offsets = []
        //
        //         (
        //             floor_lists,
        //             floor_layouts,
        //             monster_lists,
        //             trap_lists,
        //             item_lists,
        //         ) = self.minimize()
        //         # Floor list data
        //         data = bytearray(sum((len(floor_list) + 1) * 18 for floor_list in floor_lists))
        //         cursor = 0
        //         for floor_list in floor_lists:
        //             cursor += 18  # null floor
        //             for floor in floor_list:
        //                 data[cursor : cursor + 18] = floor.to_mappa()
        //                 cursor += 18
        //         # Padding
        //         if len(data) % 16 != 0:
        //             data += bytes(0x00 for _ in range(0, 16 - (len(data) % 16)))
        //         # Floor list LUT
        //         start_floor_list_lut = u32_checked(len(data))
        //         floor_list_lut = bytearray(4 * len(floor_lists))
        //         cursor_floor_data = u32(0)
        //         for i, floor_list in enumerate(floor_lists):
        //             pointer_offsets.append(u32_checked(start_floor_list_lut + i * 4))
        //             write_u32(floor_list_lut, cursor_floor_data, i * 4)
        //             cursor_floor_data = u32_checked(
        //                 cursor_floor_data + (len(floor_list) + 1) * 18
        //             )
        //         data += floor_list_lut
        //         # Padding
        //         if len(data) % 4 != 0:
        //             data += bytes(0xAA for _ in range(0, 4 - (len(data) % 4)))
        //         # Floor layout data
        //         start_floor_layout_data = u32_checked(len(data))
        //         layout_data = bytearray(32 * len(floor_layouts))
        //         for i, layout in enumerate(floor_layouts):
        //             layout_data[i * 32 : (i + 1) * 32] = layout.to_mappa()
        //         data += layout_data
        //         # Padding
        //         if len(data) % 4 != 0:
        //             data += bytes(0xAA for _ in range(0, 4 - (len(data) % 4)))
        //         # Monster spawn data
        //         monster_data_start = len(data)
        //         monster_data = bytearray(
        //             sum((len(monsters) + 1) * 8 for monsters in monster_lists)
        //         )
        //         monster_data_cursor = 0
        //         monster_data_pointer = []
        //         for i, monster_list in enumerate(monster_lists):
        //             monster_data_pointer.append(
        //                 u32_checked(monster_data_start + monster_data_cursor)
        //             )
        //
        //             single_monster_list_data = bytes(
        //                 chain.from_iterable(monster.to_mappa() for monster in monster_list)
        //             ) + bytes(8)
        //             len_single = len(single_monster_list_data)
        //             monster_data[
        //                 monster_data_cursor : monster_data_cursor + len_single
        //             ] = single_monster_list_data
        //             monster_data_cursor += len_single
        //         data += monster_data
        //         # Padding
        //         if len(data) % 4 != 0:
        //             data += bytes(0xAA for _ in range(0, 4 - (len(data) % 4)))
        //         # Monster spawn LUT
        //         start_monster_lut = u32_checked(len(data))
        //         monster_lut = bytearray(4 * len(monster_data_pointer))
        //         for i, pnt in enumerate(monster_data_pointer):
        //             pointer_offsets.append(u32_checked(start_monster_lut + i * 4))
        //             write_u32(monster_lut, pnt, i * 4)
        //         data += monster_lut
        //         # Padding
        //         if len(data) % 4 != 0:
        //             data += bytes(0xAA for _ in range(0, 4 - (len(data) % 4)))
        //         # Trap lists data
        //         trap_data_start = len(data)
        //         trap_data = bytearray(len(trap_lists) * 50)
        //         trap_data_cursor = 0
        //         trap_data_pointer = []
        //         for trap_list in trap_lists:
        //             trap_data_pointer.append(u32_checked(trap_data_start + trap_data_cursor))
        //             single_trap_list_data = trap_list.to_mappa()
        //
        //             len_single = len(single_trap_list_data)
        //             assert len_single == 50
        //             trap_data[
        //                 trap_data_cursor : trap_data_cursor + len_single
        //             ] = single_trap_list_data
        //             trap_data_cursor += len_single
        //         assert trap_data_cursor == len(trap_lists) * 50
        //         data += trap_data
        //         # Padding
        //         if len(data) % 16 != 0:
        //             data += bytes(0xAA for _ in range(0, 16 - (len(data) % 16)))
        //         # Trap lists LUT
        //         start_traps_lut = u32_checked(len(data))
        //         trap_lut = bytearray(4 * len(trap_data_pointer))
        //         for i, pnt in enumerate(trap_data_pointer):
        //             pointer_offsets.append(u32_checked(start_traps_lut + i * 4))
        //             write_u32(trap_lut, pnt, i * 4)
        //         data += trap_lut
        //         # Item spawn lists data
        //         item_data_start = len(data)
        //         # TODO: I don't need to explain why a fixed size here per list is flawed.
        //         item_data = bytearray((len(item_lists) * 500))
        //         item_data_cursor = 0
        //         item_data_pointer = []
        //         for item_list in item_lists:
        //             item_data_pointer.append(u32_checked(item_data_start + item_data_cursor))
        //
        //             single_item_list_data = item_list.to_mappa()
        //             len_single = len(single_item_list_data)
        //             assert item_data_cursor + len_single < len(item_data)
        //             item_data[
        //                 item_data_cursor : item_data_cursor + len_single
        //             ] = single_item_list_data
        //             item_data_cursor += len_single
        //         data += item_data[:item_data_cursor]
        //         # Padding
        //         if len(data) % 16 != 0:
        //             data += bytes(0xAA for _ in range(0, 16 - (len(data) % 16)))
        //         # Item spawn lists LUT
        //         start_items_lut = u32_checked(len(data))
        //         item_list_lut = bytearray(4 * len(item_data_pointer))
        //         for i, pnt in enumerate(item_data_pointer):
        //             pointer_offsets.append(u32_checked(start_items_lut + i * 4))
        //             write_u32(item_list_lut, pnt, i * 4)
        //         data += item_list_lut
        //         # Padding
        //         if len(data) % 16 != 0:
        //             data += bytes(0xAA for _ in range(0, 16 - (len(data) % 16)))
        //         # Sub-header
        //         data_pointer = u32_checked(len(data))
        //         subheader = bytearray(4 * 5)
        //         pointer_offsets.append(u32_checked(data_pointer + 0x00))
        //         write_u32(subheader, start_floor_list_lut, 0x00)
        //         pointer_offsets.append(u32_checked(data_pointer + 0x04))
        //         write_u32(subheader, start_floor_layout_data, 0x04)
        //         pointer_offsets.append(u32_checked(data_pointer + 0x08))
        //         write_u32(subheader, start_items_lut, 0x08)
        //         pointer_offsets.append(u32_checked(data_pointer + 0x0C))
        //         write_u32(subheader, start_monster_lut, 0x0C)
        //         pointer_offsets.append(u32_checked(data_pointer + 0x10))
        //         write_u32(subheader, start_traps_lut, 0x10)
        //         data += subheader
        //
        //         return data, pointer_offsets, data_pointer
        todo!()
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

#[cfg(feature = "python")]
pub(crate) fn create_st_mappa_bin_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_mappa_bin";
    let m = PyModule::new(py, name)?;
    m.add_class::<MappaTrapList>()?;
    m.add_class::<MappaMonster>()?;
    m.add_class::<MappaMonsterList>()?;
    m.add_class::<MappaItemList>()?;
    m.add_class::<MappaFloorTerrainSettings>()?;
    m.add_class::<MappaFloorLayout>()?;
    m.add_class::<MappaFloor>()?;
    m.add_class::<MappaBin>()?;
    m.add_class::<MappaBinWriter>()?;

    Ok((name, m))
}
