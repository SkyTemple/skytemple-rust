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
from typing import Sequence, List, Tuple, Iterator

from range_typed_integers import *

_EvolutionMethod = u16
_AdditionalRequirement = u16
_Gender = u8
_PokeType = u8
_MovementType = u8
_IQGroup = u8
_Ability = u8
_ShadowSize = i8


class MdPropertiesState:
    num_entities: int
    max_possible: int

    @classmethod
    def instance(cls) -> "MdPropertiesState": ...


class MdEntry:
    md_index: u32
    entid: u16
    unk31: u16
    national_pokedex_number: u16
    base_movement_speed: u16
    pre_evo_index: u16
    evo_method: _EvolutionMethod
    evo_param1: u16
    evo_param2: _AdditionalRequirement
    sprite_index: i16
    gender: _Gender
    body_size: u8
    type_primary: _PokeType
    type_secondary: _PokeType
    movement_type: _MovementType
    iq_group: _IQGroup
    ability_primary: _Ability
    ability_secondary: _Ability
    exp_yield: u16
    recruit_rate1: i16
    base_hp: u16
    recruit_rate2: i16
    base_atk: u8
    base_sp_atk: u8
    base_def: u8
    base_sp_def: u8
    weight: i16
    size: i16
    unk17: u8
    unk18: u8
    shadow_size: _ShadowSize
    chance_spawn_asleep: i8
    hp_regeneration: u8
    unk21_h: i8
    base_form_index: i16
    exclusive_item1: i16
    exclusive_item2: i16
    exclusive_item3: i16
    exclusive_item4: i16
    unk27: i16
    unk28: i16
    unk29: i16
    unk30: i16
    bitfield1_0: bool
    bitfield1_1: bool
    bitfield1_2: bool
    bitfield1_3: bool
    can_move: bool
    bitfield1_5: bool
    can_evolve: bool
    item_required_for_spawning: bool

    @classmethod
    def new_empty(cls, entid: u16) -> "MdEntry": ...

    @property
    def md_index_base(self) -> int: ...


class Md:
    entries: Sequence[MdEntry]

    def __init__(self, data: bytes): ...

    def get_by_index(self, index: int) -> MdEntry: ...

    def get_by_entity_id(self, index: int) -> List[Tuple[int, MdEntry]]: ...

    def __len__(self) -> int: ...

    def __getitem__(self, key: int) -> MdEntry: ...

    def __setitem__(self, key: int, value: MdEntry) -> None: ...

    def __delitem__(self, key: int) -> None: ...

    def __iter__(self) -> Iterator[MdEntry]: ...

class MdWriter:
    def __new__(cls): ...
    def write(self, model: Md) -> bytes: ...
