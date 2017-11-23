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
#[derive(Eq, PartialEq, Debug, Serialize)]
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
    pub fn parse<S>(text: S) -> Vec<Self>
    where
        S: Into<String>,
    {
        parse_hashtags(text)
    }

    fn new<S>(text: S, start: usize, end: usize) -> Hashtag
    where
        S: Into<String>,
    {
        Hashtag {
            text: text.into(),
            start: start,
            end: end,
        }
    }

    /// Convert a `Hashtag` into JSON using [serde_json](https://crates.io/crates/serde_json).
    ///
    /// At Tonsser we use this crate from our Rails API with [helix](https://usehelix.com) and
    /// because helix only supports passing strings back and forth we serialize the data as JSON
    /// and deserialize it in Ruby land.
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
    fn new() -> ParsingStateMachine {
        ParsingStateMachine {
            parsing_hashtag: Self::default_parse_hashtag(),
            hashtag_start_index: Self::default_hashtag_start_index(),
            hashtag_buffer: Self::default_hashtag_buffer(),
            hashtags: Self::default_hashtags(),
            consumed_anything: Self::default_consumed_anything(),
        }
    }

    fn default_parse_hashtag() -> bool {
        false
    }

    fn default_consumed_anything() -> bool {
        false
    }

    fn default_hashtag_start_index() -> usize {
        0
    }

    fn default_hashtag_buffer() -> String {
        String::new()
    }

    fn default_hashtags() -> Vec<Hashtag> {
        Vec::new()
    }

    fn parsing_hashtag(&self) -> bool {
        self.parsing_hashtag
    }

    fn hashtag_token_seen_at(&mut self, idx: usize) {
        self.hashtag_start_index = idx;
    }

    fn hashtag_finishes_at(&mut self, idx: usize) {
        if self.consumed_anything {
            self.hashtags.push(Hashtag::new(
                self.hashtag_buffer.clone(),
                self.hashtag_start_index,
                idx,
            ));
        }
        self.reset_parsing_state();
    }

    fn reset_parsing_state(&mut self) {
        self.parsing_hashtag = Self::default_parse_hashtag();
        self.hashtag_start_index = Self::default_hashtag_start_index();
        self.hashtag_buffer = Self::default_hashtag_buffer();
        self.consumed_anything = Self::default_consumed_anything();
    }

    fn hashtag_incoming(&mut self) {
        self.parsing_hashtag = true;
    }

    fn consume_char(&mut self, c: char) {
        self.hashtag_buffer.push(c);
        self.consumed_anything = true;
    }

    fn get_hashtags(self) -> Vec<Hashtag> {
        self.hashtags
    }
}

fn parse_hashtags<S>(text: S) -> Vec<Hashtag>
where
    S: Into<String>,
{
    let text: String = text.into();
    let tokens = tokenize(text);
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
    fn is_hashtag_token(&self) -> bool;
}

impl IsHashtagToken for Token {
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
    fn is_hashtag_token(&self) -> bool {
        if let &Some(ref x) = self {
            x.is_hashtag_token()
        } else {
            false
        }
    }
}

fn tokenize<S>(text: S) -> Vec<Token>
where
    S: Into<String>,
{
    let text: String = text.into();
    let mut tokens: Vec<Token> = vec![Token::StartOfString];
    let mut last_index = 0;
    text.chars()
        .enumerate()
        .map(|(idx, c)| {
            last_index = idx;
            match c {
                '#' => Token::Hashtag(idx),
                ' ' => Token::Whitespace(idx),
                '\n' => Token::Whitespace(idx),
                '\r' => Token::Whitespace(idx),
                '\t' => Token::Whitespace(idx),
                _ => Token::Char(c, idx),
            }
        })
        .for_each(|token| tokens.push(token));
    tokens.push(Token::EndOfString(last_index + 1));
    tokens
}

trait IsEndOfHashtag {
    fn is_end_of_hashtag(&self) -> bool;
}

impl IsEndOfHashtag for char {
    fn is_end_of_hashtag(&self) -> bool {
        match self {
            &'\'' | &' ' | &'%' | &'#' | &'\n' | &'"' | &'\t' | &'!' | &'@' | &'$' | &'^' |
            &'&' | &'*' | &'(' | &')' | &'\r' | &'.' | &'-' | &'<' | &'>' | &'/' | &'\\' |
            &'|' | &'[' | &']' | &'{' | &'}' | &'`' | &'~' | &'=' | &'+' => true,
            &'_' => false,
            _ => false,
        }
    }
}

impl IsEndOfHashtag for Token {
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

    #[test]
    fn it_parses_hashtags() {
        assert_parse(
            "Here comes some text #foo #bar",
            vec![Hashtag::new("foo", 21, 24), Hashtag::new("bar", 26, 29)],
        )
    }

    #[test]
    fn it_parses_tags_in_the_middle_of_words() {
        assert_parse("here comes foo#bar", vec![])
    }

    #[test]
    fn it_parses_tags_in_the_start() {
        assert_parse(
            "#foo here comes #foo",
            vec![Hashtag::new("foo", 0, 3), Hashtag::new("foo", 16, 19)],
        )
    }

    #[test]
    fn it_parses_hashes_without_text() {
        assert_parse("here # comes", vec![]);
        assert_parse("here comes#", vec![]);
        assert_parse("#here comes", vec![Hashtag::new("here", 0, 4)]);

        assert_parse("here ## comes", vec![]);
        assert_parse("here comes##", vec![]);
        assert_parse("##here comes", vec![Hashtag::new("here", 1, 5)]);
    }

    #[test]
    fn it_parses_hashtags_with_s() {
        assert_parse(
            "#bob's thing is #cool yes",
            vec![Hashtag::new("bob", 0, 3), Hashtag::new("cool", 16, 20)],
        );
    }

    #[test]
    fn it_parses_many_different_kinds() {

        assert_parse("#a1", vec![Hashtag::new("a1", 0, 2)]);
        assert_parse("#a_1", vec![Hashtag::new("a_1", 0, 3)]);
        assert_parse("#a-1", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a.1", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#😀", vec![Hashtag::new("😀", 0, 1)]);
        assert_parse(" #whá", vec![Hashtag::new("whá", 1, 4)]);
        assert_parse("fdsf dfds", vec![]);
        assert_parse("#%h%", vec![]);
        assert_parse("#%", vec![]);
        assert_parse("#%h", vec![]);
        assert_parse("#h%", vec![Hashtag::new("h", 0, 1)]);
        assert_parse("#_foo_", vec![Hashtag::new("_foo_", 0, 5)]);
        assert_parse("#-foo", vec![]);
        assert_parse("#1", vec![Hashtag::new("1", 0, 1)]);
        assert_parse("#1a", vec![Hashtag::new("1a", 0, 2)]);
        assert_parse("a#b", vec![]);
        assert_parse(
            "#a#b",
            vec![Hashtag::new("a", 0, 1), Hashtag::new("b", 2, 3)],
        );
        assert_parse("#a#", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a# whatever", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a# b", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("b #a#", vec![Hashtag::new("a", 2, 3)]);
        assert_parse("b #a# b", vec![Hashtag::new("a", 2, 3)]);
        assert_parse("#á", vec![Hashtag::new("á", 0, 1)]);
        assert_parse("#mörg", vec![Hashtag::new("mörg", 0, 4)]);
        assert_parse("#a.b", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a.", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a-b", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a-", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a-a", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a.a", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#-a", vec![]);
        assert_parse("#.a", vec![]);
        assert_parse("#a-", vec![Hashtag::new("a", 0, 1)]);
        assert_parse("#a.", vec![Hashtag::new("a", 0, 1)]);
        assert_parse(
            "#a\n#b",
            vec![Hashtag::new("a", 0, 1), Hashtag::new("b", 3, 4)],
        );
        assert_parse(
            "#a  #b",
            vec![Hashtag::new("a", 0, 1), Hashtag::new("b", 4, 5)],
        );
        assert_parse(
            "#a\r\n#b",
            vec![Hashtag::new("a", 0, 1), Hashtag::new("b", 4, 5)],
        );

    }

    fn assert_parse(text: &'static str, expected_tags: Vec<Hashtag>) {
        println!("Text: {}", text);

        let actual_tags = parse_hashtags(text);
        println!("actual_tags = {:?}", actual_tags);
        assert_eq!(actual_tags.len(), expected_tags.len());
        actual_tags.iter().zip(expected_tags.iter()).for_each(
            |(a, b)| {
                assert_eq!(a, b);
            },
        );
    }
}
