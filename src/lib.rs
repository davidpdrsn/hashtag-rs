//! The goal of this crate is to match Instagram's parsing of hashtags. So if you find strings that
//! aren't parsed correctly please open an issue 😃
//!
//! ## Sample usage
//!
//! ```
//! # extern crate hashtag;
//! # use hashtag::Hashtag;
//! # fn main() {
//! let tags: Vec<Hashtag> = Hashtag::parse("#rust is #awesome");
//!
//! assert_eq!(
//!     tags,
//!     [
//!         Hashtag {
//!             text: "rust".to_string(),
//!             start: 0,
//!             end: 4,
//!         },
//!         Hashtag {
//!             text: "awesome".to_string(),
//!             start: 9,
//!             end: 16,
//!         },
//!     ]
//! );
//! # }
//! ```
//!
//! See tests for specifics about what is considered a hashtag and what is not.

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

/// A hashtag found in some text. See documentation of top level module for more info.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Hashtag {
    /// The text of the hashtag. If hashtag is `"#rust"` the text will be `"rust"`.
    ///
    /// ```
    /// # extern crate hashtag;
    /// # use hashtag::Hashtag;
    /// # fn main() {
    /// assert_eq!(
    ///     Hashtag::parse("#rust").get(0).unwrap().text,
    ///     "rust".to_string()
    /// );
    /// # }
    /// ```
    pub text: String,

    /// The starting index of the hashtag. This includes the `#` character. This makes it easier to
    /// highlight the hashtags later. If the full text we're parsing is `"#rust"` then `start` will
    /// be 0.
    ///
    /// ```
    /// # extern crate hashtag;
    /// # use hashtag::Hashtag;
    /// # fn main() {
    /// assert_eq!(
    ///     Hashtag::parse("#rust").get(0).unwrap().start,
    ///     0
    /// );
    /// # }
    /// ```
    pub start: usize,

    /// The ending index of the hashtag, inclusive. If the full text we're parsing is `"#rust"` then `end`
    /// will be 4.
    ///
    /// ```
    /// # extern crate hashtag;
    /// # use hashtag::Hashtag;
    /// # fn main() {
    /// assert_eq!(
    ///     Hashtag::parse("#rust").get(0).unwrap().end,
    ///     4
    /// );
    /// # }
    /// ```
    pub end: usize,
}

impl Hashtag {
    /// Parse a string and return a vector of the hashtags.
    #[inline]
    pub fn parse(text: &str) -> Vec<Self> {
        parse_hashtags(text)
    }

    #[inline]
    fn new(text: &str, start: usize, end: usize) -> Hashtag {
        Hashtag {
            text: text.to_string(),
            start: start,
            end: end,
        }
    }

    /// Convert a `Hashtag` into JSON using [serde_json](https://crates.io/crates/serde_json).
    ///
    /// At Tonsser we use this crate from our Rails API with [helix](https://usehelix.com) and
    /// because helix only supports passing strings back and forth we serialize the data as JSON
    /// and deserialize it in Ruby land.
    #[inline]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

struct ParsingStateMachine {
    consumed_anything: bool,
    hashtag_buffer: String,
    hashtag_start_index: usize,
    hashtags: Vec<Hashtag>,
    parsing_hashtag: bool,
}

impl ParsingStateMachine {
    #[inline]
    fn new() -> ParsingStateMachine {
        ParsingStateMachine {
            parsing_hashtag: Self::default_parse_hashtag(),
            hashtag_start_index: Self::default_hashtag_start_index(),
            hashtag_buffer: Self::default_hashtag_buffer(),
            hashtags: Self::default_hashtags(),
            consumed_anything: Self::default_consumed_anything(),
        }
    }

    #[inline]
    fn default_parse_hashtag() -> bool {
        false
    }

    #[inline]
    fn default_consumed_anything() -> bool {
        false
    }

    #[inline]
    fn default_hashtag_start_index() -> usize {
        0
    }

    #[inline]
    fn default_hashtag_buffer() -> String {
        String::new()
    }

    #[inline]
    fn default_hashtags() -> Vec<Hashtag> {
        Vec::new()
    }

    #[inline]
    fn parsing_hashtag(&self) -> bool {
        self.parsing_hashtag
    }

    #[inline]
    fn hashtag_token_seen_at(&mut self, idx: usize) {
        self.hashtag_start_index = idx;
    }

    #[inline]
    fn hashtag_finishes_at(&mut self, idx: usize) {
        if self.consumed_anything {
            self.hashtags.push(Hashtag::new(
                &self.hashtag_buffer,
                self.hashtag_start_index,
                idx,
            ));
        }
        self.reset_parsing_state();
    }

    #[inline]
    fn reset_parsing_state(&mut self) {
        self.parsing_hashtag = Self::default_parse_hashtag();
        self.hashtag_start_index = Self::default_hashtag_start_index();
        self.hashtag_buffer = Self::default_hashtag_buffer();
        self.consumed_anything = Self::default_consumed_anything();
    }

    #[inline]
    fn hashtag_incoming(&mut self) {
        self.parsing_hashtag = true;
    }

    #[inline]
    fn consume_char(&mut self, c: char) {
        self.hashtag_buffer.push(c);
        self.consumed_anything = true;
    }

    #[inline]
    fn get_hashtags(self) -> Vec<Hashtag> {
        self.hashtags
    }
}

fn parse_hashtags(text: &str) -> Vec<Hashtag> {
    let text: String = text.into();
    let tokens = tokenize(&text);
    let mut tokens_iter = tokens.iter().peekable();

    let mut stm = ParsingStateMachine::new();

    loop {
        if let Some(token) = tokens_iter.next() {
            match token {
                &Token::Hashtag(idx) => {
                    if stm.parsing_hashtag() {
                        if tokens_iter.peek().is_end_of_hashtag() {
                            stm.reset_parsing_state();
                        } else {
                            stm.hashtag_token_seen_at(idx);
                        }
                    }
                }

                &Token::Whitespace(idx) => {
                    if stm.parsing_hashtag() {
                        stm.hashtag_finishes_at(idx - 1);
                    }

                    if tokens_iter.peek().is_hashtag_token() {
                        stm.hashtag_incoming();
                    }
                }

                &Token::Char(c, idx) => {
                    if stm.parsing_hashtag() {
                        if c.is_end_of_hashtag() {
                            stm.hashtag_finishes_at(idx - 1);
                        } else {
                            stm.consume_char(c);
                        }

                        if tokens_iter.peek().is_hashtag_token() {
                            stm.hashtag_finishes_at(idx);
                            stm.hashtag_incoming();
                        }
                    }
                }

                &Token::StartOfString => {
                    if tokens_iter.peek().is_hashtag_token() {
                        stm.hashtag_incoming();
                    }
                }

                &Token::EndOfString(idx) => {
                    if stm.parsing_hashtag() {
                        stm.hashtag_finishes_at(idx - 1);
                    }
                }
            }
        } else {
            break;
        }
    }
    let hashtags = stm.get_hashtags();
    hashtags
}

#[derive(Eq, PartialEq, Debug)]
enum Token {
    Char(char, usize),
    Whitespace(usize),
    Hashtag(usize),
    StartOfString,
    EndOfString(usize),
}

trait IsHashtagToken {
    #[inline]
    fn is_hashtag_token(&self) -> bool;
}

impl IsHashtagToken for Token {
    #[inline]
    fn is_hashtag_token(&self) -> bool {
        match self {
            &Token::Hashtag(_) => true,
            &Token::Char(_, _) => false,
            &Token::Whitespace(_) => false,
            &Token::StartOfString => false,
            &Token::EndOfString(_) => false,
        }
    }
}

impl<'a, 'b, T> IsHashtagToken for Option<&'a &'b T>
where
    T: IsHashtagToken,
{
    #[inline]
    fn is_hashtag_token(&self) -> bool {
        if let &Some(ref x) = self {
            x.is_hashtag_token()
        } else {
            false
        }
    }
}

#[inline]
fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![Token::StartOfString];
    text.chars()
        .enumerate()
        .map(|(idx, c)| match c {
            '#' => Token::Hashtag(idx),
            ' ' => Token::Whitespace(idx),
            '\n' => Token::Whitespace(idx),
            '\r' => Token::Whitespace(idx),
            '\t' => Token::Whitespace(idx),
            _ => Token::Char(c, idx),
        })
        .for_each(|token| tokens.push(token));
    let last_index = tokens.len() - 1;
    tokens.push(Token::EndOfString(last_index));
    tokens
}

trait IsEndOfHashtag {
    #[inline]
    fn is_end_of_hashtag(&self) -> bool;
}

impl IsEndOfHashtag for char {
    #[inline]
    fn is_end_of_hashtag(&self) -> bool {
        match self {
            &'\'' | &' ' | &'%' | &'#' | &'\n' | &'"' | &'\t' | &'!' | &'@' | &'$' | &'^' |
            &'&' | &'*' | &'(' | &')' | &'\r' | &'.' | &',' | &'-' | &'<' | &'>' | &'/' |
            &'\\' | &'|' | &'[' | &']' | &'{' | &'}' | &'`' | &'~' | &'=' | &'+' | &';' |
            &'?' | &'£' | &'•' | &'´' | &':' => true,
            &'_' => false,
            _ => false,
        }
    }
}

impl IsEndOfHashtag for Token {
    #[inline]
    fn is_end_of_hashtag(&self) -> bool {
        match self {
            &Token::Whitespace(_) => true,
            &Token::Char(c, _) => c.is_end_of_hashtag(),
            &Token::EndOfString(_) => true,
            &Token::Hashtag(_) => false,
            &Token::StartOfString => false,
        }
    }
}

impl<'a, 'b, T> IsEndOfHashtag for Option<&'a &'b T>
where
    T: IsEndOfHashtag,
{
    #[inline]
    fn is_end_of_hashtag(&self) -> bool {
        if let &Some(ref x) = self {
            x.is_end_of_hashtag()
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let tokens: Vec<Token> = tokenize("text #foo");
        assert_eq!(
            tokens,
            vec![
                Token::StartOfString,
                Token::Char('t', 0),
                Token::Char('e', 1),
                Token::Char('x', 2),
                Token::Char('t', 3),
                Token::Whitespace(4),
                Token::Hashtag(5),
                Token::Char('f', 6),
                Token::Char('o', 7),
                Token::Char('o', 8),
                Token::EndOfString(9),
            ]
        );
    }

    #[test]
    fn test_tokenize_strings_with_emojis() {
        assert_eq!(
            tokenize("😀"),
            vec![
                Token::StartOfString,
                Token::Char('😀', 0),
                Token::EndOfString(1),
            ]
        );
    }
}
