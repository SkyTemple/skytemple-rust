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

use encoding::{ByteWriter, CodecError, Encoding, RawDecoder, RawEncoder, StringWriter};

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
    pub fn new() -> Box<dyn RawEncoder> {
        Box::new(Pmd2Encoder)
    }
}

impl RawEncoder for Pmd2Encoder {
    fn from_self(&self) -> Box<dyn RawEncoder> {
        todo!()
    }

    fn is_ascii_compatible(&self) -> bool {
        true
    }

    fn raw_feed(&mut self, input: &str, output: &mut dyn ByteWriter) -> (usize, Option<CodecError>) {
        todo!()
    }

    fn raw_finish(&mut self, output: &mut dyn ByteWriter) -> Option<CodecError> {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct Pmd2Decoder;

impl Pmd2Decoder {
    pub fn new() -> Box<dyn RawDecoder> {
        Box::new(Pmd2Decoder)
    }
}

impl RawDecoder for Pmd2Decoder {
    fn from_self(&self) -> Box<dyn RawDecoder> {
        todo!()
    }

    fn is_ascii_compatible(&self) -> bool {
        true
    }

    fn raw_feed(&mut self, input: &[u8], output: &mut dyn StringWriter) -> (usize, Option<CodecError>) {
        todo!()
    }

    fn raw_finish(&mut self, output: &mut dyn StringWriter) -> Option<CodecError> {
        todo!()
    }
}
