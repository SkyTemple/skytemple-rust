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
use bytes::{Buf, BufMut, Bytes, BytesMut};
/// Written by an anonymous contributor, ported by Capyara.
/// Based on https://github.com/pleonex/tinke/blob/master/Plugins/999HRPERDOOR/999HRPERDOOR/AT6P.cs
/// To work with the game, a patch is required!
use std::io::Cursor;
use std::iter::once;
use std::mem::swap;

pub struct Custom999Compressor;
impl Custom999Compressor {
    pub fn run<F>(buffer: F) -> BytesMut
    where
        F: Buf + IntoIterator<Item = u8>,
    {
        let data: Vec<u8> = buffer.into_iter().flat_map(|x| [x % 16, x / 16]).collect();

        // For the original algorithm:
        // let data = buffer

        let first = data[0];
        // Add another 0 byte for the original algorithm
        // compressed.push(0)
        let mut prev = data[0];
        let mut current = data[0];
        let mut bit_list: Vec<bool> = Vec::with_capacity(data.len());
        for b in data.into_iter().skip(1) {
            if b == current {
                bit_list.push(true);
            } else if b == prev {
                bit_list.extend([false, true, false]);
                swap(&mut prev, &mut current);
            } else {
                prev = current;
                let mut diff: i16 = b as i16 - current as i16;
                let mut sign: i8;
                if diff < 0 {
                    diff = diff.abs();
                    sign = -1;
                } else {
                    sign = 1;
                }
                if diff >= 8 {
                    // For the original algorithm: diff >= 0x80
                    diff = 0x10 - diff; // For the original algorithm: diff = 0x100-diff
                    sign = -sign
                }
                let mut code: usize = if sign > 0 { 0 } else { 1 };
                code = (code as i16 + (diff << 1)) as usize;
                let len_code = format!("{:b}", code + 1).len() - 1;
                code = (code + 1) % 2_usize.pow(len_code as u32);

                let mut tmp = (0..len_code)
                    .map(|_| {
                        bit_list.push(false);
                        let val = code % 2;
                        code /= 2;
                        val > 0
                    })
                    .collect();
                bit_list.push(true);
                bit_list.append(&mut tmp);
                current = b;
            }
        }
        /*let mut compressed: VecDeque<u8> = vec![first];
        let bit_list: Vec<u8> = bit_list.into_iter().map(|x| if b {1} else {0}).collect();
        let mut bit_list_slice = &bit_list[..];
        while bit_list.len() > 0 {
            bit_list_slice.advance(8);
            compressed.push(0);
            for (i, b) in bit_list_slice.chunk().iter().enumerate() {
                compressed.push
            }
        }
        compressed*/
        once(first)
            .chain(bit_list.chunks(8).map(|currentl| {
                // Turn the bits into an u8
                currentl
                    .iter()
                    .enumerate()
                    .map(|(i, &b)| if b { 1 } else { 0 } * (2_u8.pow(i as u32)))
                    .sum()
            }))
            .collect()
    }
}

/////////////////////////////////////////

pub struct Custom999Decompressor;
impl Custom999Decompressor {
    pub fn run(compressed_data: &[u8], decompressed_size: usize) -> impl Buf + Into<StBytesMut> {
        let mut compressed_cur = Cursor::new(compressed_data);
        let mut decompressed = BytesMut::with_capacity(decompressed_size);
        let mut code = compressed_cur.get_u8();
        decompressed.put_u8(code);
        let mut prev = code;

        // In the original 999 algorithm:
        // buffer.advance(1);

        let mut nbits = 0;
        let mut flags = 0;

        while decompressed.len() < decompressed_size * 2 {
            while nbits < 17 {
                if compressed_cur.position() < compressed_data.len() as u64 {
                    flags |= (compressed_cur.get_u8() as usize) << nbits;
                }
                nbits += 8;
            }
            let mut outnbit = 8;
            for nbit in 0..=8 {
                if (flags & (1 << nbit)) != 0 {
                    outnbit = nbit;
                    break;
                }
            }
            let mut n: usize = (1 << outnbit) - 1;
            n += (flags >> (outnbit + 1)) & n;

            // ??? unused
            //let mut current_flag = compressed_cur.position() - nbits / 8;
            //if nbits % 8 != 0 {
            //    current_flag -= 1;
            //}
            if n == 1 {
                decompressed.put_u8(prev);
                swap(&mut prev, &mut code);
            } else {
                if n != 0 {
                    prev = code;
                }
                code = ((code as i64 + (n >> 1) as i64 * (1 - 2 * (n & 1) as i64)) & 0xF) as u8; // & 0xFF in the original algorithm
                decompressed.put_u8(code);
            }
            flags >>= 2 * outnbit + 1;
            nbits -= 2 * outnbit + 1;
        }

        // In the original algorithm:
        // return decompressed.freeze();

        let decompressed_done = decompressed
            .chunks(2)
            .map(|x| x[0] + x[1] * 16)
            .collect::<Bytes>();
        debug_assert_eq!(decompressed_size, decompressed_done.len());
        decompressed_done
    }
}
