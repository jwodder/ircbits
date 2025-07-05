//! This is a derivative of tokio-util's `LinesCodec` (obtained from
//! [`lines_codec.rs`][1]) adjusted as follows:
//!
//! - The encoder appends the line ending CR LF, not LF.
//!
//! - Text decoding is first attempted using UTF-8; if that fails, it falls
//!   back to Latin-1.
//!
//! - Decoder: `max_length` now includes the terminating line ending.
//!
//! [1]: https://github.com/tokio-rs/tokio/blob/a03e0420249d1740668f608a5a16f1fa614be2c7/tokio-util/src/codec/lines_codec.rs

// Copyright (c) 2022 Tokio Contributors
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::util::decode_utf8_latin1;
use bytes::{Buf, BufMut, BytesMut};
use std::{cmp, io};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

/// A simple [`Decoder`] and [`Encoder`] implementation that splits up data into lines.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IrcLinesCodec {
    // Stored index of the next index to examine for a `\n` character.
    // This is used to optimize searching.
    // For example, if `decode` was called with `abc`, it would hold `3`,
    // because that is the next index to examine.
    // The next time `decode` is called with `abcde\n`, the method will
    // only look at `de\n` before returning.
    next_index: usize,

    /// The maximum length for a given line. If `usize::MAX`, lines will be
    /// read until a `\n` character is reached.
    max_length: usize,

    /// Are we currently discarding the remainder of a line which was over
    /// the length limit?
    is_discarding: bool,
}

impl IrcLinesCodec {
    /// Returns an `IrcLinesCodec` for splitting up data into lines.
    ///
    /// # Note
    ///
    /// The returned `IrcLinesCodec` will not have an upper bound on the length
    /// of a buffered line. See the documentation for `new_with_max_length`
    /// for information on why this could be a potential security risk.
    pub fn new() -> IrcLinesCodec {
        IrcLinesCodec {
            next_index: 0,
            max_length: usize::MAX,
            is_discarding: false,
        }
    }

    /// Returns an `IrcLinesCodec` with a maximum line length limit.
    ///
    /// # Note
    ///
    /// Setting a length limit is highly recommended for any `IrcLinesCodec`
    /// which will be exposed to untrusted input. Otherwise, the size of the
    /// buffer that holds the line currently being read is unbounded. An
    /// attacker could exploit this unbounded buffer by sending an unbounded
    /// amount of input without any `\n` characters, causing unbounded memory
    /// consumption.
    pub fn new_with_max_length(max_length: usize) -> Self {
        IrcLinesCodec {
            max_length,
            ..IrcLinesCodec::new()
        }
    }
}

fn without_carriage_return(s: &[u8]) -> &[u8] {
    if s.last() == Some(&b'\r') {
        &s[..s.len() - 1]
    } else {
        s
    }
}

impl Decoder for IrcLinesCodec {
    type Item = String;
    type Error = LinesCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<String>, LinesCodecError> {
        loop {
            // Determine how far into the buffer we'll search for a newline. If
            // there's no max_length set, we'll read to the end of the buffer.
            let read_to = cmp::min(self.max_length, buf.len());
            let newline_offset = buf[self.next_index..read_to]
                .iter()
                .position(|b| *b == b'\n');
            match (self.is_discarding, newline_offset) {
                (true, Some(offset)) => {
                    // If we found a newline, discard up to that offset and
                    // then stop discarding. On the next iteration, we'll try
                    // to read a line normally.
                    buf.advance(offset + self.next_index + 1);
                    self.is_discarding = false;
                    self.next_index = 0;
                }
                (true, None) => {
                    // Otherwise, we didn't find a newline, so we'll discard
                    // everything we read. On the next iteration, we'll continue
                    // discarding up to max_len bytes unless we find a newline.
                    buf.advance(read_to);
                    self.next_index = 0;
                    if buf.is_empty() {
                        return Ok(None);
                    }
                }
                (false, Some(offset)) => {
                    // Found a line!
                    let newline_index = offset + self.next_index;
                    self.next_index = 0;
                    let line = buf.split_to(newline_index + 1);
                    let line = &line[..line.len() - 1];
                    let line = without_carriage_return(line);
                    let line = decode_utf8_latin1(line.to_vec());
                    return Ok(Some(line));
                }
                (false, None) if buf.len() >= self.max_length => {
                    // Reached the maximum length without finding a
                    // newline, return an error and start discarding on the
                    // next call.
                    self.is_discarding = true;
                    return Err(LinesCodecError::MaxLineLengthExceeded);
                }
                (false, None) => {
                    // We didn't find a line or reach the length limit, so the
                    // next call will resume searching at the current offset.
                    self.next_index = read_to;
                    return Ok(None);
                }
            }
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<String>, LinesCodecError> {
        Ok(match self.decode(buf)? {
            Some(frame) => Some(frame),
            None => {
                // No terminating newline - return remaining data, if any
                if buf.is_empty() || buf == &b"\r"[..] {
                    None
                } else {
                    let line = buf.split_to(buf.len());
                    let line = without_carriage_return(&line);
                    let line = decode_utf8_latin1(line.into());
                    self.next_index = 0;
                    Some(line)
                }
            }
        })
    }
}

impl<T> Encoder<T> for IrcLinesCodec
where
    T: AsRef<str>,
{
    type Error = LinesCodecError;

    fn encode(&mut self, line: T, buf: &mut BytesMut) -> Result<(), LinesCodecError> {
        let line = line.as_ref();
        buf.reserve(line.len() + 2);
        buf.put(line.as_bytes());
        buf.put_u8(b'\r');
        buf.put_u8(b'\n');
        Ok(())
    }
}

impl Default for IrcLinesCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// An error occurred while encoding or decoding a line.
#[derive(Debug, Error)]
pub enum LinesCodecError {
    /// The maximum line length was exceeded.
    #[error("max line length exceeded")]
    MaxLineLengthExceeded,
    /// An IO error occurred.
    #[error(transparent)]
    Io(#[from] io::Error),
}
