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
use crate::dse::st_swdl::prgi::{SwdlPrgi, PRGI_HEADER};
use crate::dse::st_swdl::wavi::{SwdlPcmdReference, SwdlWavi};
use crate::gettext::gettext;
use crate::python::PyResult;
use bytes::{Buf, BufMut, Bytes, BytesMut};

const SWDL_HEADER: &[u8] = b"swdl";

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwdlPcmdLen {
    pub reference: u32,
    pub external: bool,
}

impl SwdlPcmdLen {
    pub fn new(reference: u32, external: bool) -> Self {
        SwdlPcmdLen {
            reference,
            external,
        }
    }
}

impl From<&mut StBytes> for PyResult<SwdlPcmdLen> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 4,
            gettext("SWDL file too short (SwdlPcmdLen EOF).")
        );
        let data_i = source.get_u32_le();
        let (reference, external) = if data_i >> 0x10 == 0xAAAA {
            (data_i & 0x10, true)
        } else {
            (data_i, false)
        };
        Ok(SwdlPcmdLen::new(reference, external))
    }
}

impl From<SwdlPcmdLen> for StBytes {
    fn from(source: SwdlPcmdLen) -> Self {
        let mut b = BytesMut::with_capacity(4);
        if source.external {
            b.put_u32_le((source.reference & 0x10) + (0xAAAA << 0x10))
        } else {
            b.put_u32_le(source.reference)
        }
        debug_assert_eq!(
            source,
            <PyResult<SwdlPcmdLen>>::from(&mut StBytes::from(b.clone())).unwrap()
        );
        b.into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwdlHeader {
    pub version: u16,
    pub unk1: u8,
    pub unk2: u8,
    pub modified_date: DseDate,
    pub file_name: DseFilename,
    pub unk13: u32,
    pub pcmdlen: SwdlPcmdLen,
    pub unk17: u16,
    number_wavi_slots: u16,
    number_prgi_slots: u16,
    len_wavi: u32,
}

impl SwdlHeader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u16,
        unk1: u8,
        unk2: u8,
        modified_date: DseDate,
        file_name: DseFilename,
        unk13: u32,
        pcmdlen: SwdlPcmdLen,
        unk17: u16,
    ) -> Self {
        SwdlHeader {
            version,
            unk1,
            unk2,
            modified_date,
            file_name,
            unk13,
            pcmdlen,
            unk17,
            number_wavi_slots: 0,
            number_prgi_slots: 0,
            len_wavi: 0,
        }
    }

    fn get_initial_number_wavi_slots(&self) -> u16 {
        self.number_wavi_slots
    }

    fn get_initial_number_prgi_slots(&self) -> u16 {
        self.number_prgi_slots
    }

    fn get_initial_wavi_len(&self) -> u32 {
        self.len_wavi
    }
}

impl From<&mut StBytes> for PyResult<SwdlHeader> {
    fn from(source: &mut StBytes) -> Self {
        pyr_assert!(
            source.len() >= 80,
            gettext("SWDL file too short (Header EOF).")
        );
        let header = source.copy_to_bytes(4);
        pyr_assert!(
            SWDL_HEADER == header,
            gettext("Invalid SWDL/Header header.")
        );
        // 4 zero bytes;
        source.advance(4);
        // We don't validate the length (next 4 bytes):
        source.advance(4);
        let version = source.get_u16_le();
        // HEADER2 VALUE OF SMDL???
        let unk1 = source.get_u8();
        // HEADER1 VALUE OF SMDL???
        let unk2 = source.get_u8();
        // 8 zero bytes;
        source.advance(8);
        let modified_date: DseDate = source.into();
        let file_name = DseFilename::from_bytes_fixed(source, 16);
        // 4 padding bytes;
        source.advance(4);
        // 8 zero bytes;
        source.advance(8);
        let unk13 = source.get_u32_le();
        let pcmdlen_err: PyResult<SwdlPcmdLen> = <&mut StBytes>::into(source);
        let pcmdlen = pcmdlen_err?;
        // 2 zero bytes;
        source.advance(2);
        let number_wavi_slots = source.get_u16_le();
        let number_prgi_slots = source.get_u16_le();
        let unk17 = source.get_u16_le();
        let len_wavi = source.get_u32_le();
        Ok(SwdlHeader {
            version,
            unk1,
            unk2,
            modified_date,
            file_name,
            unk13,
            pcmdlen,
            unk17,
            number_wavi_slots,
            number_prgi_slots,
            len_wavi,
        })
    }
}

impl SwdlHeader {
    fn to_bytes(
        &self,
        data_len: u32,
        pcmdlen: SwdlPcmdLen,
        wavisitlen: u16,
        prgi_slots: u16,
        wavi_len: u32,
    ) -> StBytes {
        let mut b = BytesMut::with_capacity(80);
        b.put_slice(SWDL_HEADER);
        b.put_u32_le(0);
        b.put_u32_le(data_len);
        b.put_u16_le(self.version);
        b.put_u8(self.unk1);
        b.put_u8(self.unk2);
        b.put_u64(0);
        b.put(StBytes::from(self.modified_date.clone()).0);
        b.put(StBytes::from(self.file_name.clone()).0);
        b.put(&b"\x00\xaa\xaa\xaa"[..]);
        b.put_u64(0);
        b.put_u32_le(self.unk13);
        b.put(StBytes::from(pcmdlen).0);
        b.put_u16(0);
        b.put_u16_le(wavisitlen);
        b.put_u16_le(prgi_slots);
        b.put_u16_le(self.unk17);
        b.put_u32_le(wavi_len);
        debug_assert_eq!(80, b.len());
        b.into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Swdl {
    pub header: SwdlHeader,
    pub wavi: SwdlWavi,
    pub pcmd: Option<SwdlPcmd>,
    pub prgi: Option<SwdlPrgi>,
    pub kgrp: Option<SwdlKgrp>,
}

impl Swdl {
    /// Returns the inner name of a SWDL file (stored in the header), without
    /// the overhead of reading in the entire file.
    /// This won't do any checks, so if an invalid / non-SWDL file is passed in,
    /// this will likely panic.
    pub fn name_for<T: AsRef<[u8]> + Buf>(raw: T) -> DseFilename {
        DseFilename::from(&mut (&raw.as_ref()[0x20..0x30]))
    }
}

impl From<StBytes> for PyResult<Swdl> {
    fn from(mut source: StBytes) -> Self {
        let header = <PyResult<SwdlHeader>>::from(&mut source)?;
        let len_wavi = header.get_initial_wavi_len() + 0x10; // (0x10 = Header size) TODO: Is this correct???
        let number_wavi_slots = header.get_initial_number_wavi_slots();
        let number_prgi_slots = header.get_initial_number_prgi_slots();

        let wavi = SwdlWavi::from_bytes(&mut source, number_wavi_slots)?;
        pyr_assert!(
            len_wavi as usize == wavi.get_initial_length(),
            gettext("Swdl read error (inconsistent Wavi length)")
        );

        let prgi: Option<SwdlPrgi>;
        let kgrp: Option<SwdlKgrp>;

        if &source.clone().copy_to_bytes(4)[..] == PRGI_HEADER {
            // Has PRGI & KGRP
            prgi = Some(SwdlPrgi::from_bytes(&mut source, number_prgi_slots)?);
            kgrp = Some(<PyResult<SwdlKgrp>>::from(&mut source)?);
        } else {
            prgi = None;
            kgrp = None;
        }

        let pcmd = if !header.pcmdlen.external && header.pcmdlen.reference > 0 {
            Some(<PyResult<SwdlPcmd>>::from(&mut source)?)
        } else {
            None
        };

        let mut slf = Swdl {
            header,
            wavi,
            pcmd,
            prgi,
            kgrp,
        };

        for sample in slf.wavi.sample_info_table.iter_mut().flatten() {
            if let Some(pcmd) = &slf.pcmd {
                pyr_assert!(
                    sample.get_initial_sample_pos() + sample.get_sample_length()
                        <= pcmd.chunk_data.len() as u32,
                    gettext("Swdl read: Invalid sample data")
                );
            }
            sample.sample = Some(SwdlPcmdReference::new(
                sample.get_initial_sample_pos(),
                sample.get_sample_length(),
            ));
        }

        Ok(slf)
    }
}

impl From<Swdl> for StBytes {
    fn from(source: Swdl) -> Self {
        let (pcmdlen, pcmd) = if let Some(pcmd) = source.pcmd {
            (
                SwdlPcmdLen::new(pcmd.chunk_data.len() as u32, false),
                StBytes::from(pcmd).0,
            )
        } else {
            (
                SwdlPcmdLen::new(source.header.pcmdlen.reference, true),
                Bytes::new(),
            )
        };

        // The file might have PRGI slots set, even if none are defined
        let mut prgi_slots = source.header.get_initial_number_prgi_slots();
        if let Some(ref slots) = source.prgi {
            prgi_slots = slots.program_table.len() as u16;
        }

        let wavisitlen = source.wavi.sample_info_table.len();
        let wavi = StBytes::from(source.wavi).0;
        let wavi_len = wavi.len() - 0x10;

        let data = wavi
            .into_iter()
            .chain(if let Some(prgi) = source.prgi {
                StBytes::from(prgi).0
            } else {
                Bytes::new()
            })
            .chain(if let Some(kgrp) = source.kgrp {
                StBytes::from(kgrp).0
            } else {
                Bytes::new()
            })
            .chain(pcmd)
            .chain(
                b"eod \x00\x00\x15\x04\x10\x00\x00\x00\x00\x00\x00\x00"
                    .iter()
                    .copied(),
            )
            .collect::<Bytes>();

        source
            .header
            .to_bytes(
                80 + data.len() as u32,
                pcmdlen,
                wavisitlen as u16,
                prgi_slots,
                wavi_len as u32,
            )
            .0
            .into_iter()
            .chain(data)
            .collect()
    }
}
