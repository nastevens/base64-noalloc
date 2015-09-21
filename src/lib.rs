// The MIT License (MIT)
//
// Copyright (c) 2015 Nick Stevens <nick@bitcurry.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#![crate_type = "lib"]
#![crate_name = "base64"]

#![allow(unused_features)]
#![feature(no_std, core_slice_ext, convert)]
#![no_std]

//! Heapless Base64 binary encoder and decoder, using only libcore
//!
//! Implements a Base64 encoder and decoder that operate using only the stack.
//! To reduce total memory usage, the implementation does not use a separate
//! encode/decode buffer. Instead, the resulting encoded/decoded value is read
//! from an iterator.
//!
//! # Examples
//!
//! Typical usage looks like the following. This utilizes the "fixedvec"
//! library, which allows for easy draining of iterators into fixed-size
//! buffers.
//!
//! ```
//! extern crate base64;
//! #[macro_use] extern crate fixedvec;
//!
//! use base64::{Base64Encoder, Base64Decoder};
//! use fixedvec::FixedVec;
//!
//! fn main() {
//!     // Allocate FixedVec for result
//!     let mut _backing_data = alloc_stack!([u8; 20]);
//!     let mut result = FixedVec::new(&mut _backing_data);
//!
//!     // Create encoder
//!     let test_data = b"foobar";
//!     let mut encoder = Base64Encoder::new(&b"foobar"[..]);
//!
//!     // Sink all data from encoder into result vector
//!     result.extend(encoder);
//!
//!     assert_eq!(result.as_slice(), &b"Zm9vYmFy"[..]);
//!
//!     // Create decoder
//!     let mut decoder = Base64Decoder::new(&b"Zm9vYmFy"[..]);
//!
//!     // Sink all data from decoder into result vector
//!     result.clear();
//!     result.extend(decoder);
//!
//!     assert_eq!(result.as_slice(), &b"foobar"[..]);
//! }
//! ```

#[cfg(test)]
#[macro_use] extern crate std;

#[cfg(test)]
extern crate rand;

use core::slice::Chunks;

pub type Base64Result = Result<(), ()>;

pub struct Base64Encoder<'a> {
    input: Chunks<'a, u8>,
    output: EncodeIterator,
}

// Small iterator to for an encoded "chunk" of 3 bytes -> 4 chars
struct EncodeIterator {
    buffer: [u8; 4],
    idx: usize,
}

impl <'a> Base64Encoder<'a> {

    /// Create a new Base64Encoder from the provided slice.
    ///
    /// # Example
    /// ```
    /// use base64::Base64Encoder;
    ///
    /// let buffer = [0u8, 1, 2, 3, 4, 5];
    /// let encoder = Base64Encoder::new(&buffer);
    /// ```
    pub fn new(input: &'a [u8]) -> Base64Encoder {
        Base64Encoder {
            input: input.chunks(3),
            output: EncodeIterator {
                buffer: [0; 4],
                idx: 4,
            }
        }
    }
}

impl <'a> Iterator for Base64Encoder<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if let Some(n) = self.output.next() {
            Some(n)
        } else {
            if let Some(chunk) = self.input.next() {
                encode_chunk(chunk, &mut self.output);
                self.output.next()
            } else {
                None
            }
        }
    }
}

impl Iterator for EncodeIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.idx < self.buffer.len() {
            self.idx += 1;
            Some(self.buffer[self.idx - 1])
        } else {
            None
        }
    }
}

fn encode_chunk(chunk: &[u8], output: &mut EncodeIterator) {
    let combined: u32 = combine_bytes(chunk);
    for (i, shift) in [18, 12, 6, 0].iter().enumerate() {
        let u: u8 = ((combined >> *shift) as u8) & 0b0011_1111;
        output.buffer[i] = match u {
             0...25 => b'A' + u,
            26...51 => b'a' + u - 26,
            52...61 => b'0' + u - 52,
                 62 => b'+',
                 63 => b'/',
                  _ => unreachable!()
        };
    }
    if chunk.len() <= 1 { output.buffer[2] = b'='; }
    if chunk.len() <= 2 { output.buffer[3] = b'='; }
    output.idx = 0;
}

// Combines up to 3 bytes into a u32.
fn combine_bytes(bytes: &[u8]) -> u32 {
    0 | if bytes.len() >= 1 {
        ((bytes[0] as u32) << 16)
    } else {
        0
    } | if bytes.len() >= 2 {
        ((bytes[1] as u32) << 8 )
    } else {
        0
    } | if bytes.len() >= 3 {
        ((bytes[2] as u32) << 0 )
    } else {
        0
    }
}

pub struct Base64Decoder<'a> {
    input: Chunks<'a, u8>,
    output: DecodeIterator,
    status: Base64Result,
}

struct DecodeIterator {
    buffer: [u8; 3],  // 4 characters generate 3 bytes
    idx: usize,
    len: usize,
}

impl <'a> Base64Decoder<'a> {

    /// Create a new Base64Decoder from the provided slice.
    ///
    /// # Example
    /// ```
    /// use base64::Base64Decoder;
    ///
    /// let buffer = b"Zm9vYmFy";
    /// let decoder = Base64Decoder::new(buffer);
    /// ```
    pub fn new(input: &'a [u8]) -> Base64Decoder {
        Base64Decoder {
            input: input.chunks(4),
            output: DecodeIterator {
                buffer: [0; 3],
                idx: 3,
                len: 0,
            },
            status: Ok(()),
        }
    }

    /// Check the decoder status for errors.
    ///
    /// Because results are returned as an iterator, and iterators do not
    /// provide a built-in mechanism for returning errors, it is necessary to
    /// check the status of the decoder after decoding.
    ///
    /// # Example
    /// ```
    /// use base64::Base64Decoder;
    ///
    /// let buffer = b"Zm9vYmFy==="; // Contains bad padding
    /// let mut decoder = Base64Decoder::new(buffer);
    /// while let Some(_) = decoder.next() { }
    /// assert!(decoder.status().is_err());
    /// ```
    pub fn status(&self) -> Base64Result {
        Clone::clone(&self.status)
    }
}

impl <'a> Iterator for Base64Decoder<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if let Some(n) = self.output.next() {
            Some(n)
        } else {
            if let Some(chunk) = self.input.next() {
                self.status = decode_chunk(chunk, &mut self.output);
                if self.status.is_ok() {
                    self.output.next()
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

impl Iterator for DecodeIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.idx < self.len {
            self.idx += 1;
            Some(self.buffer[self.idx - 1])
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Decoded {
    Value(u8),
    Padding,
    Invalid,
}

fn decode_chunk(chunk: &[u8], output: &mut DecodeIterator) -> Base64Result {


    use Decoded::*;

    let mut tmp: [Decoded; 4] = [Decoded::Invalid; 4];

    for (i, value) in chunk.iter().enumerate() {
        tmp[i] = match *value {
            b'A'...b'Z' => Value(value - 0x41),
            b'a'...b'z' => Value(value - 0x47),
            b'0'...b'9' => Value(value + 0x04),
            b'+' | b'-' => Value(0x3E),
            b'/' | b'_' => Value(0x3F),
                   b'=' => Padding,
                      _ => Invalid
        };
    }

    // There should always be chunks of 4 characters
    if tmp.iter().any(|x| match *x { Invalid => true, _ => false }) {
        return Err(());
    }

    // Only positions 2 and 3 can be padding
    if tmp[0] == Padding || tmp[1] == Padding {
        return Err(());
    }

    // Position 2 can only be padding if position 3 is padding
    if tmp[2] == Padding && tmp[3] != Padding {
        return Err(());
    }

    if let (Value(a), Value(b)) = (tmp[0], tmp[1]) {
        output.buffer[0] = (a << 2) | ((b >> 4) & 0b0000_0011);
        output.len = 1;
    }

    if let (Value(a), Value(b)) = (tmp[1], tmp[2]) {
        output.buffer[1] = (a << 4) | ((b >> 2) & 0b0000_1111);
        output.len = 2;
    }

    if let (Value(a), Value(b)) = (tmp[2], tmp[3]) {
        output.buffer[2] = (a << 6) | ((b >> 0) & 0b0011_1111);
        output.len = 3;
    }

    output.idx = 0;

    Ok(())
}


#[cfg(test)]
mod test {

    use super::*;
    use std::prelude::v1::*;

    #[test]
    fn test_encoder_basic() {
        let test_wrapper = |s: &str| -> String {
            let encoder = Base64Encoder::new(s.as_bytes());
            String::from_utf8(encoder.collect()).unwrap()
        };
        assert_eq!(test_wrapper(""), "");
        assert_eq!(test_wrapper("f"), "Zg==");
        assert_eq!(test_wrapper("fo"), "Zm8=");
        assert_eq!(test_wrapper("foo"), "Zm9v");
        assert_eq!(test_wrapper("foob"), "Zm9vYg==");
        assert_eq!(test_wrapper("fooba"), "Zm9vYmE=");
        assert_eq!(test_wrapper("foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_decoder_basic() {
        let test_wrapper = |s: &str| -> Vec<u8> {
            let decoder = Base64Decoder::new(s.as_bytes());
            decoder.collect()
        };
        assert_eq!(test_wrapper(""), b"");
        assert_eq!(test_wrapper("Zg=="), b"f");
        assert_eq!(test_wrapper("Zm8="), b"fo");
        assert_eq!(test_wrapper("Zm9v"), b"foo");
        assert_eq!(test_wrapper("Zm9vYg=="), b"foob");
        assert_eq!(test_wrapper("Zm9vYmE="), b"fooba");
        assert_eq!(test_wrapper("Zm9vYmFy"), b"foobar");
    }

    #[test]
    fn test_decoder_invalid() {
        let test_wrapper = |s: &str| {
            let mut decoder = Base64Decoder::new(s.as_bytes());
            while let Some(_) = decoder.next() { }
            assert!(decoder.status().is_err());
        };

        test_wrapper("Zm$=");
        test_wrapper("Zg==$");
        test_wrapper("Z===");
    }

    #[test]
    fn test_base64_random() {
        use rand::{thread_rng, Rng};

        let encode_wrapper = |s: &[u8]| -> String {
            let encoder = Base64Encoder::new(s);
            String::from_utf8(encoder.collect()).unwrap()
        };

        let decode_wrapper = |s: &str| -> Vec<u8> {
            let decoder = Base64Decoder::new(s.as_bytes());
            decoder.collect()
        };

        for _ in 0..1000 {
            let times = thread_rng().gen_range(1, 100);
            let v = thread_rng().gen_iter::<u8>().take(times)
                                .collect::<Vec<_>>();
            let encoded = encode_wrapper(v.as_slice());
            let decoded = decode_wrapper(encoded.as_str());
            assert_eq!(decoded, v);
        }
    }
}
