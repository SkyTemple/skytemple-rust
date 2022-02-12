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
from typing import Optional, Sequence


class SwdlPcmd:
    chunk_data: bytes


class SwdlPcmdReference:
    pcmd: SwdlPcmd
    offset: int
    length: int


class SwdlSampleInfoTblEntry:
    id: int
    ftune: int
    ctune: int
    rootkey: int  # seems unused by game!
    ktps: int
    volume: int  # (0-127)
    pan: int  # (0-64-127)
    unk5: int  # probably key_group, always 0
    unk58: int
    sample_format: int  # compare against SampleFormatConsts
    unk9: int
    loops: bool
    unk10: int
    unk11: int
    unk12: int
    unk13: int
    sample_rate: int
    sample: Optional[SwdlPcmdReference]
    loop_begin_pos: int  # (For ADPCM samples, the 4 bytes preamble is counted in the loopbeg!)
    loop_length: int

    envelope: int
    envelope_multiplier: int
    unk19: int
    unk20: int
    unk21: int
    unk22: int
    attack_volume: int
    attack: int
    decay: int
    sustain: int
    hold: int
    decay2: int
    release: int
    unk57: int


class SwdlWavi:
    sample_info_table: Sequence[Optional[SwdlSampleInfoTblEntry]]


class SwdlLfoEntry:
    unk34: int
    unk52: int
    dest: int
    wshape: int
    rate: int
    unk29: int
    depth: int
    delay: int
    unk32: int
    unk33: int


class SwdlSplitEntry:
    id: int
    unk11: int
    unk25: int
    lowkey: int
    hikey: int
    lolevel: int
    hilevel: int
    unk16: int
    unk17: int
    sample_id: int
    ftune: int
    ctune: int
    rootkey: int
    ktps: int
    sample_volume: int
    sample_pan: int
    keygroup_id: int
    unk22: int
    unk23: int
    unk24: int

    envelope: int
    envelope_multiplier: int
    unk37: int
    unk38: int
    unk39: int
    unk40: int
    attack_volume: int
    attack: int
    decay: int
    sustain: int
    hold: int
    decay2: int
    release: int
    unk53: int


class SwdlProgramTable:
    id: int
    prg_volume: int
    prg_pan: int
    unk3: int
    that_f_byte: int
    unk4: int
    unk5: int
    unk7: int
    unk8: int
    unk9: int
    lfos: Sequence[SwdlLfoEntry]
    splits: Sequence[SwdlSplitEntry]


class SwdlPrgi:
    program_table: Sequence[Optional[SwdlProgramTable]] = []


class SwdlKeygroup:
    id: int
    poly: int
    priority: int
    vclow: int
    vchigh: int
    unk50: int
    unk51: int


class SwdlKgrp:
    keygroups: Sequence[SwdlKeygroup]


class SwdlPcmdLen:
    reference: Optional[int]
    external: bool


class SwdlHeader:
    version: int
    unk1: int
    unk2: int
    modified_date: bytes
    file_name: bytes
    unk13: int
    pcmdlen: SwdlPcmdLen
    unk17: int


class Swdl:
    header: SwdlHeader
    wavi: SwdlWavi
    pcmd: Optional[SwdlPcmd]
    prgi: Optional[SwdlPrgi]
    kgrp: Optional[SwdlKgrp]

    def __init__(self, data: bytes): ...


class SwdlWriter:
    def __new__(cls): ...
    def write(self, model: Swdl) -> bytes: ...
