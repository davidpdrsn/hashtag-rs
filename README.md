[![Crates.io](https://img.shields.io/crates/v/hashtag.svg)](https://crates.io/crates/hashtag)
[![Docs](https://docs.rs/hashtag/badge.svg)](https://docs.rs/hashtag)
![maintenance-status](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)

# Hashtag

A parser for finding hashtags in strings. For example parsing `Rust is #awesome` gives you `awesome` and its location within the string.

The goal of this crate is to match Instagram's parsing of hashtags. So if you find strings that aren't parsed correctly please open an issue 😃

## Sample usage

```rust
use hashtag::{Hashtag, HashtagParser};
use std::borrow::Cow;

let mut parser = HashtagParser::new("#rust is #awesome");

assert_eq!(
    parser.next().unwrap(),
    Hashtag {
        text: Cow::from("rust"),
        start: 0,
        end: 4,
    }
);

assert_eq!(
    parser.next().unwrap(),
    Hashtag {
        text: Cow::from("awesome"),
        start: 9,
        end: 16,
    }
);

assert_eq!(parser.next(), None);
```

## Benchmarking

I have written a fairly simple benchmarking.

Run it with:

    cargo build --release && ./target/release/benchmark

## Contribution

Contributions are more than welcome!

## License

`hashtag` is available under the MIT license. See the LICENSE file for more info.
