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


class Dpci:
    tiles: Sequence[bytes]

    def __init__(self, data: bytes): ...

    def tiles_to_pil(self, palettes: Sequence[Sequence[int]], width_in_tiles=20, palette_index=0) -> Image.Image: ...

    def pil_to_tiles(self, image: Image.Image): ...

    def import_tiles(self, tiles: Sequence[bytearray], contains_null_tile=False): ...


class DpciWriter:
    def __new__(cls): ...
    def write(self, model: Dpci) -> bytes: ...
