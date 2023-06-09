#  Copyright 2020-2023 Capypara and the SkyTemple Contributors
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
from typing import Sequence

from range_typed_integers import *

COUNT_GLOBAL_VARS: u32
COUNT_LOCAL_VARS: u32
DEFINITION_STRUCT_SIZE: int


class ScriptVariableDefinition:
    id: int
    type: u16
    unk1: u16
    memoffset: u16
    bitshift: u16
    nbvalues: u16
    default: i16
    name: str


class ScriptVariableTables:
    globals: Sequence[ScriptVariableDefinition]
    locals: Sequence[ScriptVariableDefinition]

    def __init__(self, arm9: bytes, global_start: u32, local_start: u32, subtract_from_name_addrs: u32):
        ...
