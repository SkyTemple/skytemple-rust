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

from skytemple_rust import TilemapEntry
from skytemple_rust.st_bpa import Bpa
from range_typed_integers import *


class BpcLayer:
    # The actual number of tiles is one lower
    number_tiles: u16
    # There must be 4 BPAs. (0 for not used)
    bpas: Sequence[u16]
    # NOTE: Incosistent with number_tiles. We are including the null chunk in this count.
    chunk_tilemap_len: u16
    # May also be set from outside after creation:
    tiles:  Sequence[bytes]
    tilemap: Sequence[TilemapEntry]

    def __init__(self, number_tiles: int, bpas: List[int], chunk_tilemap_len: int, tiles: List[bytes], tilemap: List[TilemapEntry]) -> None: ...


class Bpc:
    tiling_width: int
    tiling_height: int
    number_of_layers: int
    layers: Sequence[BpcLayer]

    def __init__(self, data: bytes, tiling_width: int, tiling_height: int): ...
    def chunks_to_pil(self, layer: int, palettes: Sequence[Sequence[int]], width_in_mtiles: int = 20) -> Image.Image: ...
    def single_chunk_to_pil(self, layer: int, chunk_idx: int, palettes: Sequence[Sequence[int]]) -> Image.Image: ...
    def tiles_to_pil(self, layer: int, palettes: Sequence[Sequence[int]], width_in_tiles: int = 20, single_palette: Optional[int] = None) -> Image.Image: ...

    def chunks_animated_to_pil(
            self, layer: int, palettes: Sequence[Sequence[int]], bpas: Sequence[Optional[Bpa]], width_in_mtiles: int = 20
    ) -> List[Image.Image]: ...

    def single_chunk_animated_to_pil(
            self, layer: int, chunk_idx: int, palettes: Sequence[Sequence[int]], bpas: Sequence[Optional[Bpa]]
    ) -> List[Image.Image]: ...

    def pil_to_tiles(self, layer: int, image: Image.Image) -> None: ...
    def pil_to_chunks(self, layer: int, image: Image.Image, force_import: bool = True) -> List[List[int]]: ...
    def get_tile(self, layer: int, index: int) -> TilemapEntry: ...
    def set_tile(self, layer: int, index: int, tile_mapping) -> None: ...  # : List[TilemapEntry]
    def get_chunk(self, layer: int, index: int) -> Sequence[TilemapEntry]: ...
    def import_tiles(self, layer: int, tiles: List[bytes], contains_null_tile: bool = False) -> None: ...

    def import_tile_mappings(
            self, layer: int, tile_mappings,  # : List[TilemapEntry]
            contains_null_chunk: bool = False, correct_tile_ids: bool = True
    ) -> None: ...

    def get_bpas_for_layer(self, layer: int, bpas_from_bg_list: Sequence[Optional[Bpa]]) -> List[Bpa]: ...
    def set_chunk(self, layer: int, index: int, new_tilemappings) -> None: ...  # : List[TilemapEntry]
    def remove_upper_layer(self) -> None: ...
    def add_upper_layer(self) -> None: ...
    def process_bpa_change(self, bpa_index: int, tiles_bpa_new: u16) -> None: ...


class BpcWriter:
    def __new__(cls): ...
    def write(self, model: Bpc) -> bytes: ...
