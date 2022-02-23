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
from typing import Sequence
from PIL import Image

from skytemple_rust.st_dpc import Dpc
from skytemple_rust.st_dpci import Dpci

_DmaType = int
    # WALL = 0
    # WATER = 1
    # FLOOR = 2


_DmaExtraType = int
    # FLOOR1 = 0
    # WALL_OR_VOID = 1
    # FLOOR2 = 2


class Dma:
    chunk_mappings: Sequence[int]

    def __init__(self, data: bytes): ...

    def get(self, get_type: _DmaType, neighbors_same: int) -> Sequence[int]: ...

    def get_extra(self, extra_type: _DmaExtraType) -> Sequence[int]: ...

    def set(self, get_type: _DmaType, neighbors_same: int, variation_index: int, value: int): ...

    def set_extra(self, extra_type: _DmaExtraType, index: int, value: int): ...

    def to_pil(
            self, dpc: Dpc, dpci: Dpci, palettes: Sequence[Sequence[int]]
    ) -> Image.Image: ...


class DmaWriter:
    def __new__(cls): ...
    def write(self, model: Dma) -> bytes: ...
