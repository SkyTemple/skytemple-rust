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
from typing import List, Optional, Protocol, Union, Sequence

from skytemple_rust.st_bma import Bma
from skytemple_rust.st_bpa import Bpa
from skytemple_rust.st_bpc import Bpc
from skytemple_rust.st_bpl import Bpl


class RomFileProviderProtocol(Protocol):
    def getFileByName(self, filename: str) -> bytes: ...


class BgListEntry:
    bpl_name: str
    bpc_name: str
    bma_name: str
    bpa_names: Sequence[Optional[str]]
    def __init__(self, bpl_name: str, bpc_name: str, bma_name: str, bpa_names: List[Optional[str]]): ...
    def get_bpl(self, rom_or_directory_root: Union[str, RomFileProviderProtocol]) -> Bpl: ...
    def get_bpc(self, rom_or_directory_root: Union[str, RomFileProviderProtocol], bpc_tiling_width: int = 3, bpc_tiling_height: int = 3) -> Bpc: ...
    def get_bma(self, rom_or_directory_root: Union[str, RomFileProviderProtocol]) -> Bma: ...
    def get_bpas(self, rom_or_directory_root: Union[str, RomFileProviderProtocol]) -> List[Optional[Bpa]]: ...


class BgList(BgListEntry):
    level: Sequence[BgListEntry]

    def __init__(self, data: bytes): ...
    def find_bma(self, name: str) -> int: ...
    def find_bpl(self, name: str) -> int: ...
    def find_bpc(self, name: str) -> int: ...
    def find_bpa(self, name: str) -> int: ...
    def add_level(self, level: BgListEntry): ...
    def set_level(self, level_id: int, level: BgListEntry): ...
    def set_level_bpa(self, level_id: int, bpa_id: int, bpa_name: Optional[str]): ...

class BgListWriter:
    def __new__(cls): ...
    def write(self, model: BgList) -> bytes: ...
