# __all__ contains all submodules.

class TilemapEntry:
    idx: int
    flip_x: bool
    flip_y: bool
    pal_idx: int

    def __init__(self, idx: int, flip_x: bool, flip_y: bool, pal_idx: int, ignore_too_large: bool = False): ...
    def to_int(self) -> int: ...
    @classmethod
    def from_int(cls, entry: int) -> TilemapEntry: ...
