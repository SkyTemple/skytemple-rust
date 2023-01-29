#  Copyright 2021-2021 Parakoopa and the SkyTemple Contributors
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

from typing import List, Tuple, Optional
from PIL.Image import Image

class WanImage:
    fragment_bytes_store: FragmentBytesStore
    frame_store: FrameStore
    animation_store: AnimationStore
    palette: Palette
    is_256_color: bool
    sprite_type: SpriteType
    unk2: int


class FragmentBytesStore:
    fragment_bytes: List[FragmentBytes]


class FragmentBytes:
    mixed_pixels: List[int]
    z_index: int

    def decode_fragment(self, resolution: FragmentResolution) -> List[int]: ...

    def to_image(self, palette: Palette, fragment: Fragment) -> List[int]: ...


class FrameStore:
    frames: List[Frame]
    max_fragment_alloc_count: int


class Fragment:
    unk1: int
    unk3_4: Optional[Tuple[bool, bool]]
    unk5: bool
    fragment_bytes_index: int
    offset_y: int
    offset_x: int
    flip: FragmentFlip
    is_mosaic: bool
    pal_idx: int
    resolution: FragmentResolution

class FragmentFlip:
    flip_h: bool
    flip_v: bool

class Frame:
    fragments: List[Fragment]
    frame_offset: Optional[FrameOffset]

class FrameOffset:
    head: Tuple[int, int]
    hand_left: Tuple[int, int]
    hand_right: Tuple[int, int]
    center: Tuple[int, int]

class FragmentResolution:
    x: int
    y: int


class AnimationStore:
    copied_on_previous: Optional[List[bool]]
    anim_groups: List[List[Animation]]


class Animation:
    frames: List[AnimationFrame]


class AnimationFrame:
    duration: int
    flag: int
    frame_id: int
    offset_x: int
    offset_y: int
    shadow_offset_x: int
    shadow_offset_y: int


class Palette:
    palette: List[int]


class SpriteType:
    PropsUI: SpriteType
    Chara: SpriteType
    Unknown: SpriteType
    name: str
    value: int

def encode_image_to_static_wan_file(image: Image) -> bytes: ...