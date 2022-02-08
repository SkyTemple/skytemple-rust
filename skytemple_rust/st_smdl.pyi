#  Copyright 2020-2022 Capypara and the SkyTemple Contributors
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
from typing import Sequence, Union, Optional


class SmdlHeader:
    version: int
    unk1: int
    unk2: int
    modified_date: bytes
    file_name: bytes
    unk5: int
    unk6: int
    unk8: int
    unk9: int


class SmdlSong:
    unk1: int
    unk2: int
    unk3: int
    unk4: int
    tpqn: int
    unk5: int
    nbchans: int
    unk6: int
    unk7: int
    unk8: int
    unk9: int
    unk10: int
    unk11: int
    unk12: int


class SmdlEoc:
    param1: int
    param2: int


class SmdlTrackHeader:
    param1: int
    param2: int


class SmdlTrackPreamble:
    track_id: int
    channel_id: int
    unk1: int
    unk2: int


class SmdlEventPlayNote:
    velocity: int
    octave_mod: int
    note: int
    key_down_duration: Optional[int]


class SmdlEventPause:
    value: int


class SmdlEventSpecial:
    op: int
    params: Sequence[int]


class SmdlTrack:
    header: SmdlTrackHeader
    preamble: SmdlTrackPreamble
    events: Sequence[Union[SmdlEventSpecial, SmdlEventPause, SmdlEventPlayNote]]


class Smdl:
    header: SmdlHeader
    song: SmdlSong
    tracks: Sequence[SmdlTrack]
    eoc: SmdlEoc

    def __init__(self, data: bytes): ...


class SmdlWriter:
    def __new__(cls): ...
    def write(self, model: Smdl) -> bytes: ...
