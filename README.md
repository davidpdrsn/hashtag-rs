# Hashtag

A parser for finding hashtags in strings. For example parsing `Rust is #awesome` gives you `awesome` and its location within the string.

The goal of this crate is to match Instagram's parsing of hashtags. So if you find strings that aren't parsed correctly please open an issue 😃

## Sample usage

```rust
use hashtag::Hashtag;

let tags: Vec<Hashtag> = Hashtag::parse("#rust is #awesome");
println!("{:?}", tags);
// => [
//     Hashtag { text: "rust", start: 0, end: 4 },
//     Hashtag { text: "awesome", start: 9, end: 16 }
// ]
```

## Benchmarking

I have written a fairly simple benchmarking.

Run it with:

    cargo build --release && ./target/release/benchmark

## Contribution

Contributions are more than welcome!

## License

`hashtag` is available under the MIT license. See the LICENSE file for more info.
