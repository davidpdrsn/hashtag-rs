//! The goal of this crate is to match Instagram's parsing of hashtags. So if you find strings that
//! aren't parsed correctly please open an issue 😃
//!
//! ## Example
//!
//! ```
//! use hashtag::{Hashtag, HashtagParser};
//! use std::borrow::Cow;
//!
//! let mut parser = HashtagParser::new("#rust is #awesome");
//!
//! assert_eq!(
//!     parser.next().unwrap(),
//!     Hashtag {
//!         text: Cow::from("rust"),
//!         start: 0,
//!         end: 4,
//!     }
//! );
//!
//! assert_eq!(
//!     parser.next().unwrap(),
//!     Hashtag {
//!         text: Cow::from("awesome"),
//!         start: 9,
//!         end: 16,
//!     }
//! );
//!
//! assert_eq!(parser.next(), None);
//! ```
//!
//! See tests for specifics about what is considered a hashtag and what is not.
//!
//! # Features
//!
//! - `serde`: Enable `#[derive(Serialize)]` for [`Hashtag`].

#![deny(
    missing_docs,
    unused_imports,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unknown_lints
)]
#![doc(html_root_url = "https://docs.rs/hashtag/1.0.0")]

use std::{
    borrow::Cow,
    fmt,
    iter::{once, Chain},
    iter::{Enumerate, Peekable},
};

/// A hashtag found in some text. See documentation of top level module for more info.
#[derive(Eq, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Hashtag<'a> {
    /// The text of the hashtag.
    ///
    /// [`Cow`] is used to allow hashtags to be references into the original string if it contains
    /// only ascii.
    ///
    /// If hashtag is `"#rust"` the text will be `"rust"`.
    pub text: Cow<'a, str>,

    /// The starting index of the hashtag.
    ///
    /// This includes the `#` character. This makes it easier to highlight the hashtags later. If
    /// the full text we're parsing is `"#rust"` then `start` will be 0.
    pub start: usize,

    /// The ending index of the hashtag, inclusive.
    ///
    /// If the full text we're parsing is `"#rust"` then `end` will be 4.
    pub end: usize,
}

impl<'a> fmt::Display for Hashtag<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#")?;
        write!(f, "{}", self.text)?;
        Ok(())
    }
}

impl<'a> AsRef<str> for Hashtag<'a> {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

/// A parser that finds hashtags in a string.
///
/// Implements [`Iterator`] and yields [`Hashtag`]s.
#[derive(Debug)]
pub struct HashtagParser<'a> {
    whole_string: &'a str,
    state: IterState<'a>,
    done: bool,
}

impl<'a> HashtagParser<'a> {
    /// Create a new `HashtagParser` that will parse the given string.
    pub fn new(text: &'a str) -> Self {
        Self {
            whole_string: text,
            done: false,
            state: IterState::Init,
        }
    }
}

#[derive(Debug)]
enum IterState<'a> {
    Init,
    Parsing {
        tokens: Peekable<Enumerate<TokenIter<'a>>>,
        stm: ParsingStateMachine<'a>,
    },
}

impl<'a> Iterator for HashtagParser<'a> {
    type Item = Hashtag<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.done {
                return None;
            }

            match &mut self.state {
                IterState::Init => {
                    let tokens = tokenize(&self.whole_string).enumerate().peekable();
                    let stm = ParsingStateMachine::new(&self.whole_string);
                    self.state = IterState::Parsing { tokens, stm };
                }
                IterState::Parsing { tokens, stm } => {
                    while let Some((i, token)) = tokens.next() {
                        match token {
                            Token::Hashtag => {
                                if stm.parsing_hashtag() {
                                    if tokens.peek().map(|(_, tok)| tok).is_end_of_hashtag() {
                                        stm.reset_parsing_state();
                                    } else {
                                        stm.hashtag_token_seen_at(i - 1);
                                    }
                                }
                            }

                            Token::Whitespace => {
                                let mut hashtag = None;

                                if stm.parsing_hashtag() {
                                    hashtag = stm.hashtag_finishes_at(i - 2);
                                }

                                if tokens.peek().map(|(_, tok)| tok).is_hashtag_token() {
                                    stm.hashtag_incoming();
                                }

                                if let Some(hashtag) = hashtag.take() {
                                    return Some(hashtag);
                                }
                            }

                            Token::Char(c) => {
                                if stm.parsing_hashtag() {
                                    let mut hashtag = None;

                                    if c.is_end_of_hashtag() {
                                        hashtag = stm.hashtag_finishes_at(i - 2);
                                    } else {
                                        stm.consume_char(c);
                                    }

                                    if tokens.peek().map(|(_, tok)| tok).is_hashtag_token() {
                                        hashtag = stm.hashtag_finishes_at(i - 1);
                                        stm.hashtag_incoming();
                                    }

                                    if let Some(hashtag) = hashtag.take() {
                                        return Some(hashtag);
                                    }
                                }
                            }

                            Token::StartOfString => {
                                if tokens.peek().map(|(_, tok)| tok).is_hashtag_token() {
                                    stm.hashtag_incoming();
                                }
                            }

                            Token::EndOfString => {
                                let hashtag = if stm.parsing_hashtag() {
                                    stm.hashtag_finishes_at(i - 2)
                                } else {
                                    None
                                };
                                self.done = true;
                                return hashtag;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'a> std::iter::FusedIterator for HashtagParser<'a> {}

#[derive(Debug)]
struct ParsingStateMachine<'a> {
    consumed_anything: bool,
    hashtag_buffer: String,
    hashtag_start_index: usize,
    parsing_hashtag: bool,
    is_ascii: bool,
    whole_string: &'a str,
}

impl<'a> ParsingStateMachine<'a> {
    #[inline]
    fn new(text: &'a str) -> ParsingStateMachine<'a> {
        ParsingStateMachine {
            parsing_hashtag: Self::default_parse_hashtag(),
            hashtag_start_index: Self::default_hashtag_start_index(),
            hashtag_buffer: String::new(),
            consumed_anything: Self::default_consumed_anything(),
            is_ascii: text.is_ascii(),
            whole_string: text,
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
    fn parsing_hashtag(&self) -> bool {
        self.parsing_hashtag
    }

    #[inline]
    fn hashtag_token_seen_at(&mut self, idx: usize) {
        self.hashtag_start_index = idx;
    }

    #[inline]
    fn hashtag_finishes_at(&mut self, idx: usize) -> Option<Hashtag<'a>> {
        let hashtag = if self.consumed_anything {
            let text = if self.is_ascii {
                Cow::Borrowed(&self.whole_string[self.hashtag_start_index + 1..idx + 1])
            } else {
                Cow::Owned(self.hashtag_buffer.clone())
            };

            Some(Hashtag {
                text,
                start: self.hashtag_start_index,
                end: idx,
            })
        } else {
            None
        };
        self.reset_parsing_state();
        hashtag
    }

    #[inline]
    fn reset_parsing_state(&mut self) {
        self.parsing_hashtag = Self::default_parse_hashtag();
        self.hashtag_start_index = Self::default_hashtag_start_index();
        self.hashtag_buffer.clear();
        self.consumed_anything = Self::default_consumed_anything();
    }

    #[inline]
    fn hashtag_incoming(&mut self) {
        self.parsing_hashtag = true;
    }

    #[inline]
    fn consume_char(&mut self, c: char) {
        if !self.is_ascii {
            self.hashtag_buffer.push(c);
        }
        self.consumed_anything = true;
    }
}
#[derive(Eq, PartialEq, Debug)]
enum Token {
    Char(char),
    Whitespace,
    Hashtag,
    StartOfString,
    EndOfString,
}

trait IsHashtagToken {
    fn is_hashtag_token(&self) -> bool;
}

impl IsHashtagToken for Token {
    #[inline]
    fn is_hashtag_token(&self) -> bool {
        matches!(self, Token::Hashtag)
    }
}

impl<'a, T> IsHashtagToken for Option<&'a T>
where
    T: IsHashtagToken,
{
    #[inline]
    fn is_hashtag_token(&self) -> bool {
        if let Some(x) = self {
            x.is_hashtag_token()
        } else {
            false
        }
    }
}

type SingleToken = std::iter::Once<Token>;
type TokensFromStr<'a> = std::iter::Map<std::str::Chars<'a>, fn(char) -> Token>;
type TokenIter<'a> = Chain<Chain<SingleToken, TokensFromStr<'a>>, SingleToken>;

#[inline]
fn tokenize(text: &str) -> TokenIter<'_> {
    once(Token::StartOfString)
        .chain(text.chars().map(token_from_char as _))
        .chain(once(Token::EndOfString))
}

#[inline]
fn token_from_char(c: char) -> Token {
    match c {
        '#' => Token::Hashtag,
        ' ' => Token::Whitespace,
        '\n' => Token::Whitespace,
        '\r' => Token::Whitespace,
        '\t' => Token::Whitespace,
        _ => Token::Char(c),
    }
}

trait IsEndOfHashtag {
    fn is_end_of_hashtag(&self) -> bool;
}

impl IsEndOfHashtag for char {
    #[inline]
    fn is_end_of_hashtag(&self) -> bool {
        match self {
            &'\'' | &' ' | &'%' | &'#' | &'\n' | &'"' | &'\t' | &'!' | &'@' | &'€' | &'$'
            | &'^' | &'&' | &'*' | &'(' | &')' | &'\r' | &'.' | &',' | &'-' | &'<' | &'>'
            | &'/' | &'\\' | &'|' | &'[' | &']' | &'{' | &'}' | &'`' | &'~' | &'=' | &'+'
            | &';' | &'?' | &'£' | &'•' | &'´' | &':' => true,
            &'_' => false,
            _ => false,
        }
    }
}

impl IsEndOfHashtag for Token {
    #[inline]
    fn is_end_of_hashtag(&self) -> bool {
        match self {
            Token::Whitespace => true,
            Token::Char(c) => c.is_end_of_hashtag(),
            Token::EndOfString => true,
            Token::Hashtag => false,
            Token::StartOfString => false,
        }
    }
}

impl<'a, T> IsEndOfHashtag for Option<&'a T>
where
    T: IsEndOfHashtag,
{
    #[inline]
    fn is_end_of_hashtag(&self) -> bool {
        self.map(|x| x.is_end_of_hashtag()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let tokens: Vec<Token> = tokenize("text #foo").collect();
        assert_eq!(
            tokens,
            vec![
                Token::StartOfString,
                Token::Char('t'),
                Token::Char('e'),
                Token::Char('x'),
                Token::Char('t'),
                Token::Whitespace,
                Token::Hashtag,
                Token::Char('f'),
                Token::Char('o'),
                Token::Char('o'),
                Token::EndOfString,
            ]
        );
    }

    #[test]
    fn test_tokenize_strings_with_emojis() {
        assert_eq!(
            tokenize("😀").collect::<Vec<_>>(),
            vec![Token::StartOfString, Token::Char('😀'), Token::EndOfString,]
        );
    }
}
