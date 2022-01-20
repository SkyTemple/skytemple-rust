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
class Atupx:
    def __init__(self, data: bytes): ...
    def decompress(self) -> bytes: ...
    def to_bytes(self) -> bytes: ...
    @classmethod
    def cont_size(cls, data: bytes, byte_offset=0): ...
    @classmethod
    def compress(cls, data: bytes) -> 'Atupx': ...
