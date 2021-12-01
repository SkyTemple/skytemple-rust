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


class WanImage:
    image_store: ImageStore
    meta_frame_store: MetaFrameStore
    anim_store: AnimStore
    palette: Palette
    raw_particule_table: List[int]
    is_256_color: bool
    sprite_type: SpriteType
    unk_1: int
    unk2: int


class ImageStore:
    images: List[ImageBytes]


class ImageBytes:
    mixed_pixels: List[int]
    z_index: int


class MetaFrameStore:
    meta_frames: List[MetaFrame]
    meta_frame_groups: List[MetaFrameGroup]


class MetaFrame:
    unk1: int
    unk2: int
    unk3: bool
    image_index: int
    offset_y: int
    offset_x: int
    is_last: bool
    v_flip: bool
    h_flip: bool
    is_mosaic: bool
    pal_idx: int
    resolution: Resolution


class MetaFrameGroup:
    meta_frames_id: List[int]


class Resolution:
    x: int
    y: int


class AnimStore:
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

