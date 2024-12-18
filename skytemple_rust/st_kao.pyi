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
from PIL.Image import Image
from typing import Optional, Tuple, Iterator

class KaoPropertiesState:
    kao_image_limit: int

    @classmethod
    def instance(cls) -> "KaoPropertiesState": ...


class KaoImage:
    @classmethod
    def create_from_raw(cls, cimg: bytes, pal: bytes) -> "KaoImage": ...
    def get(self) -> Image: ...
    def clone(self) -> KaoImage: ...
    def size(self) -> int: ...
    def set(self, pil: Image) -> "KaoImage": ...
    def raw(self) -> Tuple[bytes, bytes]: ...

class Kao:
    def __init__(self, data: bytes): ...
    @classmethod
    def create_new(cls, number_entries: int): ...
    def expand(self, new_size: int): ...
    def n_entries(self) -> int: ...
    def get(self, index: int, subindex: int) -> Optional[KaoImage]: ...
    def set(self, index: int, subindex: int, img: KaoImage): ...
    def set_from_img(self, index: int, subindex: int, pil: Image): ...
    def delete(self, index: int, subindex: int): ...
    def __iter__(self) -> Iterator[Tuple[int, int, Optional[KaoImage]]]: ...

class KaoWriter:
    def __new__(cls): ...
    def write(self, model: Kao) -> bytes: ...
