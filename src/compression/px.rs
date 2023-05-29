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

use crate::python::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::VecDeque;
use std::io::{Cursor, Read, Seek, SeekFrom};

// Length of the default lookback buffer. The "sliding window" so to speak!
// Used to determine how far back the compressor looks for matching sequences!
const PX_LOOKBACK_BUFFER_SIZE: usize = 4096;
// The longest sequence of similar bytes we can use!
const PX_MAX_MATCH_SEQLEN: usize = 18;
// The shortest sequence of similar bytes we can use!
const PX_MIN_MATCH_SEQLEN: usize = 3;
// The nb of unique lengths we can use when copying a sequence.
// This is due to ctrl flags taking over a part of the value range between 0x0 and 0xF
// The amount of possible lengths a sequence to lookup can have, considering
// there are 9 ctrl flags, and only 0 to 15 as range to contain all that info!
// 9 + 7 = 16
const PX_NB_POSSIBLE_SEQUENCES_LEN: usize = 7;

#[repr(i8)]
#[derive(PartialEq, Eq, PartialOrd, FromPrimitive, Debug)]
#[allow(clippy::enum_variant_names)]
enum Operation {
    CopyAsis = -1,
    CopyNybble4times = 0,
    CopyNybble4timesExIncrallDecrnybble0 = 1,
    CopyNybble4timesExDecrnybble1 = 2,
    CopyNybble4timesExDecrnybble2 = 3,
    CopyNybble4timesExDecrnybble3 = 4,
    CopyNybble4timesExDecrallIncrnybble0 = 5,
    CopyNybble4timesExIncrnybble1 = 6,
    CopyNybble4timesExIncrnybble2 = 7,
    CopyNybble4timesExIncrnybble3 = 8,
    CopySequence = 9,
}

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd)]
pub enum PxCompLevel {
    // No compression     - All command bytes are 0xFF, and values are stored uncompressed. File size is increased!
    Level0 = 0,
    // Low compression    - We handle 4 byte patterns, using only ctrl flag 0
    Level1 = 1,
    // Medium compression - We handle 4 byte patterns, using all control flags
    Level2 = 2,
    // Full compression   - We handle everything above, along with repeating sequences of bytes already decompressed.
    Level3 = 3,
}

#[derive(Debug)]
struct CompOp {
    // The operation to do
    typ: Operation, // = Operation.CopyAsis
    // The value of the compressed high nybble if applicable
    highnybble: u8, // = 0
    // The value of the compressed low nybble
    lownybble: u8, // = 0
    // Value of the compressed next byte if applicable
    nextbytevalue: u8, // = 0
}

struct MatchingSeq {
    pos: usize,
    length: usize,
}

/////////////////////////////////////////
/////////////////////////////////////////

pub struct PxCompressor<F: Buf> {
    flags: [u8; 9],
    in_buffer: F,
    in_cur_buffer: Cursor<F>,
    output: BytesMut,
    compression_level: PxCompLevel,
    should_search_first: bool,
    pending_operations: VecDeque<CompOp>,
    high_nibble_lengths_possible: Vec<usize>,
}

impl PxCompressor<Bytes> {
    pub fn run(
        buffer: Bytes,
        compression_level: PxCompLevel,
        should_search_first: bool,
    ) -> PyResult<(Bytes, [u8; 9])> {
        let input_size = buffer.len();
        if input_size > u32::MAX as usize {
            return Err(exceptions::PyValueError::new_err(format!(
                "PX Compression: The input data is too long {}.",
                input_size
            )));
        }
        let pad: usize = if input_size % 8 != 0 { 1 } else { 0 };
        let mut slf = Self {
            flags: [0; 9],
            in_buffer: buffer.clone(),
            in_cur_buffer: Cursor::new(buffer),
            // Allocate at least as much memory as the input + some extra in case of dummy compression!
            // Worst case, we got 1 more bytes per 8 bytes.
            // And if we're not divisible by 8, add an extra
            // byte for the last command byte!
            output: BytesMut::with_capacity(input_size + input_size + pad),
            compression_level,
            should_search_first,
            pending_operations: VecDeque::with_capacity(1000),
            // Set by default those two possible matching sequence length, given we want 99% of the time to
            // have those 2 to cover the gap between the 2 bytes to 1 algorithm and the string search,
            // and also get to use the string search's capability to its maximum!
            high_nibble_lengths_possible: vec![0, 0xF],
        };

        // Do compression
        while slf.in_cur_buffer.has_remaining() {
            slf.handle_a_block()?;
        }

        // Build control flag table, now that we determined all our string search lengths!
        slf.build_ctrl_flags_list();

        // Execute all operations from our queue
        slf.output_all_operations();

        // Validate compressed size
        if slf.output.len() > u16::MAX as usize {
            return Err(exceptions::PyValueError::new_err(format!(
                "PX Compression: Compressed size {} overflows 16 bits unsigned integer!",
                slf.output.len()
            )));
        }

        Ok((slf.output.freeze(), slf.flags))
    }

    fn handle_a_block(&mut self) -> PyResult<()> {
        for _ in 0..8 {
            if !self.in_cur_buffer.has_remaining() {
                return Ok(());
            }
            let best_op = self.determine_best_operation()?;
            self.pending_operations.push_back(best_op);
        }
        Ok(())
    }

    fn determine_best_operation(&mut self) -> PyResult<CompOp> {
        let mut myop = CompOp {
            typ: Operation::CopyAsis,
            highnybble: 0,
            lownybble: 0,
            nextbytevalue: 0,
        };
        if self.should_search_first
            && self.compression_level >= PxCompLevel::Level3
            && self.can_use_a_matching_sequence(self.in_cur_buffer.clone(), &mut myop)?
        {
            self.in_cur_buffer
                .advance(myop.highnybble as usize + PX_MIN_MATCH_SEQLEN);
        } else if (self.compression_level >= PxCompLevel::Level1
            && Self::can_compress_to_2_in_1_byte(self.in_cur_buffer.clone(), &mut myop))
            || (self.compression_level >= PxCompLevel::Level2
                && Self::can_compress_to_2_in_1_byte_with_manipulation(
                    self.in_cur_buffer.clone(),
                    &mut myop,
                ))
        {
            self.in_cur_buffer.advance(2);
        } else if !self.should_search_first
            && self.compression_level >= PxCompLevel::Level3
            && self.can_use_a_matching_sequence(self.in_cur_buffer.clone(), &mut myop)?
        {
            self.in_cur_buffer
                .advance(myop.highnybble as usize + PX_MIN_MATCH_SEQLEN);
        } else {
            // Level 0
            // If all else fails, add the byte as-is
            let b = self.in_cur_buffer.get_u8();
            debug_assert!(myop.typ == Operation::CopyAsis);
            myop.highnybble = (b >> 4) & 0xF;
            myop.lownybble = b & 0xF;
        }
        Ok(myop)
    }

    /// Check whether the 2 bytes at l_cusor can be stored as a single byte.
    fn can_compress_to_2_in_1_byte(mut buf: impl Buf, result: &mut CompOp) -> bool {
        let mut both_bytes: u16 = 0;
        for i in [1, 0] {
            if buf.has_remaining() {
                both_bytes |= (buf.get_u8() as u16) << (8 * i);
            } else {
                return false;
            }
        }
        result.lownybble = (both_bytes & 0xF) as u8;
        for i in [3, 2, 1, 0] {
            // Compare every nybbles with the low nybble we got above.
            // The 4 must match for this to work !
            if (both_bytes >> (4 * i)) & 0x0F != result.lownybble as u16 {
                return false;
            }
        }
        result.typ = Operation::CopyNybble4times;
        true
    }

    /// Check whether the 2 bytes at l_cursor can be stored as a single byte,
    ///  only if we use special operations based on the ctrl flag index contained
    ///  in the high nibble!
    fn can_compress_to_2_in_1_byte_with_manipulation(
        mut buf: impl Buf,
        result: &mut CompOp,
    ) -> bool {
        let mut nibbles: [u8; 4] = [0, 0, 0, 0];
        // Read 4 nibbles from the input
        for i in [0, 2] {
            if buf.has_remaining() {
                let b = buf.get_u8();
                nibbles[i] = (b >> 4) & 0x0F;
                nibbles[i + 1] = b & 0x0F;
            } else {
                return false;
            }
        }
        // Count the nb of occurrences for each nibble
        let mut nibbles_matches: [u8; 4] = [0, 0, 0, 0];
        for i in 0..4 {
            nibbles_matches[i] = nibbles.iter().filter(|x| **x == nibbles[i]).count() as u8;
        }
        // We got at least 3 values that come back 3 times
        if nibbles_matches.iter().filter(|x| **x == 3).count() >= 3 {
            let nmin = *nibbles.iter().min().unwrap();
            let nmax = *nibbles.iter().max().unwrap();
            // If the difference between the biggest and smallest nybble is one, we're good
            if nmax - nmin == 1 {
                // Get the index of the smallest value
                let indexsmallest = nibbles.iter().position(|x| *x == nmin).unwrap();
                let indexlargest = nibbles.iter().position(|x| *x == nmax).unwrap();
                if nibbles_matches[indexsmallest] == 1 {
                    // This case is for ctrl flag indexes 1 to 4. There are 2 cases here:
                    // A) The decompressor decrements a nybble not at index 0 once.
                    // B) The decompressor increments all of them once, and then decrements the one at index 0 !
                    // indexsmallest : is the index of the nybble that gets decremented.
                    result.typ = FromPrimitive::from_i8(
                        indexsmallest as i8 + Operation::CopyNybble4timesExIncrallDecrnybble0 as i8,
                    )
                    .unwrap();
                    if indexsmallest == 0 {
                        // Copy as-is, given the decompressor increment it then decrement this value
                        result.lownybble = nibbles[indexsmallest];
                    } else {
                        // Add one since we subtract 1 during decompression
                        result.lownybble = nibbles[indexsmallest] + 1;
                    }
                } else {
                    // This case is for ctrl flag indexes 5 to 8. There are 2 cases here:
                    // A) The decompressor increments a nybble not at index 0 once.
                    // B) The decompressor decrements all of them once, and then increments the one at index 0 again!
                    // indexlargest : is the index of the nybble that gets incremented.
                    result.typ = FromPrimitive::from_i8(
                        indexlargest as i8 + Operation::CopyNybble4timesExDecrallIncrnybble0 as i8,
                    )
                    .unwrap();
                    if indexlargest == 0 {
                        // Since we decrement and then increment this one during decomp, use it as-isalue
                        result.lownybble = nibbles[indexlargest];
                    } else {
                        // Subtract 1 since we increment during decompression
                        result.lownybble = nibbles[indexlargest] - 1;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Search through the lookback buffer for a string of bytes that matches the
    /// string beginning at l_cursor. It searches for at least 3 matching bytes
    /// at first, then, finds the longest matching sequence it can!
    fn can_use_a_matching_sequence(
        &mut self,
        buf: Cursor<Bytes>,
        result: &mut CompOp,
    ) -> PyResult<bool> {
        // Get offset of LookBack Buffer beginning
        let current_offset = buf.position() as usize;
        let lb_buffer_begin = if current_offset > PX_LOOKBACK_BUFFER_SIZE {
            current_offset - PX_LOOKBACK_BUFFER_SIZE
        } else {
            0
        };

        let it_seq_end = Self::adv_as_much_as_possible(
            current_offset,
            self.in_buffer.len(),
            PX_MAX_MATCH_SEQLEN,
        );

        let cur_seq_len = it_seq_end - current_offset;

        // Make sure out sequence is at least three bytes long
        if cur_seq_len < PX_MIN_MATCH_SEQLEN {
            return Ok(false);
        }
        let seqres = self.find_longest_matching_sequence(
            lb_buffer_begin,
            current_offset,
            current_offset,
            it_seq_end,
        )?;

        if seqres.length >= PX_MIN_MATCH_SEQLEN {
            // Subtract 3 given that's how they're stored!
            let mut valid_high_nibble = seqres.length - PX_MIN_MATCH_SEQLEN;
            // Check the length in the table!
            if !self.check_sequence_high_nibble_valid_or_add(valid_high_nibble) {
                // If the size is not one of the allowed ones, and we can't add it to the list,
                // shorten our found sequence to the longest length in the list of allowed lengths!
                for candidate in &self.high_nibble_lengths_possible {
                    // Since the list is sorted, just break once we can't find anything smaller than the value we found!
                    if candidate + PX_MIN_MATCH_SEQLEN < seqres.length {
                        valid_high_nibble = *candidate;
                    }
                }
                debug_assert!(valid_high_nibble <= PX_MAX_MATCH_SEQLEN - PX_MIN_MATCH_SEQLEN);
            }
            let signed_offset = -(current_offset as i64 - seqres.pos as i64);
            result.lownybble = ((signed_offset >> 8) & 0xF) as u8;
            result.nextbytevalue = (signed_offset & 0xFF) as u8;
            result.highnybble = valid_high_nibble as u8;
            result.typ = Operation::CopySequence;
            return Ok(true);
        }
        Ok(false)
    }

    /// Find the longest matching sequence of at least PX_MIN_MATCH_SEQLEN bytes
    /// and at most PX_MAX_MATCH_SEQLEN bytes.
    /// - searchbeg      : Beginning of the zone to look for the sequence.
    /// - searchend      : End of the zone to look for the sequence.
    /// - tofindbeg      : Beginning of the sequence to find.
    /// - tofindend      : End of the sequence to find.
    fn find_longest_matching_sequence(
        &self,
        searchbeg: usize,
        searchend: usize,
        tofindbeg: usize,
        tofindend: usize,
    ) -> PyResult<MatchingSeq> {
        let mut longestmatch = MatchingSeq {
            pos: searchend,
            length: 0,
        };
        let seq_to_find_short_end =
            Self::adv_as_much_as_possible(tofindbeg, tofindend, PX_MIN_MATCH_SEQLEN);

        let mut cur_search_pos = searchbeg;

        let mut searchbeg_buffer = self.in_buffer.clone();
        searchbeg_buffer.advance(tofindbeg);
        searchbeg_buffer.truncate(seq_to_find_short_end - tofindbeg);
        while cur_search_pos < searchend {
            let fnd_tpl = Self::find_subsequence(
                &self.in_buffer[cur_search_pos..searchend],
                &searchbeg_buffer,
            );
            if let Some(x) = fnd_tpl {
                cur_search_pos += x;
            } else {
                cur_search_pos = searchend;
            }

            if cur_search_pos != searchend {
                let nbmatches = Self::count_equal_consecutive_elem(
                    &self.in_buffer,
                    cur_search_pos,
                    Self::adv_as_much_as_possible(cur_search_pos, searchend, PX_MAX_MATCH_SEQLEN),
                    tofindbeg,
                    tofindend,
                );
                debug_assert!(nbmatches <= PX_MAX_MATCH_SEQLEN);
                if longestmatch.length < nbmatches {
                    longestmatch.length = nbmatches;
                    longestmatch.pos = cur_search_pos;
                }
                if nbmatches == PX_MAX_MATCH_SEQLEN {
                    return Ok(longestmatch);
                }
                cur_search_pos += 1;
            } else {
                break;
            }
        }
        Ok(longestmatch)
    }

    /// Because the length is stored as the high nybble in the compressed output, and
    /// that the high nybble also contains the ctrl flags, we need to make sure the
    /// lengths of sequences to use do not overlap over values of the control flags !
    /// So we'll build a list of length to reserve as we go!
    /// -> If the value is in our reserved list, and we have PX_NB_POSSIBLE_SEQ_LEN
    ///     of them already, return true.
    /// -> If the value isn't in our reserved list, and we still have space left,
    ///     add it and return true!
    /// -> If the value isn't in our reserved list, and all PX_NB_POSSIBLE_SEQ_LEN
    ///     slots are taken, return false!
    ///
    /// NOTE:
    ///     DO NOT pass the exact sequence length. The value stored in the
    ///     high nybble is essentially : SequenceLen - PX_MIN_MATCH_SEQLEN
    fn check_sequence_high_nibble_valid_or_add(&mut self, hnybbleorlen: usize) -> bool {
        if !self.high_nibble_lengths_possible.contains(&hnybbleorlen) {
            // We didn't find the length.. Check if we can add it.
            if self.high_nibble_lengths_possible.len() < PX_NB_POSSIBLE_SEQUENCES_LEN {
                self.high_nibble_lengths_possible.push(hnybbleorlen);
                self.high_nibble_lengths_possible.sort_unstable();
                return true;
            }
            return false;
        }
        true
    }

    /// Outputs into the output buffer at position self.output_cursor the compressed
    /// form of the operation passed in parameter!
    fn output_an_operation(&mut self, operation: CompOp) {
        if operation.typ == Operation::CopyAsis {
            self.output
                .put_u8((operation.highnybble << 4 & 0xF0) | operation.lownybble);
        } else if operation.typ == Operation::CopySequence {
            self.output
                .put_u8((operation.highnybble << 4 & 0xF0) | operation.lownybble);
            self.output.put_u8(operation.nextbytevalue);
        } else {
            let flag = self.flags[operation.typ as usize];
            self.output.put_u8((flag << 4) | operation.lownybble);
        }
    }

    /// This determines all the control flags values, based on what matching
    /// sequence lengths have been reserved so far!
    fn build_ctrl_flags_list(&mut self) {
        // Make sure we got PX_NB_POSSIBLE_SEQ_LEN values taken up by the length nybbles
        if self.high_nibble_lengths_possible.len() != PX_NB_POSSIBLE_SEQUENCES_LEN {
            // If we don't have PX_NB_POSSIBLE_SEQ_LEN nybbles reserved for the lengths,
            // just come up with some then.. Its a possible eventuality..
            for nybbleval in 0..0xF {
                if self.high_nibble_lengths_possible.len() >= PX_NB_POSSIBLE_SEQUENCES_LEN {
                    break;
                }
                if !self.high_nibble_lengths_possible.contains(&nybbleval) {
                    self.high_nibble_lengths_possible.push(nybbleval);
                }
            }
        }

        // Build our flag list, based on the allowed length values!
        // We only have 16 possible values to contain lengths and control flags..
        // Pos to insert a ctrl flag at
        let mut itctrlflaginsert = 0;
        for flagval in 0..0xF {
            if !self.high_nibble_lengths_possible.contains(&flagval) && itctrlflaginsert < 9 {
                // Flag value is not taken ! So go ahead and make it a control flag value !
                self.flags[itctrlflaginsert] = flagval as u8;
                itctrlflaginsert += 1;
            }
        }
    }

    /// This does the necessary to execute all operations we put in our operation
    //  queue. It also calculate the proper high nybble value for operation
    //  using a control flag index!
    fn output_all_operations(&mut self) {
        // Output all our operations!
        while !self.pending_operations.is_empty() {
            // Make a command byte using the 8 first operations in the operation queue !
            let mut command_byte = 0;
            for i in 0..8 {
                if i >= self.pending_operations.len() {
                    break;
                }
                if self.pending_operations[i].typ == Operation::CopyAsis {
                    // Set the bit to 1 only when we copy the byte as-is !
                    command_byte |= 1 << (7 - i);
                }
            }

            // Output command byte
            self.output.put_u8(command_byte);

            // Run 8 operations before another command byte!
            for _ in 0..8 {
                if self.pending_operations.is_empty() {
                    break;
                }
                let op = self.pending_operations.pop_front().unwrap();
                self.output_an_operation(op);
            }
        }
    }

    /// Advance an counter until either the given number of increments are made,
    /// or the end is reached!
    /// (relic from Pys's C++ code base)
    #[inline]
    fn adv_as_much_as_possible(iter: usize, itend: usize, displacement: usize) -> usize {
        if iter + displacement > itend {
            return itend;
        }
        iter + displacement
    }

    /// Count the amount of similar consecutive values between two sequences.
    /// It stops counting once it stumbles on a differing value.
    #[inline]
    fn count_equal_consecutive_elem(
        data: &[u8],
        mut first_1: usize,
        last_1: usize,
        mut first_2: usize,
        last_2: usize,
    ) -> usize {
        let mut count = 0;
        while first_1 != last_1 && first_2 != last_2 && data[first_1] == data[first_2] {
            count += 1;
            first_1 += 1;
            first_2 += 1;
        }
        count
    }

    #[inline]
    fn find_subsequence<T>(haystack: &[T], needle: &[T]) -> Option<usize>
    where
        for<'b> &'b [T]: PartialEq,
    {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}

/////////////////////////////////////////
/////////////////////////////////////////

pub struct PxDecompressor<'a, F: Buf> {
    buffer: F,
    output: BytesMut,
    flags: &'a [u8],
}

impl<'a, F> PxDecompressor<'a, F>
where
    F: Buf + Clone,
{
    pub fn run(buffer: F, flags: &'a [u8], max_size: u16) -> PyResult<Bytes> {
        let mut slf = Self {
            buffer,
            output: BytesMut::with_capacity(max_size as usize),
            flags,
        };

        while slf.buffer.remaining() > 0 {
            slf.handle_control_byte()?;
        }

        Ok(slf.output.freeze())
    }

    fn handle_control_byte(&mut self) -> PyResult<()> {
        let ctrl_byte = self.buffer.get_u8();
        for b in (0..8).rev() {
            if self.buffer.remaining() == 0 {
                break;
            }
            if ctrl_byte & (1 << b) > 0 {
                self.output.put_u8(self.buffer.get_u8());
            } else {
                self.handle_special_case()?;
            }
        }
        Ok(())
    }

    fn handle_special_case(&mut self) -> PyResult<()> {
        let next = self.buffer.get_u8();
        let hinibble = (next >> 4) & 0xF;
        let lonibble = next & 0xF;
        match self.matches_flags(hinibble) {
            Some(x) => self.insert_byte_pattern(x, lonibble),
            None => self.copy_sequence(lonibble, hinibble)?,
        }
        Ok(())
    }

    fn matches_flags(&mut self, hinibble: u8) -> Option<usize> {
        for (idx, nbl) in self.flags.iter().enumerate() {
            if *nbl == hinibble {
                return Some(idx);
            }
        }
        None
    }

    #[inline]
    fn insert_byte_pattern(&mut self, idx_ctrl_flags: usize, lonibble: u8) {
        self.output
            .put(&Self::compute_four_nibbles_pattern(idx_ctrl_flags, lonibble)[..]);
    }

    fn copy_sequence(&mut self, lonibble: u8, hinibble: u8) -> PyResult<()> {
        let offset = (-0x1000 + ((lonibble as i64) << 8)) | (self.buffer.get_u8() as i64);
        let curoutbyte: i64 = self.output.len() as i64;
        if offset < -curoutbyte {
            return Err(exceptions::PyValueError::new_err(format!(
                "Sequence to copy out of bound! Expected max. {} but got {}. \
                Either the data to decompress is not valid PX compressed data, or \
                something happened with our cursor that made us read the wrong bytes..",
                -curoutbyte, offset
            )));
        }
        let bytes_to_copy = hinibble as usize + PX_MIN_MATCH_SEQLEN;
        let mut cur = Cursor::new(&*(self.output));
        cur.seek(SeekFrom::End(offset))?;

        let mut buf: Vec<u8> = vec![0; bytes_to_copy];
        cur.read_exact(&mut buf[..])?;
        self.output.put(&*buf);

        Ok(())
    }

    fn compute_four_nibbles_pattern(idx_ctrl_flags: usize, lonibble: u8) -> [u8; 2] {
        // The index of the control flag defines modifies one or more nibbles to modify.
        //     if idx_ctrl_flags == 0:
        if idx_ctrl_flags == 0 {
            // In this case, all our 4 nibbles have the value of the "lonibble" as their value
            // Since we're dealing with half bytes, shift one left by 4 and bitwise OR it with the other!
            let n = lonibble << 4 | lonibble;
            [n, n]
        } else {
            // Here we handle 2 special cases together
            let mut nibble_base = lonibble;
            // At these indices exactly, the base value for all nibbles has to be changed:
            if idx_ctrl_flags == 1 {
                nibble_base += 1;
            } else if idx_ctrl_flags == 5 {
                nibble_base -= 1;
            }
            let mut ns = [nibble_base, nibble_base, nibble_base, nibble_base];
            // In these cases, only specific nibbles have to be changed:
            if (1..=4).contains(&idx_ctrl_flags) {
                ns[idx_ctrl_flags - 1] -= 1;
            } else {
                ns[idx_ctrl_flags - 5] += 1;
            }
            [ns[0] << 4 | ns[1], ns[2] << 4 | ns[3]]
        }
    }
}
