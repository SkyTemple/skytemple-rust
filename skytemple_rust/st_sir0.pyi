from __future__ import annotations

from typing import runtime_checkable, Protocol, Tuple, List, Optional, Any

from range_typed_integers import u32


@runtime_checkable
# See skytemple_files.container.sir0.sir0_serializable
class Sir0Serializable(Protocol):
    def sir0_serialize_parts(self) -> Tuple[bytes, List[u32], Optional[u32]]:
        ...

    @classmethod
    def sir0_unwrap(cls, content_data: bytes, data_pointer: u32) -> 'Sir0Serializable':
        ...


class Sir0:
    data_pointer: u32
    content: bytes
    content_pointer_offsets: List[u32]

    def __init__(
            self, content: bytes, pointer_offsets: List[u32], data_pointer: Optional[int] = None
    ):
        ...

    @classmethod
    def from_bin(cls, data: bytes) -> Sir0:
        ...


class Sir0Writer:
    def __new__(cls): ...
    def write(self, model: Sir0) -> bytes: ...

