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
use crate::bytes::StBytesMut;
use crate::compression::generic::nrl::{
    compression_step, decompression_step, NrlCompRead, NrlDecompWrite,
};
use crate::python::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Cursor;

// Operations are encoded in command bytes (CMD):
// PHASE 2
const CMD_2_SEEK_OFFSET: u8 = 0x80; // All values below
const CMD_2_FILL_LOW: u8 = 0x80; // All values equal/above until next
const CMD_2_COPY_LOW: u8 = 0xC0; // All values equal/above

/////////////////////////////////////////
/////////////////////////////////////////

#[derive(Clone)]
struct CompRead(Cursor<Bytes>);

impl NrlCompRead<u8> for CompRead {
    fn nrl_get(&mut self) -> u8 {
        let v = self.0.get_u8();
        if self.0.has_remaining() {
            self.0.advance(1);
        }
        v
    }
    fn nrl_advance(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.0.advance(n * 2 - 1);
        if self.0.has_remaining() {
            self.0.advance(1);
        }
    }
    fn nrl_has_remaining(&self) -> bool {
        Buf::has_remaining(&self.0)
    }
}

pub struct BpcTilemapCompressor;

impl BpcTilemapCompressor {
    pub fn run(decompressed_data: Bytes) -> PyResult<Bytes> {
        if decompressed_data.is_empty() {
            return Ok(Bytes::new());
        }
        let mut compressed_data = BytesMut::with_capacity(decompressed_data.len() * 2);

        // First we process all the high bytes (LE)
        let mut cursor = CompRead(Cursor::new(decompressed_data.clone()));
        cursor.0.advance(1);
        while cursor.nrl_has_remaining() {
            compression_step(&mut cursor, &mut compressed_data);
        }

        // And then all the low bytes (LE)
        let mut cursor = CompRead(Cursor::new(decompressed_data));
        while cursor.nrl_has_remaining() {
            compression_step(&mut cursor, &mut compressed_data);
        }

        Ok(compressed_data.freeze())
    }
}

/////////////////////////////////////////
/////////////////////////////////////////

struct DecompWrite(BytesMut);

impl NrlDecompWrite<u8> for DecompWrite {
    fn nrl_put(&mut self, b: u8) {
        self.0.put_u16_le((b as u16) << 8);
    }
}

pub struct BpcTilemapDecompressor<'a, T>
where
    T: AsRef<[u8]>,
{
    compressed_data: &'a mut Cursor<T>,
    decompressed_data: DecompWrite,
    stop_when_size: usize,
    phase2_out_pos: usize,
}

impl<'a, T> BpcTilemapDecompressor<'a, T>
where
    T: AsRef<[u8]>,
{
    pub fn run(compressed_data: &'a mut Cursor<T>, stop_when_size: usize) -> PyResult<Bytes> {
        let mut slf = Self {
            compressed_data,
            decompressed_data: DecompWrite(BytesMut::with_capacity(stop_when_size)),
            stop_when_size,
            phase2_out_pos: 0,
        };

        // Handle high bytes
        while slf.decompressed_data.0.len() < slf.stop_when_size {
            if !slf.compressed_data.has_remaining() {
                return Err(exceptions::PyValueError::new_err(format!(
                    "BPC Tilemap Decompressor: Phase1: End result length unexpected. \
                    Should be {}, is {}.",
                    slf.stop_when_size,
                    slf.decompressed_data.0.len()
                )));
            }

            decompression_step(slf.compressed_data, &mut slf.decompressed_data);
        }

        if slf.decompressed_data.0.len() > slf.stop_when_size {
            slf.decompressed_data.0.truncate(slf.stop_when_size);
        }

        while slf.phase2_out_pos < slf.stop_when_size {
            if !slf.compressed_data.has_remaining() {
                return Err(exceptions::PyValueError::new_err(format!(
                    "BPC Tilemap Decompressor: Phase2: End result length unexpected. \
                    Should be {}, is {}.",
                    slf.stop_when_size, slf.phase2_out_pos
                )));
            }

            slf.process_phase2();
        }

        Ok(slf.decompressed_data.0.freeze())
    }

    fn process_phase2(&mut self) {
        let cmd = self.compressed_data.get_u8();
        if cmd < CMD_2_SEEK_OFFSET {
            // We skip over the nb of words indicated by the cmd
            self.phase2_out_pos += (cmd as usize + 1) * 2;
            //debug_assert!(self.phase2_out_pos <= self.stop_when_size);
        } else if (CMD_2_FILL_LOW..CMD_2_COPY_LOW).contains(&cmd) {
            // cmd - CMD_2_SEEK_OFFSET is the nb of words to write with the next byte as low byte
            let cmd_value = self.compressed_data.get_u8() as u16;
            for _ in CMD_2_SEEK_OFFSET - 1..cmd {
                let bytes: [u8; 2] = (u16::from_le_bytes(
                    self.decompressed_data.0[self.phase2_out_pos..=self.phase2_out_pos + 1]
                        .try_into()
                        .unwrap(),
                ) | cmd_value)
                    .to_le_bytes();
                self.decompressed_data.0[self.phase2_out_pos] = bytes[0];
                self.decompressed_data.0[self.phase2_out_pos + 1] = bytes[1];
                self.phase2_out_pos += 2;
            }
        } else {
            //  cmd - CMD_2_COPY_LOW is the nb of words to write with the sequence of bytes as low byte
            for _ in CMD_2_COPY_LOW - 1..cmd {
                let bytes: [u8; 2] = (u16::from_le_bytes(
                    self.decompressed_data.0[self.phase2_out_pos..=self.phase2_out_pos + 1]
                        .try_into()
                        .unwrap(),
                ) | self.compressed_data.get_u8() as u16)
                    .to_le_bytes();
                self.decompressed_data.0[self.phase2_out_pos] = bytes[0];
                self.decompressed_data.0[self.phase2_out_pos + 1] = bytes[1];
                self.phase2_out_pos += 2;
            }
        }
    }
}

/////////////////////////////////////////
/////////////////////////////////////////

// "Private" container for compressed data for use with tests written in Python (skytemple-files):
#[pyclass(module = "skytemple_rust._st_bpc_tilemap_compression")]
#[derive(Clone)]
pub(crate) struct BpcTilemapCompressionContainer {
    compressed_data: Bytes,
    length_decompressed: u16,
}

impl BpcTilemapCompressionContainer {
    pub fn compress(data: &[u8]) -> PyResult<Self> {
        let compressed_data = BpcTilemapCompressor::run(Bytes::copy_from_slice(data))?;
        Ok(Self {
            length_decompressed: data.len() as u16,
            compressed_data,
        })
    }
    fn cont_size(data: Bytes, byte_offset: usize) -> u16 {
        (data.len() - byte_offset) as u16
    }
}

#[pymethods]
impl BpcTilemapCompressionContainer {
    const DATA_START: usize = 8;
    const MAGIC: &'static [u8; 6] = b"BPCTLM";

    #[new]
    pub fn new(data: &[u8]) -> PyResult<Self> {
        let mut data = Bytes::from(data.to_vec());
        data.advance(6);
        let length_decompressed = data.get_u16_le();
        Ok(Self {
            compressed_data: data,
            length_decompressed,
        })
    }
    pub fn decompress(&self) -> PyResult<StBytesMut> {
        let mut cur = Cursor::new(self.compressed_data.clone());
        Ok(BpcTilemapDecompressor::run(&mut cur, self.length_decompressed as usize)?.into())
    }
    pub fn to_bytes(&self) -> StBytesMut {
        let mut res = BytesMut::with_capacity(self.compressed_data.len() + Self::DATA_START);
        res.put(Bytes::from_static(Self::MAGIC));
        res.put_u16_le(self.length_decompressed);
        res.put(self.compressed_data.clone());
        res.into()
    }
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(signature = (data, byte_offset = 0))]
    #[pyo3(name = "cont_size")]
    fn _cont_size(_cls: &PyType, data: crate::bytes::StBytes, byte_offset: usize) -> u16 {
        Self::cont_size(data.0, byte_offset)
    }
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "compress")]
    fn _compress(_cls: &PyType, data: &[u8]) -> PyResult<Self> {
        Self::compress(data)
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bpc_tilemap_compression_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust._st_bpc_tilemap_compression";
    let m = PyModule::new(py, name)?;
    m.add_class::<BpcTilemapCompressionContainer>()?;

    Ok((name, m))
}
