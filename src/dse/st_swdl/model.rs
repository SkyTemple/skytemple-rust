/*
 * Copyright 2021-2022 Capypara and the SkyTemple Contributors
 *
 * This file is part of SkyTemple.
 *
 * SkyTemple is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SkyTemple is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with SkyTemple.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::bytes::StBytes;
use crate::dse::date::DseDate;
use crate::dse::filename::DseFilename;
use crate::dse::st_swdl::kgrp::SwdlKgrp;
use crate::dse::st_swdl::pcmd::SwdlPcmd;
use crate::dse::st_swdl::prgi::SwdlPrgi;
use crate::dse::st_swdl::wavi::SwdlWavi;

#[derive(Clone)]
pub struct SwdlPcmdLen {
    pub reference: Option<u32>,
    pub external: bool,
}

#[derive(Clone)]
pub struct SwdlHeader {
    pub version: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub modified_date: DseDate,
    pub file_name: DseFilename,
    pub unk13: u32,
    pub pcmdlen: u32,
    pub unk17: u32,
}

#[derive(Clone)]
pub struct Swdl {
    pub header: SwdlHeader,
    pub wavi: SwdlWavi,
    pub pcmd: Option<SwdlPcmd>,
    pub prgi: Option<SwdlPrgi>,
    pub kgrp: Option<SwdlKgrp>,
}

impl From<StBytes> for Swdl {
    fn from(source: StBytes) -> Self {
        todo!()
    }
}

impl From<Swdl> for StBytes {
    fn from(source: Swdl) -> Self {
        todo!()
    }
}
