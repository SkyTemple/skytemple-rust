#  Copyright 2020-2022 Capypara and the SkyTemple Contributors
#
#  This file is part of SkyTemple.
#
#  SkyTemple is free software: you can redistribute it and/or modify
#  it under the terms of the GNU General Public License as published by
#  the Free Software Foundation, either version 3 of the License, or
#  (at your option) any later version.
#
#  SkyTemple is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#  GNU General Public License for more details.
#
#  You should have received a copy of the GNU General Public License
#  along with SkyTemple.  If not, see <https://www.gnu.org/licenses/>.
from typing import Union, List, Dict, Sequence, MutableSequence

from range_typed_integers import *

from skytemple_rust.st_sir0 import Sir0Serializable

_MappaFloorStructureType = u8
_MappaFloorWeather = u8
_MappaFloorDarknessLevel = u8
_MappaTrapType = u8
_MappaItemCategory = int
_MappaItem = int


class MappaTrapList:
    weights: Dict[_MappaTrapType, u16]

    def __init__(self, weights: Union[List[u16], Dict[_MappaTrapType, u16]]): ...

    def __eq__(self, other: object) -> bool: ...


class MappaMonster:
    level: u8
    main_spawn_weight: u16
    monster_house_spawn_weight: u16
    md_index: u16

    def __init__(self, level: u8, main_spawn_weight: u16, monster_house_spawn_weight: u16, md_index: u16):
        self.level = level
        self.main_spawn_weight = main_spawn_weight
        self.monster_house_spawn_weight = monster_house_spawn_weight
        self.md_index = md_index

    def __eq__(self, other: object) -> bool: ...


class MappaItemList:
    categories: Dict[_MappaItemCategory, int]
    items: Dict[_MappaItem, int]

    def __init__(
            self,
            categories: Dict[
                _MappaItemCategory, int
            ],
            items: Dict[_MappaItem, int],
    ):
        ...

    @classmethod
    def from_bytes(cls, data: bytes, pointer: int) -> MappaItemList: ...

    def to_bytes(self) -> bytes: ...

    def __eq__(self, other: object) -> bool: ...


class MappaFloorTerrainSettings:
    has_secondary_terrain: bool
    unk1: bool
    generate_imperfect_rooms: bool
    unk3: bool
    unk4: bool
    unk5: bool
    unk6: bool
    unk7: bool

    def __init__(
            self,
            has_secondary_terrain: bool,
            unk1: bool,
            generate_imperfect_rooms: bool,
            unk3: bool,
            unk4: bool,
            unk5: bool,
            unk6: bool,
            unk7: bool,
    ):
        ...

    def __eq__(self, other: object) -> bool: ...


class MappaFloorLayout:
    structure: _MappaFloorStructureType
    room_density: i8
    tileset_id: u8
    music_id: u8
    weather: _MappaFloorWeather
    floor_connectivity: u8
    initial_enemy_density: i8
    kecleon_shop_chance: u8
    monster_house_chance: u8
    unused_chance: u8
    sticky_item_chance: u8
    dead_ends: bool
    secondary_terrain: u8
    terrain_settings: MappaFloorTerrainSettings
    unk_e: bool
    item_density: u8
    trap_density: u8
    floor_number: u8
    fixed_floor_id: u8
    extra_hallway_density: u8
    buried_item_density: u8
    water_density: u8
    darkness_level: _MappaFloorDarknessLevel
    max_coin_amount: int
    kecleon_shop_item_positions: u8
    empty_monster_house_chance: u8
    unk_hidden_stairs: u8
    hidden_stairs_spawn_chance: u8
    enemy_iq: u16
    iq_booster_boost: i16

    def __init__(
            self,
            *,
            structure: _MappaFloorStructureType,
            room_density: i8,
            tileset_id: u8,
            music_id: u8,
            weather: _MappaFloorWeather,
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
            darkness_level: _MappaFloorDarknessLevel,
            max_coin_amount: int,
            kecleon_shop_item_positions: u8,
            empty_monster_house_chance: u8,
            unk_hidden_stairs: u8,
            hidden_stairs_spawn_chance: u8,
            enemy_iq: u16,
            iq_booster_boost: i16,
    ):
        ...

    def __eq__(self, other: object) -> bool: ...


class MappaFloor:
    layout: MappaFloorLayout
    monsters: MutableSequence[MappaMonster]
    traps: MappaTrapList
    floor_items: MappaItemList
    shop_items: MappaItemList
    monster_house_items: MappaItemList
    buried_items: MappaItemList
    unk_items1: MappaItemList
    unk_items2: MappaItemList

    def __init__(
            self,
            layout: MappaFloorLayout,
            monsters: List[MappaMonster],
            traps: MappaTrapList,
            floor_items: MappaItemList,
            shop_items: MappaItemList,
            monster_house_items: MappaItemList,
            buried_items: MappaItemList,
            unk_items1: MappaItemList,
            unk_items2: MappaItemList,
    ):
        ...

    def __eq__(self, other: object) -> bool: ...


class MappaBin(Sir0Serializable):
    floor_lists: Sequence[Sequence[MappaFloor]]

    def __init__(self, floor_lists: List[List[MappaFloor]]): ...

    def add_floor_list(self, floor_list: List[MappaFloor]):...

    def remove_floor_list(self, index: int):...

    def add_floor_to_floor_list(self, floor_list_index: int, floor: MappaFloor): ...

    def insert_floor_in_floor_list(self, floor_list_index: int, insert_index: int, floor: MappaFloor): ...

    def remove_floor_from_floor_list(self, floor_list_index: int, floor_index: int):  ...

    def __eq__(self, other: object) -> bool: ...

class MappaBinWriter:
    def __new__(cls): ...
    def write(self, model: MappaBin) -> bytes: ...
