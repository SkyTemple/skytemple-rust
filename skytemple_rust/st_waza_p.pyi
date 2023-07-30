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
from typing import MutableSequence, Sequence

from range_typed_integers import *

from skytemple_rust.st_sir0 import Sir0Serializable


class LevelUpMove:
    move_id: u16
    level_id: u16

    def __init__(self, move_id: u16, level_id: u16):
        ...

    def __eq__(self, other: object) -> bool:
        ...


class MoveLearnset:
    level_up_moves: MutableSequence[LevelUpMove]
    tm_hm_moves: MutableSequence[u32]
    egg_moves: MutableSequence[u32]

    def __init__(self, level_up_moves: Sequence[LevelUpMove], tm_hm_moves: Sequence[u32], egg_moves: Sequence[u32]):
        ...

    def __eq__(self, other: object) -> bool:
        ...


class WazaMoveRangeSettings:
    target: int
    range: int
    condition: int
    unused: int

    def __init__(self, data: bytes):
        ...

    def __int__(self):
        ...

    def __eq__(self, other: object) -> bool:
        ...

class WazaMove:
    base_power: u16
    type: u8
    category: u8
    settings_range: WazaMoveRangeSettings
    settings_range_ai: WazaMoveRangeSettings
    base_pp: u8
    ai_weight: u8
    miss_accuracy: u8
    accuracy: u8
    ai_condition1_chance: u8
    number_chained_hits: u8
    max_upgrade_level: u8
    crit_chance: u8
    affected_by_magic_coat: bool
    is_snatchable: bool
    uses_mouth: bool
    ai_frozen_check: bool
    ignores_taunted: bool
    range_check_text: u8
    move_id: u16
    message_id: u8

    def __init__(self, data: bytes):
        ...

    def to_bytes(self) -> bytes:
        ...

    def __eq__(self, other: object) -> bool:
        ...


class WazaP(Sir0Serializable):
    moves: MutableSequence[WazaMove]
    learnsets: MutableSequence[MoveLearnset]

    def __init__(self, data: bytes, waza_content_pointer: int):
        ...

    def __eq__(self, other: object) -> bool:
        ...


class WazaPWriter:
    def __new__(cls): ...
    def write(self, model: WazaP) -> bytes: ...
