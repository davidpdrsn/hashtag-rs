# Hashtag

A parser for finding hashtags in strings. For example parsing `Rust is #awesome` gives you `awesome` and its location within the string.

## Sample usage

```rust
extern crate hashtag;

use hashtag::Hashtag;

pub fn main() {
    let tags: Vec<Hashtag> = Hashtag::parse("#rust is #awesome");
    println!("{:?}", tags);
    // => [Hashtag { text: "rust", start: 0, end: 4 }, Hashtag { text: "awesome", start: 9, end: 16 }]
}
```
