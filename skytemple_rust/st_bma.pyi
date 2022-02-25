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
from typing import List, Optional, Sequence

from PIL import Image

from skytemple_rust.st_bpa import Bpa
from skytemple_rust.st_bpc import Bpc
from skytemple_rust.st_bpl import Bpl


class Bma:
    map_width_camera: int
    map_height_camera: int
    tiling_width: int
    tiling_height: int
    map_width_chunks: int
    map_height_chunks: int
    number_of_layers: int
    unk6: int
    number_of_collision_layers: int

    layer0: Sequence[int]
    layer1: Optional[Sequence[int]]

    # if unk6:
    unknown_data_block: Optional[Sequence[int]]
    # if number_of_collision_layers > 0:
    collision: Optional[Sequence[bool]]
    # if number_of_collision_layers > 1:
    collision2: Optional[Sequence[bool]]

    def __init__(self, data: bytes): ...

    def to_pil_single_layer(self, bpc: Bpc, palettes: List[List[int]], bpas: Sequence[Optional[Bpa]], layer: int) -> Image.Image: ...
    def to_pil(
            self, bpc: Bpc, bpl: Bpl, bpas: List[Optional[Bpa]],
            include_collision: bool = True, include_unknown_data_block: bool = True, pal_ani: bool = True, single_frame: bool = False
    ) -> List[Image.Image]: ...
    def from_pil(
            self, bpc: Bpc, bpl: Bpl, lower_img: Optional[Image.Image] = None, upper_img: Optional[Image.Image] = None,
            force_import: bool = False, how_many_palettes_lower_layer: int = 16
    ) -> None: ...
    def remove_upper_layer(self) -> None: ...
    def add_upper_layer(self) -> None: ...
    def resize(self, new_width_chunks: int, new_height_chunks: int, new_width_camera: int, new_height_camera: int) -> None: ...
    def place_chunk(self, layer_id: int, x: int, y: int, chunk_index: int) -> None: ...
    def place_collision(self, collision_layer_id: int, x: int, y: int, is_solid: bool) -> None: ...
    def place_data(self, x: int, y: int, data: int) -> None: ...
    def deepcopy(self) -> Bma: ...

class BmaWriter:
    def __new__(cls): ...
    def write(self, model: Bma) -> bytes: ...
