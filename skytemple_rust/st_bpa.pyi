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
from typing import List, Sequence, Optional

from PIL import Image
from range_typed_integers import *


class BpaFrameInfo:
    duration_per_frame: u16
    unk2: u16
    def __init__(self, duration_per_frame: int, unk2: int): ...


class Bpa:
    number_of_tiles: u16
    number_of_frames: u16
    tiles: Sequence[bytes]
    frame_info: Sequence[BpaFrameInfo]

    def __init__(self, data: bytes): ...
    @classmethod
    def new_empty(cls) -> 'Bpa': ...
    def get_tile(self, tile_idx: int, frame_idx: int) -> bytes: ...
    def tiles_to_pil_separate(self, palette: Sequence[int], width_in_tiles: int = 20) -> List[Image.Image]: ...
    def tiles_to_pil(self, palette: Sequence[int]) -> Optional[Image.Image]: ...
    def pil_to_tiles(self, image: Image.Image) -> None: ...
    def pil_to_tiles_separate(self, images: List[Image.Image]) -> None: ...
    def tiles_for_frame(self, frame: int) -> Sequence[bytes]: ...


class BpaWriter:
    def __new__(cls): ...
    def write(self, model: Bpa) -> bytes: ...
