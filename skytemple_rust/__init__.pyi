from range_typed_integers import u16


class TilemapEntry:
    idx: int
    flip_x: bool
    flip_y: bool
    pal_idx: int

    def __init__(self, idx: int, flip_x: bool, flip_y: bool, pal_idx: int, ignore_too_large: bool = False): ...
    def to_int(self) -> u16: ...
    @classmethod
    def from_int(cls, entry: u16) -> TilemapEntry: ...
