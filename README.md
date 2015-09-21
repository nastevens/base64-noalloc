base64-noalloc
==============

[![Build Status] (https://img.shields.io/travis/nastevens/base64-noalloc.svg)](https://travis-ci.org/nastevens/base64-noalloc)

- [API Documentation](http://nastevens.github.io/base64-noalloc/)

`base64-noalloc` is a Rust library providing a heapless base64 encoder and decoder.

Install/Use
-----------

You must be using the nightly Rust release to use `base64-noalloc`, as it
relies on the unstable libcore feature. If you're building for an embedded
system, you're probably on the nightly anyway though.

To use `base64-noalloc`, add the following to your `Cargo.toml`:

```toml
[dependencies.base64]
git = "https://github.com/nastevens/base64-noalloc.git
```

Then add the following to your crate root:

```rust
extern crate base64;
```

Example
-------

This example uses the [fixedvec](https://github.com/nastevens/fixedvec-rs)
library to make result storage easier.

```rust
extern crate base64;
#[macro_use] extern crate fixedvec;

use base64::{Base64Encoder, Base64Decoder};
use fixedvec::FixedVec;

fn main() {
    // Allocate FixedVec for result
    let mut _backing_data = alloc_stack!([u8; 20]);
    let mut result = FixedVec::new(&mut _backing_data);

    // Create encoder
    let test_data = b"foobar";
    let mut encoder = Base64Encoder::new(&b"foobar"[..]);

    // Sink all data from encoder into result vector
    result.extend(encoder);

    assert_eq!(result.as_slice(), &b"Zm9vYmFy"[..]);

    // Create decoder
    let mut decoder = Base64Decoder::new(&b"Zm9vYmFy"[..]);

    // Sink all data from decoder into result vector
    result.clear();
    result.extend(decoder);

    assert_eq!(result.as_slice(), &b"foobar"[..]);
}
```

License
-------

```
Copyright (c) 2015, Nick Stevens <nick@bitcurry.com>

The MIT License (MIT)

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
