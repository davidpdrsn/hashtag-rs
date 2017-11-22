#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[derive(Eq, PartialEq, Debug, Serialize)]
pub struct Hashtag {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Hashtag {
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
        self.hashtag_start_index = idx + 1;
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

                &Token::Space(idx) => {
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
    Space(usize),
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
            _ => false,
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
                ' ' => Token::Space(idx),
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
            &'\'' | &' ' | &'%' | &'#' => true,
            _ => false,
        }
    }
}

impl IsEndOfHashtag for Token {
    fn is_end_of_hashtag(&self) -> bool {
        match self {
            &Token::Space(_) => true,
            &Token::Char(c, _) => c.is_end_of_hashtag(),
            _ => false,
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
                Token::Space(4),
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
            vec![Hashtag::new("foo", 22, 24), Hashtag::new("bar", 27, 29)],
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
            vec![Hashtag::new("foo", 1, 3), Hashtag::new("foo", 17, 19)],
        )
    }

    #[test]
    fn it_parses_hashes_without_text() {
        assert_parse("here # comes", vec![]);
        assert_parse("here comes#", vec![]);
        assert_parse("#here comes", vec![Hashtag::new("here", 1, 4)]);

        assert_parse("here ## comes", vec![]);
        assert_parse("here comes##", vec![]);
        assert_parse("##here comes", vec![Hashtag::new("here", 2, 5)]);
    }

    #[test]
    fn it_parses_hashtags_with_s() {
        assert_parse(
            "#bob's thing is #cool yes",
            vec![Hashtag::new("bob", 1, 3), Hashtag::new("cool", 17, 20)],
        );
    }

    #[test]
    fn it_parses_many_different_kinds() {
        assert_parse("#a1", vec![Hashtag::new("a1", 1, 2)]);
        assert_parse("#a_1", vec![Hashtag::new("a_1", 1, 3)]);
        assert_parse("#a-1", vec![Hashtag::new("a-1", 1, 3)]);
        assert_parse("#a.1", vec![Hashtag::new("a.1", 1, 3)]);
        assert_parse("#😀", vec![Hashtag::new("😀", 1, 1)]);
        assert_parse(" #whá", vec![Hashtag::new("whá", 2, 4)]);
        assert_parse("fdsf dfds", vec![]);
        assert_parse("#%h%", vec![]);
        assert_parse("#%", vec![]);
        assert_parse("#%h", vec![]);
        assert_parse("#h%", vec![Hashtag::new("h", 1, 1)]);
        assert_parse("#_foo_", vec![Hashtag::new("_foo_", 1, 5)]);
        assert_parse("#-foo-", vec![Hashtag::new("-foo-", 1, 5)]);
        assert_parse("#1", vec![Hashtag::new("1", 1, 1)]);
        assert_parse("#1a", vec![Hashtag::new("1a", 1, 2)]);
        assert_parse("a#b", vec![]);
        assert_parse(
            "#a#b",
            vec![Hashtag::new("a", 1, 1), Hashtag::new("b", 3, 3)],
        );
        assert_parse("#a#", vec![Hashtag::new("a", 1, 1)]);
        assert_parse("#a# whatever", vec![Hashtag::new("a", 1, 1)]);
        assert_parse("#a# b", vec![Hashtag::new("a", 1, 1)]);
        assert_parse("b #a#", vec![Hashtag::new("a", 3, 3)]);
        assert_parse("b #a# b", vec![Hashtag::new("a", 3, 3)]);
        assert_parse("#á", vec![Hashtag::new("á", 1, 1)]);
        assert_parse("#m-örg", vec![Hashtag::new("m-örg", 1, 5)]);
        assert_parse("#m.örg", vec![Hashtag::new("m.örg", 1, 5)]);

        // Emoji that are more than one codepoint...
        // assert_parse("#☝🏽", vec![Hashtag::new("#☝🏽", 1, 1)]);
        // assert_parse(
        //     "#👩‍👩‍👦‍👦",
        //     vec![Hashtag::new("#👩‍👩‍👦‍👦", 1, 1)],
        // );
    }

    fn assert_parse(text: &'static str, expected_tags: Vec<Hashtag>) {
        println!("Text: {}", text);

        let actual_tags = parse_hashtags(text);
        assert_eq!(actual_tags.len(), expected_tags.len());
        actual_tags.iter().zip(expected_tags.iter()).for_each(
            |(a, b)| {
                assert_eq!(a, b);
            },
        );
        //
    }
}
