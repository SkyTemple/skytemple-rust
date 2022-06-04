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

//! Fork from encoding @ 0.2 util.rs

#[cfg(feature = "strings")]
use encoding::types;
#[cfg(feature = "strings")]
use std::marker::PhantomData;
use std::str::Chars;

/// External iterator for a string's characters with its corresponding byte offset range.
pub struct StrCharIndexIterator<'r> {
    index: usize,
    chars: Chars<'r>,
}

impl<'r> Iterator for StrCharIndexIterator<'r> {
    type Item = ((usize, usize), char);

    #[inline]
    fn next(&mut self) -> Option<((usize, usize), char)> {
        if let Some(ch) = self.chars.next() {
            let prev = self.index;
            let next = prev + ch.len_utf8();
            self.index = next;
            Some(((prev, next), ch))
        } else {
            None
        }
    }
}

/// A trait providing an `index_iter` method.
pub trait StrCharIndex<'r> {
    fn index_iter(&self) -> StrCharIndexIterator<'r>;
}

impl<'r> StrCharIndex<'r> for &'r str {
    /// Iterates over each character with corresponding byte offset range.
    fn index_iter(&self) -> StrCharIndexIterator<'r> {
        StrCharIndexIterator {
            index: 0,
            chars: self.chars(),
        }
    }
}

#[cfg(feature = "strings")]
/// A helper struct for the stateful decoder DSL.
pub struct StatefulDecoderHelper<'a, St, Data: 'a> {
    /// The current buffer.
    pub buf: &'a [u8],
    /// The current index to the buffer.
    pub pos: usize,
    /// The output buffer.
    pub output: &'a mut (dyn types::StringWriter + 'a),
    /// The last codec error. The caller will later collect this.
    pub err: Option<types::CodecError>,
    /// The additional data attached for the use from transition functions.
    pub data: &'a Data,
    /// A marker for the phantom type parameter `St`.
    _marker: PhantomData<St>,
}

#[cfg(feature = "strings")]
impl<'a, St: Default, Data> crate::encoding_utils::StatefulDecoderHelper<'a, St, Data> {
    /// Makes a new decoder context out of given buffer and output callback.
    #[inline(always)]
    pub fn new(
        buf: &'a [u8],
        output: &'a mut (dyn types::StringWriter + 'a),
        data: &'a Data,
    ) -> crate::encoding_utils::StatefulDecoderHelper<'a, St, Data> {
        crate::encoding_utils::StatefulDecoderHelper {
            buf,
            pos: 0,
            output,
            err: None,
            data,
            _marker: PhantomData,
        }
    }

    /// Reads one byte from the buffer if any.
    #[inline(always)]
    pub fn read(&mut self) -> Option<u8> {
        match self.buf.get(self.pos) {
            Some(&c) => {
                self.pos += 1;
                Some(c)
            }
            None => None,
        }
    }

    /// Resets back to the initial state.
    /// This should be the last expr in the rules.
    #[inline(always)]
    pub fn reset(&self) -> St {
        Default::default()
    }

    /// Writes one Unicode scalar value to the output.
    /// There is intentionally no check for `c`, so the caller should ensure that it's valid.
    /// If this is the last expr in the rules, also resets back to the initial state.
    #[inline(always)]
    pub fn emit(&mut self, c: u32) -> St {
        self.output.write_char(std::char::from_u32(c).unwrap());
        Default::default()
    }

    /// Issues a codec error with given message at the current position.
    /// If this is the last expr in the rules, also resets back to the initial state.
    #[inline(always)]
    pub fn err(&mut self, msg: &'static str) -> St {
        self.err = Some(types::CodecError {
            upto: self.pos as isize,
            cause: msg.into(),
        });
        Default::default()
    }

    /// Issues a codec error with given message at the current position minus `backup` bytes.
    /// If this is the last expr in the rules, also resets back to the initial state.
    ///
    /// This should be used to implement "prepending byte to the stream" in the Encoding spec,
    /// which corresponds to `ctx.backup_and_err(1, ...)`.
    #[inline(always)]
    pub fn backup_and_err(&mut self, backup: usize, msg: &'static str) -> St {
        let upto = self.pos as isize - backup as isize;
        self.err = Some(types::CodecError {
            upto,
            cause: msg.into(),
        });
        Default::default()
    }
}

#[cfg(feature = "strings")]
/// Defines a stateful decoder from given state machine.
macro_rules! stateful_decoder {
    (
        module $stmod:ident; // should be unique from other existing identifiers
        $(internal $item:item)* // will only be visible from state functions
    initial:
        state $inist:ident($inictx:ident: Context) {
            $(case $($inilhs:pat),+ => $($inirhs:expr),+;)+
            final => $($inifin:expr),+;
        }
    checkpoint:
        $(state $ckst:ident($ckctx:ident: Context $(, $ckarg:ident: $ckty:ty)*) {
            $(case $($cklhs:pat),+ => $($ckrhs:expr),+;)+
            final => $($ckfin:expr),+;
        })*
    transient:
        $(state $st:ident($ctx:ident: Context $(, $arg:ident: $ty:ty)*) {
            $(case $($lhs:pat),+ => $($rhs:expr),+;)+
            final => $($fin:expr),+;
        })*
    ) => (
        #[allow(non_snake_case)]
        mod $stmod {
            pub use self::State::*;

            #[derive(PartialEq, Eq, Clone, Copy)]
            pub enum State {
                $inist,
                $(
                    $ckst(() $(, $ckty)*),
                )*
                $(
                    $st(() $(, $ty)*),
                )*
            }

            impl ::std::default::Default for State {
                #[inline(always)] fn default() -> State { $inist }
            }

            pub mod internal {
                pub type Context<'a, Data> = crate::encoding_utils::StatefulDecoderHelper<'a, super::State, Data>;

                $($item)*
            }

            pub mod start {
                use super::internal::*;

                #[inline(always)]
                pub fn $inist<T>($inictx: &mut Context<T>) -> super::State {
                    // prohibits all kind of recursions, including self-recursions
                    #[allow(unused_imports)] use super::transient::*;
                    match $inictx.read() {
                        None => super::$inist,
                        Some(c) => match c { $($($inilhs)|+ => { $($inirhs);+ })+ },
                    }
                }

                $(
                    #[inline(always)]
                    pub fn $ckst<T>($ckctx: &mut Context<T> $(, $ckarg: $ckty)*) -> super::State {
                        // prohibits all kind of recursions, including self-recursions
                        #[allow(unused_imports)] use super::transient::*;
                        match $ckctx.read() {
                            None => super::$ckst(() $(, $ckarg)*),
                            Some(c) => match c { $($($cklhs)|+ => { $($ckrhs);+ })+ },
                        }
                    }
                )*
            }

            pub mod transient {
                use super::internal::*;

                #[inline(always)]
                #[allow(dead_code)]
                pub fn $inist<T>(_: &mut Context<T>) -> super::State {
                    super::$inist // do not recurse further
                }

                $(
                    #[inline(always)]
                    #[allow(dead_code)]
                    pub fn $ckst<T>(_: &mut Context<T> $(, $ckarg: $ckty)*) -> super::State {
                        super::$ckst(() $(, $ckarg)*) // do not recurse further
                    }
                )*

                $(
                    #[inline(always)]
                    pub fn $st<T>($ctx: &mut Context<T> $(, $arg: $ty)*) -> super::State {
                        match $inictx.read() {
                            None => super::$st(() $(, $arg)*),
                            Some(c) => match c { $($($lhs)|+ => { $($rhs);+ })+ },
                        }
                    }
                )*
            }

            pub fn raw_feed<T>(mut st: State, input: &[u8], output: &mut dyn encoding::types::StringWriter,
                               data: &T) -> (State, usize, Option<encoding::types::CodecError>) {
                output.writer_hint(input.len());

                let mut ctx = crate::encoding_utils::StatefulDecoderHelper::new(input, output, data);
                let mut processed = 0;

                let st_ = match st {
                    $inist => $inist,
                    $(
                        $ckst(() $(, $ckarg)*) => start::$ckst(&mut ctx $(, $ckarg)*),
                    )*
                    $(
                        $st(() $(, $arg)*) => transient::$st(&mut ctx $(, $arg)*),
                    )*
                };
                match (ctx.err.take(), st_) {
                    (None, $inist) $(| (None, $ckst(..)))* => { st = st_; processed = ctx.pos; }
                    // XXX splitting the match case improves the performance somehow, but why?
                    (None, _) => { return (st_, processed, None); }
                    (Some(err), _) => { return (st_, processed, Some(err)); }
                }

                while ctx.pos < ctx.buf.len() {
                    let st_ = match st {
                        $inist => start::$inist(&mut ctx),
                        $(
                            $ckst(() $(, $ckarg)*) => start::$ckst(&mut ctx $(, $ckarg)*),
                        )*
                        _ => unreachable!(),
                    };
                    match (ctx.err.take(), st_) {
                        (None, $inist) $(| (None, $ckst(..)))* => { st = st_; processed = ctx.pos; }
                        // XXX splitting the match case improves the performance somehow, but why?
                        (None, _) => { return (st_, processed, None); }
                        (Some(err), _) => { return (st_, processed, Some(err)); }
                    }
                }

                (st, processed, None)
            }

            pub fn raw_finish<T>(mut st: State, output: &mut dyn encoding::types::StringWriter,
                                 data: &T) -> (State, Option<encoding::types::CodecError>) {
                #![allow(unused_mut, unused_variables)]
                let mut ctx = crate::encoding_utils::StatefulDecoderHelper::new(&[], output, data);
                let st = match ::std::mem::replace(&mut st, $inist) {
                    $inist => { let $inictx = &mut ctx; $($inifin);+ },
                    $(
                        $ckst(() $(, $ckarg)*) => { let $ckctx = &mut ctx; $($ckfin);+ },
                    )*
                    $(
                        $st(() $(, $arg)*) => { let $ctx = &mut ctx; $($fin);+ },
                    )*
                };
                (st, ctx.err.take())
            }
        }
    );

    // simplified rules: no checkpoint and default final actions
    (
        module $stmod:ident; // should be unique from other existing identifiers
        $(internal $item:item)* // will only be visible from state functions
    initial:
        state $inist:ident($inictx:ident: Context) {
            $(case $($inilhs:pat),+ => $($inirhs:expr),+;)+
        }
    transient:
        $(state $st:ident($ctx:ident: Context $(, $arg:ident: $ty:ty)*) {
            $(case $($lhs:pat),+ => $($rhs:expr),+;)+
        })*
    ) => (
        stateful_decoder! {
            module $stmod;
            $(internal $item)*
        initial:
            state $inist($inictx: Context) {
                $(case $($inilhs),+ => $($inirhs),+;)+
                final => $inictx.reset();
            }
        checkpoint:
        transient:
            $(state $st($ctx: Context $(, $arg: $ty)*) {
                $(case $($lhs),+ => $($rhs),+;)+
                final => $ctx.err("incomplete sequence");
            })*
        }
    );
}
