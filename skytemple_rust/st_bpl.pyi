#  Copyright 2021-2022 Capypara and the SkyTemple Contributors
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
from typing import List, Sequence

from range_typed_integers import *


class BplAnimationSpec:
    duration_per_frame: u16
    number_of_frames: u16

    def __init__(self, duration_per_frame: u16, number_of_frames: u16): ...


class Bpl:
    number_palettes: u16
    has_palette_animation: bool
    palettes: Sequence[Sequence[int]]
    animation_specs: Sequence[BplAnimationSpec]
    animation_palette: Sequence[Sequence[int]]

    def __init__(self, data: bytes) -> None: ...
    def import_palettes(self, palettes: List[List[int]]) -> None: ...
    def apply_palette_animations(self, frame: int) -> List[List[int]]: ...
    def is_palette_affected_by_animation(self, pal_idx: int) -> bool: ...
    def get_real_palettes(self) -> List[List[int]]: ...
    def set_palettes(self, palettes: List[List[int]]) -> None: ...


class BplWriter:
    def __new__(cls): ...
    def write(self, model: Bpl) -> bytes: ...
