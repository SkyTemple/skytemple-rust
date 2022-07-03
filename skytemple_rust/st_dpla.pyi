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
from typing import Sequence, List

from skytemple_rust.st_sir0 import Sir0Serializable


class Dpla(Sir0Serializable):
    colors: Sequence[Sequence[int]]
    durations_per_frame_for_colors: Sequence[int]

    def __init__(self, data: bytes, pointer_to_pointers: int): ...

    def get_palette_for_frame(self, pal_idx: int, frame_id: int) -> List[int]: ...

    def has_for_palette(self, palette_idx: int) -> bool: ...

    def get_frame_count_for_palette(self, palette_idx: int) -> int: ...

    def enable_for_palette(self, palid: int) -> None: ...

    def disable_for_palette(self, palid: int) -> None: ...

    def get_duration_for_palette(self, palette_idx: int) -> int: ...

    def set_duration_for_palette(self, palid: int, duration: int) -> None: ...

    def apply_palette_animations(self, palettes: Sequence[Sequence[int]], frame_idx: int) -> List[List[int]]: ...
