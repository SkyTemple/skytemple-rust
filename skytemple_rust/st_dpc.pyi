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
from typing import Sequence, Tuple

from PIL import Image

from skytemple_rust import TilemapEntry
from skytemple_rust.st_dpci import Dpci


class Dpc:
    chunks: Sequence[Sequence[TilemapEntry]]

    def __init__(self, data: bytes): ...

    def chunks_to_pil(self, dpci: Dpci, palettes: Sequence[Sequence[int]], width_in_mtiles=16) -> Image.Image: ...

    def single_chunk_to_pil(self, chunk_idx, dpci: Dpci, palettes: Sequence[Sequence[int]]) -> Image.Image: ...

    def pil_to_chunks(self, image: Image.Image, force_import=True) -> Tuple[Sequence[bytes], Sequence[Sequence[int]]]: ...

    def import_tile_mappings(
            self, tile_mappings: Sequence[Sequence[TilemapEntry]],
            contains_null_chunk=False, correct_tile_ids=True
    ): ...


class DpcWriter:
    def __new__(cls): ...
    def write(self, model: Dpc) -> bytes: ...
