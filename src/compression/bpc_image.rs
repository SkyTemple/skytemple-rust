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
use crate::python::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Cursor;
use std::mem::swap;

// COMPRESSION CONSTANTS
// How much space is in the CMD byte to store the number of repetitions
const BPC_IMGC_REPEAT_MAX_CMD: u8 = 31 - 1; // -1 because of the __NEXT reserved special case
                                            // Same for the CMD_LOAD_BYTE_AS_PATTERN_AND_CP case (C0-80)=64(dec.)
const BPC_IMGC_REPEAT_MAX_CMD_LOAD_AS_PATTERN: u8 = 63 - 1;
// How much space for the __NEXT case there is (storing it in one separate byte)
const BPC_IMGC_REPEAT_MAX_NEXT: u8 = 254;
//
// Copy pattern limits
const BPC_IMGC_COPY_MAX_CMD: u8 = 127 - 2; // -2 because of the __NEXT reserved special cases
                                           // How much space for the __NEXT case there is (storing it in one separate byte)
const BPC_IMGC_COPY_MAX_NEXT: u8 = 255;
// How much space for the CMD_COPY__NEXT__LE_16 case there is (storing as 16 bit after cmd byte)
const BPC_IMGC_COPY_MAX_NEXT_16B: u16 = 0xffff;

// Minimum repeat count for using the pattern ops.
const BPC_MIN_REPEAT_COUNT: usize = 3;

// DECOMPRESSION CONSTANTS
//  Operations are encoded in command bytes (CMD):
//  BASE OPERATIONS
//const CMD_CP_FROM_POS: u8                         = 0x80;  //  All values below: Copy from pos
//  We build and copy a pattern:
const CMD_CYCLE_PATTERN_AND_CP: u8 = 0xE0; //  All values equal/above
const CMD_USE_LAST_PATTERN_AND_CP: u8 = 0xC0; //  All values equal/above until next
const CMD_LOAD_BYTE_AS_PATTERN_AND_CP: u8 = 0x80; //  All values equal/above until next

//  SPECIAL OPERATIONS (ALl values equal)
//  Base operations, but the number of bytes to copy is stored in the next byte, not the CMD
const CMD_CYCLE_PATTERN_AND_CP__NEXT: u8 = 0xFF;
const CMD_USE_LAST_PATTERN_AND_CP__NEXT: u8 = 0xDF;
const CMD_LOAD_BYTE_AS_PATTERN_AND_CP__NEXT: u8 = 0xBF;
const CMD_CP_FROM_POS__NEXT: u8 = 0x7E;
//  In list for if:
const CMD__NEXT: [u8; 4] = [
    CMD_CP_FROM_POS__NEXT,
    CMD_CYCLE_PATTERN_AND_CP__NEXT,
    CMD_LOAD_BYTE_AS_PATTERN_AND_CP__NEXT,
    CMD_USE_LAST_PATTERN_AND_CP__NEXT,
];
//  Like above, but with the next 16-bit LE int:
const CMD_COPY__NEXT__LE_16: u8 = 0x7F;

/////////////////////////////////////////
/////////////////////////////////////////

#[derive(PartialEq, Eq)]
enum WherePattern {
    WriteAsByte,
    IsCurrentPattern,
    IsPreviousPatternCycle,
}

enum ByteOrPattern {
    Byte(u8),
    Pattern(Bytes),
}

impl Default for ByteOrPattern {
    fn default() -> Self {
        ByteOrPattern::Byte(0)
    }
}

#[derive(Default)]
struct BpcImageCompressorOperation {
    // Byte for repeat case with WherePattern.WRITE_AS_BYTE and sequence for COPY. None otherwise
    byte_or_sequence: ByteOrPattern,
    where_pattern: Option<WherePattern>, // only relevant for pattern_op = True
    repeats: u16,
}

pub struct BpcImageCompressor {
    decompressed_data: Bytes,
    compressed_data: BytesMut,
    // The currently stored pattern. This has to be in-sync with the decompression!
    pattern: u8,
    // The previously stored pattern. Also has to be in-sync!
    pattern_buffer: u8,
}

impl BpcImageCompressor {
    pub fn run(decompressed_data: Bytes) -> PyResult<Bytes> {
        if decompressed_data.len() % 2 != 0 {
            return Err(exceptions::PyValueError::new_err(
                "BPC Image compressor can only compress data with an even length.",
            ));
        }
        let mut slf = Self {
            compressed_data: BytesMut::with_capacity(decompressed_data.len() * 2),
            decompressed_data,
            pattern: 0,
            pattern_buffer: 0,
        };

        while slf.decompressed_data.has_remaining() {
            slf.process()
        }

        Ok(slf.compressed_data.freeze())
    }

    /// Process a single byte. This reads first and builds an operation,
    /// and then calls _run_operation to run the actual operation.
    fn process(&mut self) {
        let mut op = BpcImageCompressorOperation::default();
        // Check if byte repeats
        let (repeat_pattern, repeat_count) = self.look_ahead_repeats();
        if repeat_count >= BPC_MIN_REPEAT_COUNT as u8 {
            op.repeats = repeat_count as u16;
            // Don't forget to also advance the read cursor! The lookahead functions don't do that
            self.decompressed_data.advance((repeat_count + 1) as usize);
            if repeat_pattern == self.pattern {
                // Check if byte is current pattern
                op.where_pattern = Some(WherePattern::IsCurrentPattern);
            } else if repeat_pattern == self.pattern_buffer {
                // ...or the previous stored pattern...
                op.where_pattern = Some(WherePattern::IsPreviousPatternCycle);
            } else {
                // ...or something new.
                op.where_pattern = Some(WherePattern::WriteAsByte);
                op.byte_or_sequence = ByteOrPattern::Byte(repeat_pattern);
            }

            // Run the actual operation
            self.run_pattern_operation(op)
        } else {
            // If not: COPY_BYTES
            let sequence = self.look_ahead_byte_sequence();
            op.repeats = (sequence.len() - 1) as u16;
            // Don't forget to also advance the read cursor! The lookahead functions don't do that
            self.decompressed_data.advance(sequence.len());
            op.byte_or_sequence = ByteOrPattern::Pattern(sequence);

            // Run the actual operation
            self.run_copy_operation(op)
        }
    }

    /// Write a pattern operation
    fn run_pattern_operation(&mut self, op: BpcImageCompressorOperation) {
        match op.byte_or_sequence {
            ByteOrPattern::Byte(out_byte) => {
                let mut cmd;
                if op.where_pattern == Some(WherePattern::IsCurrentPattern) {
                    cmd = CMD_USE_LAST_PATTERN_AND_CP;
                    // Nothing to change for pattern buffers
                } else if op.where_pattern == Some(WherePattern::IsPreviousPatternCycle) {
                    cmd = CMD_CYCLE_PATTERN_AND_CP;
                    // The decompressor will now swap the pattern buffers
                    swap(&mut self.pattern, &mut self.pattern_buffer);
                } else {
                    cmd = CMD_LOAD_BYTE_AS_PATTERN_AND_CP;
                    // This now means, that we will write the pattern into the next byte.
                    // The decompressor will load this and change it's pattern buffers like so:
                    self.pattern_buffer = self.pattern;
                    self.pattern = out_byte;
                }

                // Determine the length
                if op.repeats <= BPC_IMGC_REPEAT_MAX_CMD as u16
                    || (op.repeats <= BPC_IMGC_REPEAT_MAX_CMD_LOAD_AS_PATTERN as u16
                        && op.where_pattern == Some(WherePattern::WriteAsByte))
                {
                    // Fits in CMD
                    cmd += op.repeats as u8;
                    self.compressed_data.put_u8(cmd);
                } else {
                    // Store in next byte
                    cmd = CMD_LOAD_BYTE_AS_PATTERN_AND_CP__NEXT;
                    if op.where_pattern == Some(WherePattern::IsCurrentPattern) {
                        cmd = CMD_USE_LAST_PATTERN_AND_CP__NEXT;
                    } else if op.where_pattern == Some(WherePattern::IsPreviousPatternCycle) {
                        cmd = CMD_CYCLE_PATTERN_AND_CP__NEXT;
                    }
                    self.compressed_data.put_u8(cmd);
                    self.compressed_data.put_u8(op.repeats as u8);
                }

                if op.where_pattern == Some(WherePattern::WriteAsByte) {
                    // Don't forget to write the pattern as a byte
                    self.compressed_data.put_u8(out_byte)
                }
            }
            _ => panic!("Invalid state in compressor."),
        }
    }

    /// Write an instruction to copy the following bytes and paste that sequence
    fn run_copy_operation(&mut self, op: BpcImageCompressorOperation) {
        // Determine the length
        if op.repeats <= BPC_IMGC_COPY_MAX_CMD as u16 {
            // Fits in CMD
            self.compressed_data.put_u8(op.repeats as u8);
            //             self._write(op.repeats)
        } else if op.repeats <= BPC_IMGC_COPY_MAX_NEXT as u16 {
            // Fits in one byte
            self.compressed_data.put_u8(CMD_CP_FROM_POS__NEXT);
            self.compressed_data.put_u8(op.repeats as u8);
        } else {
            // Fits in two bytes (LE!)
            self.compressed_data.put_u8(CMD_COPY__NEXT__LE_16);
            self.compressed_data.put_u16_le(op.repeats);
        }
        // Write the sequence
        match op.byte_or_sequence {
            ByteOrPattern::Pattern(sequence) => {
                // + 1 since we are counting repeats and always have 1
                debug_assert_eq!((op.repeats + 1) as usize, sequence.len());
                self.compressed_data.put(sequence);
            }
            _ => panic!("Invalid state in compressor."),
        }
    }

    /// Look how often the byte in the input data repeats, up to NRL_LOOKAHEAD_MAX_BYTES
    fn look_ahead_repeats(&mut self) -> (u8, u8) {
        let mut nc = self.decompressed_data.clone();
        let byte_at_pos = nc.get_u8();
        let mut repeats = 0;
        while nc.has_remaining() && nc.get_u8() == byte_at_pos && repeats < BPC_IMGC_REPEAT_MAX_NEXT
        {
            repeats += 1;
        }
        (byte_at_pos, repeats)
    }

    fn look_ahead_byte_sequence(&self) -> Bytes {
        let mut seq = BytesMut::with_capacity(BPC_IMGC_COPY_MAX_NEXT_16B as usize);
        // If the repeat counter reaches BPC_MIN_REPEAT_COUNT,
        // the sequence ends BPC_MIN_REPEAT_COUNT entries before that
        let mut repeat_counter = 0;
        let mut previous_byt_at_pos = None;
        let mut nc = self.decompressed_data.clone();
        loop {
            let byt_at_pos = nc.get_u8();
            repeat_counter = if Some(byt_at_pos) == previous_byt_at_pos {
                repeat_counter + 1
            } else {
                0
            };

            previous_byt_at_pos = Some(byt_at_pos);
            seq.put_u8(byt_at_pos);

            if repeat_counter > BPC_MIN_REPEAT_COUNT {
                seq.truncate(seq.len() - BPC_MIN_REPEAT_COUNT - 1);
                break;
            }

            if seq.len() + 1 >= BPC_IMGC_COPY_MAX_NEXT_16B as usize || !nc.has_remaining() {
                break;
            }
        }

        seq.freeze()
    }
}

/////////////////////////////////////////
/////////////////////////////////////////

pub struct BpcImageDecompressor<'a, T>
where
    T: 'a + AsRef<[u8]>,
{
    compressed_data: &'a mut Cursor<T>,
    decompressed_data: BytesMut,
    stop_when_size: usize,
    has_leftover: bool,
    leftover: u16,
    pattern: u16,
    pattern_buffer: u16,
}

impl<'a, T> BpcImageDecompressor<'a, T>
where
    T: 'a + AsRef<[u8]>,
{
    pub fn run(compressed_data: &'a mut Cursor<T>, stop_when_size: usize) -> PyResult<Bytes> {
        if stop_when_size % 2 != 0 {
            return Err(exceptions::PyValueError::new_err(
                "BPC Image compressor can only decompress data with an even output length.",
            ));
        }
        let mut slf = Self {
            decompressed_data: BytesMut::with_capacity(stop_when_size),
            compressed_data,
            stop_when_size,
            has_leftover: false,
            leftover: 0,
            pattern: 0,
            pattern_buffer: 0,
        };

        while slf.decompressed_data.len() < slf.stop_when_size {
            if !slf.compressed_data.has_remaining() {
                // if we have leftover, try processing that before erroring.
                if slf.has_leftover {
                    if slf.stop_when_size - slf.decompressed_data.len() == 2 {
                        slf.decompressed_data.put_u16_le(slf.leftover);
                        break;
                    } else if slf.stop_when_size - slf.decompressed_data.len() == 1 {
                        slf.decompressed_data.put_u8((slf.leftover & 0xFF) as u8);
                        break;
                    }
                }
                return Err(exceptions::PyValueError::new_err(format!(
                    "BPC Image Decompressor: End result length unexpected. \
                    Should be {}, is {}.",
                    slf.stop_when_size,
                    slf.decompressed_data.len()
                )));
            }

            slf.process()?
        }

        Ok(slf.decompressed_data.freeze())
    }
    /// Process a single run.
    fn process(&mut self) -> PyResult<()> {
        let cmd = self.compressed_data.get_u8();
        let number_of_bytes_to_output = self.read_nb_bytes_to_output(cmd);

        // Perform the special pattern operations based on the current CMD's value:
        if Self::should_cycle_pattern(cmd) {
            swap(&mut self.pattern_buffer, &mut self.pattern);
        }
        if Self::is_loading_pattern_from_next_byte(cmd) {
            self.pattern = self.compressed_data.get_u8() as u16;
        }

        // Check if we have leftover byte patterns to add:
        // This leftover exists because we are always working with words (= 2 bytes).
        // Keep in mind, that the number of bytes to write, is actually one lower than it should actually be. On odd
        // numbers there are supposed to be one word written more. Because we are always writing two words,
        // this is the case. However if there is an even amount of number_of_bytes_to_output, we are actually missing
        // one written byte because the ACTUAL amount of bytes to write is actually one higher.
        // So we need to "fill" the boundaries like this.
        // This is also why we decrease the number_of_bytes_to_output in the _read_nb_words_to_output
        // method: We are writing this byte right here!
        // Example:
        // number_of_bytes_to_output = 2
        //   -> bb [leftover = true]
        //   On next run:
        //   -> bb bx xx ...
        //         #
        // number_of_bytes_to_output = 3
        //   -> bb bb [leftover = false]
        //   On next run:
        //   -> bb bb xx ...

        if self.has_leftover {
            self.handle_leftover(cmd);
        }

        if number_of_bytes_to_output >= 0 {
            self.handle_main_operation(cmd, number_of_bytes_to_output as u16);
        }
        Ok(())
    }

    fn handle_leftover(&mut self, cmd: u8) {
        let final_pattern = if Self::is_pattern_op(cmd) {
            self.leftover | (self.pattern << 8)
        } else {
            self.leftover | (self.compressed_data.get_u8() as u16) << 8
        };
        self.decompressed_data.put_u16_le(final_pattern);
        self.has_leftover = false;
    }

    fn handle_main_operation(&mut self, cmd: u8, number_of_bytes_to_output: u16) {
        if Self::is_pattern_op(cmd) {
            // We are writing the stored pattern!
            // Convert current stored pattern in a 2 byte repeating pattern
            let pattern = self.pattern | (self.pattern << 8);
            for _ in (0..number_of_bytes_to_output).step_by(2) {
                self.decompressed_data.put_u16_le(pattern)
            }
        } else {
            // We are copying whatever comes next!
            for _ in (0..number_of_bytes_to_output).step_by(2) {
                self.decompressed_data
                    .put_u16_le(self.compressed_data.get_u16_le())
            }
        }

        // If the amount copied was even, we setup the copy a leftover word on the next command byte
        if number_of_bytes_to_output % 2 == 0 {
            self.has_leftover = true;
            self.leftover = if Self::is_pattern_op(cmd) {
                self.pattern
            } else {
                self.compressed_data.get_u8() as u16
            }
        }
    }

    /// Determine the number of bytes to output. This is controlled by the CMD value.
    fn read_nb_bytes_to_output(&mut self, cmd: u8) -> i32 {
        let mut nb = if CMD__NEXT.contains(&cmd) {
            // Number is encoded in next byte
            self.compressed_data.get_u8() as i32
        } else if cmd == CMD_COPY__NEXT__LE_16 {
            // Number is encoded in next two bytes
            self.compressed_data.get_u16_le() as i32
        } else {
            // Number is in CMD. Depending on the case, we may need to subtract different things.
            let mut nb = cmd;

            if cmd >= CMD_CYCLE_PATTERN_AND_CP {
                nb -= CMD_CYCLE_PATTERN_AND_CP
            } else if cmd >= CMD_USE_LAST_PATTERN_AND_CP {
                nb -= CMD_USE_LAST_PATTERN_AND_CP
            } else if cmd >= CMD_LOAD_BYTE_AS_PATTERN_AND_CP {
                nb -= CMD_LOAD_BYTE_AS_PATTERN_AND_CP
            }

            nb as i32
        };

        // When we currently have a leftover word, we subtract one word to read
        if self.has_leftover {
            nb -= 1
        }
        nb
    }

    #[inline]
    fn should_cycle_pattern(cmd: u8) -> bool {
        Self::is_loading_pattern_from_next_byte(cmd) || (CMD_CYCLE_PATTERN_AND_CP <= cmd)
        // always true then: && cmd <= CMD_CYCLE_PATTERN_AND_CP__NEXT
    }

    #[inline]
    fn is_loading_pattern_from_next_byte(cmd: u8) -> bool {
        (CMD_LOAD_BYTE_AS_PATTERN_AND_CP..CMD_USE_LAST_PATTERN_AND_CP).contains(&cmd)
    }

    #[inline]
    fn is_pattern_op(cmd: u8) -> bool {
        cmd >= CMD_LOAD_BYTE_AS_PATTERN_AND_CP
    }
}

// "Private" container for compressed data for use with tests written in Python (skytemple-files):
#[pyclass(module = "skytemple_rust._st_bpc_image_compression")]
#[derive(Clone)]
pub(crate) struct BpcImageCompressionContainer {
    compressed_data: Bytes,
    length_decompressed: u16,
}

impl BpcImageCompressionContainer {
    pub fn compress(data: &[u8]) -> PyResult<Self> {
        let compressed_data = BpcImageCompressor::run(Bytes::copy_from_slice(data))?;
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
impl BpcImageCompressionContainer {
    const DATA_START: usize = 8;
    const MAGIC: &'static [u8; 6] = b"BPCIMG";

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
        Ok(BpcImageDecompressor::run(&mut cur, self.length_decompressed as usize)?.into())
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
pub(crate) fn create_st_bpc_image_compression_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust._st_bpc_image_compression";
    let m = PyModule::new(py, name)?;
    m.add_class::<BpcImageCompressionContainer>()?;

    Ok((name, m))
}
