from typing import Protocol, Dict, Optional, Sequence, List, TypeVar, Tuple, Any
from enum import Enum

exps_available = False
try:
    from explorerscript.source_map import SourceMap
    exps_available = True
except ImportError:
    pass


class HasIdAndNameProtocol(Protocol):
    id: int
    name: str


class ScriptDirectionProtocol(Protocol):
    ssb_id: int
    name: str


class ScriptOpCodeArgumentProtocol(Protocol):
    id: int
    type: str
    name: str


class ScriptOpCodeRepeatingArgumentGroupProtocol(Protocol):
    id: int
    arguments: Sequence[ScriptOpCodeArgumentProtocol]


class ScriptOpCodeProtocol(HasIdAndNameProtocol, Protocol):
    params: int
    stringidx: int
    unk2: int
    unk3: int
    arguments: Sequence[ScriptOpCodeArgumentProtocol]
    repeating_argument_group: Optional[ScriptOpCodeRepeatingArgumentGroupProtocol]


class ScriptDataProtocol(Protocol):
    @property
    def game_variables__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def game_variables__by_name(self) -> Dict[str, HasIdAndNameProtocol]: ...
    @property
    def objects__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def face_names__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def face_position_modes__by_id(self) -> List[HasIdAndNameProtocol]: ...
    @property
    def directions__by_ssb_id(self) -> Dict[int, ScriptDirectionProtocol]: ...
    @property
    def common_routine_info__by_id(self) -> List[HasIdAndNameProtocol]: ...
    @property
    def menus__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def process_specials__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def sprite_effects__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def bgms__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def level_list__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def level_entities__by_id(self) -> Dict[int, HasIdAndNameProtocol]: ...
    @property
    def op_codes__by_id(self) -> Dict[int, ScriptOpCodeProtocol]: ...


class SourceMapV2Protocol(Protocol):
    # todo
    pass


class SsbRoutineType(Enum):
    GENERIC = 1
    ACTOR = 3
    OBJECT = 4
    PERFORMER = 5
    COROUTINE = 9
    INVALID = -1


class SsbOperation(Protocol):
    offset: int
    op_code: HasIdAndNameProtocol
    params: List[Any]


class RoutineInfo(Protocol):
    type: SsbRoutineType
    linked_to: int
    linked_to_name: Optional[str] = None

    @property
    def linked_to_repr(self) -> Optional[str]: ...


class Ssb:
    original_binary_data: bytes
    routine_info: List[Tuple[int, RoutineInfo]]
    routine_ops: List[List[SsbOperation]]
    constants: List[str]
    strings: Dict[str, List[str]]

    @classmethod
    def create_empty(cls, scriptdata: ScriptDataProtocol, game_region: str) -> 'Ssb': ...
    def __init__(self, data: bytes, scriptdata: ScriptDataProtocol, game_region: str, string_codec: str): ...
    def to_explorerscript_v2(self) -> Tuple[str, SourceMapV2Protocol]: ...

    # NOTE: These two only work if exploerscript is installed. SsbOperation and RoutineInfo will be converted
    # to the appropriate explorerscript library types.
    if exps_available:
        def to_explorerscript(self) -> Tuple[str, 'SourceMap']: ...
        def to_ssb_script(self) -> Tuple[str, 'SourceMap']: ...


class SsbWriter:
    def __init__(self, script_data: ScriptDataProtocol, game_region: str, string_codec: str): ...
    def write(self, _model: Ssb) -> bytes: ...
