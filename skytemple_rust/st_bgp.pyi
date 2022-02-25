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

from PIL import Image

from skytemple_rust import TilemapEntry


class Bgp:
    palettes: Sequence[Sequence[int]] = []
    tiles: Sequence[bytearray] = []
    tilemap: Sequence[TilemapEntry] = []

    def __init__(self, data: bytes): ...

    def to_pil(self, ignore_flip_bits=False) -> Image.Image: ...

    def from_pil(self, pil: Image.Image, force_import=False) -> None: ...


class BgpWriter:
    def __new__(cls): ...
    def write(self, model: Bgp) -> bytes: ...
