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

use crate::encoding_utils::StrCharIndex;
use encoding::{ByteWriter, CodecError, Encoding, RawDecoder, RawEncoder, StringWriter};
use encoding_index_singlebyte::windows_1252;

#[derive(Clone, Copy)]
pub struct Pmd2Encoding;

impl Encoding for Pmd2Encoding {
    fn name(&self) -> &'static str {
        "pmd2str"
    }
    fn raw_encoder(&self) -> Box<dyn RawEncoder> {
        Pmd2Encoder::new()
    }
    fn raw_decoder(&self) -> Box<dyn RawDecoder> {
        Pmd2Decoder::new()
    }
}

#[derive(Clone, Copy)]
pub struct Pmd2Encoder;

impl Pmd2Encoder {
    pub fn new() -> Box<Self> {
        Box::new(Pmd2Encoder)
    }
}

impl RawEncoder for Pmd2Encoder {
    fn from_self(&self) -> Box<dyn RawEncoder> {
        Self::new()
    }

    fn is_ascii_compatible(&self) -> bool {
        true
    }

    fn raw_feed(
        &mut self,
        input: &str,
        output: &mut dyn ByteWriter,
    ) -> (usize, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i, j), ch) in input.index_iter() {
            match ch {
                '\u{0}'..='\u{80}' => {
                    output.write_byte(ch as u8);
                }
                '♂' => output.write_byte(0xBD),
                '♀' => output.write_byte(0xBE),
                _ => {
                    // Is either ANSI
                    let index = windows_1252::backward(ch as u32);
                    if index != 0 {
                        output.write_byte(index);
                    } else {
                        // Or corresponds to a shift jis char
                        let sjindex = pmdshiftjis::backward(ch as u32);
                        if sjindex == 0 {
                            return (
                                i,
                                Some(CodecError {
                                    upto: j as isize,
                                    cause: format!("unrepresentable character ({})", ch).into(),
                                }),
                            );
                        } else {
                            output.write_byte(0x81);
                            output.write_byte(sjindex);
                        }
                    }
                }
            }
        }
        (input.len(), None)
    }

    fn raw_finish(&mut self, _output: &mut dyn ByteWriter) -> Option<CodecError> {
        None
    }
}

#[derive(Clone, Copy)]
pub struct Pmd2Decoder {
    st: pmd2dec::State,
}

impl Pmd2Decoder {
    pub fn new() -> Box<Self> {
        Box::new(Pmd2Decoder {
            st: Default::default(),
        })
    }
}

impl RawDecoder for Pmd2Decoder {
    fn from_self(&self) -> Box<dyn RawDecoder> {
        Self::new()
    }

    fn is_ascii_compatible(&self) -> bool {
        true
    }

    fn raw_feed(
        &mut self,
        input: &[u8],
        output: &mut dyn StringWriter,
    ) -> (usize, Option<CodecError>) {
        let (st, processed, err) = pmd2dec::raw_feed(self.st, input, output, &());
        self.st = st;
        (processed, err)
    }

    fn raw_finish(&mut self, output: &mut dyn StringWriter) -> Option<CodecError> {
        let (st, err) = pmd2dec::raw_finish(self.st, output, &());
        self.st = st;
        err
    }
}

stateful_decoder! {
    module pmd2dec;

initial:
    // shift_jis lead = 0x00
    state S0(ctx: Context) {
        case b @ 0x00..=0x80 => ctx.emit(b as u32);
        case _b @ 0x8D => ctx.emit('♂' as u32);
        case _b @ 0x8E => ctx.emit('♀' as u32);
        case _b @ 0x81 => S1(ctx);
        case b @ 0x82..=0xFF => match encoding_index_singlebyte::windows_1252::forward(b) {
            0xffff => ctx.backup_and_err(1, "invalid sequence"), // unconditional
            ch => ctx.emit(ch as u32)
        };
    }

transient:
    state S1(ctx: Context) {
        case b => match crate::encoding::pmd2_encoder::pmdshiftjis::forward(b) {
            0xffff => ctx.backup_and_err(1, "invalid sequence"), // unconditional
            ch => ctx.emit(ch as u32)
        };
    }
}

mod pmdshiftjis {
    static FORWARD_TABLE: &[u16] = &[
        0, 65309, 8800, 65308, 65310, 8806, 8807, 8734, 8756, 0, 0, 0, 8242, 8243, 8451, 65509,
        65284, 0, 0, 65285, 65283, 65286, 65290, 65312, 0, 9734, 9733, 9675, 9679, 9678, 9671,
        9670, 9633, 9632, 9651, 9650, 9661, 9660, 8251, 12306, 8594, 8592, 8593, 8595, 12307, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 8712, 8715, 8838, 8839, 8834, 8835, 8746, 8745, 12288, 12289,
        12290, 65292, 65294, 12539, 65306, 65307, 8743, 8744, 12443, 8658, 8660, 8704, 8707, 65342,
        65507, 65343, 12541, 12542, 12445, 12446, 12291, 20189, 12293, 12294, 8736, 8869, 8978,
        8706, 8711, 8801, 8786, 8810, 8811, 8730, 8765, 8733, 8757, 8747, 8748, 65288, 65289,
        12308, 12309, 65339, 65341, 65371, 8491, 12296, 9839, 9837, 9834, 12300, 12301, 12302,
        12303, 12304, 12305, 65291, 9711, 0, 0, 0,
    ];

    /// Returns the index code point for pointer `code` in this index.
    #[inline]
    pub fn forward(code: u8) -> u16 {
        FORWARD_TABLE[(code - 0x80) as usize]
    }

    /// Returns the index pointer for code point `code` in this index.
    #[inline]
    pub fn backward(code: u32) -> u8 {
        match code {
            8208 => 93,
            8213 => 92,
            8214 => 97,
            8229 => 100,
            8242 => 140,
            8243 => 141,
            8251 => 166,
            8451 => 142,
            8491 => 240,
            8592 => 169,
            8593 => 170,
            8594 => 168,
            8595 => 171,
            8658 => 203,
            8660 => 204,
            8704 => 205,
            8706 => 221,
            8707 => 206,
            8711 => 222,
            8712 => 184,
            8715 => 185,
            8722 => 124,
            8730 => 227,
            8733 => 229,
            8734 => 135,
            8736 => 218,
            8743 => 200,
            8744 => 201,
            8745 => 191,
            8746 => 190,
            8747 => 231,
            8748 => 232,
            8756 => 136,
            8757 => 230,
            8765 => 228,
            8786 => 224,
            8800 => 130,
            8801 => 223,
            8806 => 133,
            8807 => 134,
            8810 => 225,
            8811 => 226,
            8834 => 188,
            8835 => 189,
            8838 => 186,
            8839 => 187,
            8869 => 219,
            8978 => 220,
            9632 => 161,
            9633 => 160,
            9650 => 163,
            9651 => 162,
            9660 => 165,
            9661 => 164,
            9670 => 159,
            9671 => 158,
            9675 => 155,
            9678 => 157,
            9679 => 156,
            9711 => 252,
            9733 => 154,
            9734 => 153,
            9834 => 244,
            9837 => 243,
            9839 => 242,
            12288 => 64,
            12289 => 65,
            12290 => 66,
            12291 => 86,
            12293 => 88,
            12294 => 89,
            12295 => 90,
            12296 => 113,
            12297 => 114,
            12298 => 115,
            12299 => 116,
            12300 => 117,
            12301 => 118,
            12302 => 119,
            12303 => 120,
            12304 => 121,
            12305 => 122,
            12306 => 167,
            12307 => 172,
            12308 => 107,
            12309 => 108,
            12316 => 96,
            12443 => 74,
            12444 => 75,
            12445 => 84,
            12446 => 85,
            12539 => 69,
            12540 => 91,
            12541 => 82,
            12542 => 83,
            20189 => 87,
            65281 => 73,
            65283 => 148,
            65284 => 144,
            65285 => 147,
            65286 => 149,
            65288 => 105,
            65289 => 106,
            65290 => 150,
            65291 => 123,
            65292 => 67,
            65294 => 68,
            65295 => 94,
            65306 => 70,
            65307 => 71,
            65308 => 131,
            65309 => 129,
            65310 => 132,
            65311 => 72,
            65312 => 151,
            65339 => 109,
            65341 => 110,
            65342 => 79,
            65343 => 81,
            65344 => 77,
            65371 => 111,
            65372 => 98,
            65373 => 112,
            65507 => 80,
            65509 => 143,
            _ => 0,
        }
    }
}
