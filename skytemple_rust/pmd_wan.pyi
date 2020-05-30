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


class ImageStore:
    images: List[Image]


class Image:
    img: List[int]
    width: int
    height: int
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
    resolution: Optional[Resolution]


class MetaFrameGroup:
    meta_frames_id: List[int]


class Resolution:
    x: int
    y: int


class AnimStore:
    animations: List[Animation]
    copied_on_previous: Optional[List[bool]]
    anim_groups: List[Optional[Tuple[int, int]]]


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
    palette: List[Tuple[int, int, int, int]]


class SpriteType:
    PropsUI: SpriteType
    Chara: SpriteType
    Unknown: SpriteType

    name: str
    value: int

