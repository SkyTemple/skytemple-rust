# __all__ contains all submodules.
from typing import runtime_checkable, Protocol, Tuple, List, Optional, Any


class TilemapEntry:
    idx: int
    flip_x: bool
    flip_y: bool
    pal_idx: int

    def __init__(self, idx: int, flip_x: bool, flip_y: bool, pal_idx: int, ignore_too_large: bool = False): ...
    def to_int(self) -> int: ...
    @classmethod
    def from_int(cls, entry: int) -> TilemapEntry: ...


@runtime_checkable
class Sir0Serializable(Protocol):
    def sir0_serialize_parts(self) -> Tuple[bytes, List[int], Optional[int]]:
        """
        Prepares this object to be wrapped in Sir0.
        Returns:
        - The binary content data for this type
        - A list of pointers in the binary content (offsets)
        - Optionally a pointer to the start of the data, if None, the beginning of the data is used.
        """
        ...

    @classmethod
    def sir0_unwrap(cls, content_data: bytes, data_pointer: int, static_data: Optional[Any] = None) -> 'Sir0Serializable':
        """
        Builds the model from the unwrapped Sir0.
        static_data is not used by skytemple_rust (see the protocol in skytemple_files for more info).
        """
        ...
