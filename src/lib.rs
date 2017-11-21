#[derive(Eq, PartialEq, Debug)]
pub struct Hashtag {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Hashtag {
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
}

#[derive(Eq, PartialEq, Debug)]
pub enum Token {
    Char(char, usize),
    Space(usize),
    Hashtag(usize),
    StartOfString,
    EndOfString(usize),
}

pub fn parse_hashtags<S>(text: S) -> Vec<Hashtag>
where
    S: Into<String>,
{
    let text: String = text.into();
    let tokens = tokenize(text);
    let mut tokens_iter = tokens.iter().peekable();

    // TODO: Wrap these in some kind of state machine
    let mut parse_hashtag = false;
    let mut hashtag_start_index = 0;
    let mut hashtag_buffer = String::new();
    let mut acc: Vec<Hashtag> = Vec::new();

    loop {
        if let Some(token) = tokens_iter.next() {
            match token {
                &Token::Hashtag(idx) => {
                    if parse_hashtag {
                        hashtag_start_index = idx + 1;
                    }
                }

                &Token::Space(idx) => {
                    // TODO: Hashtags can end with other things than spaces
                    if parse_hashtag {
                        acc.push(Hashtag::new(
                            hashtag_buffer.clone(),
                            hashtag_start_index,
                            idx - 1,
                        ));
                        parse_hashtag = false;
                        hashtag_start_index = 0;
                        hashtag_buffer = String::new();
                    }

                    if let Some(&&Token::Hashtag(_)) = tokens_iter.peek() {
                        parse_hashtag = true;
                    }
                }

                &Token::Char(c, _idx) => {
                    if parse_hashtag {
                        hashtag_buffer.push(c);
                    }
                }

                &Token::StartOfString => {
                    if let Some(&&Token::Hashtag(_)) = tokens_iter.peek() {
                        parse_hashtag = true;
                    }
                }

                &Token::EndOfString(idx) => {
                    if parse_hashtag {
                        acc.push(Hashtag::new(
                            hashtag_buffer.clone(),
                            hashtag_start_index,
                            idx - 1,
                        ));
                    }
                }
            }
        } else {
            break;
        }
    }
    println!("{:?}", acc);
    acc
}

pub fn tokenize<S>(text: S) -> Vec<Token>
where
    S: Into<String>,
{
    let text: String = text.into();
    let mut tokens: Vec<Token> = vec![Token::StartOfString];
    text.chars()
        .enumerate()
        .map(|(idx, c)| match c {
            '#' => Token::Hashtag(idx),
            ' ' => Token::Space(idx),
            _ => Token::Char(c, idx),
        })
        .for_each(|token| tokens.push(token));
    tokens.push(Token::EndOfString(text.len()));
    tokens
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
        assert_parse("#here comes", vec![]);

        assert_parse("here ## comes", vec![]);
        assert_parse("here comes##", vec![]);
        assert_parse("##here comes", vec![]);
    }

    #[test]
    fn it_parses_hashtags_with_s() {
        assert_parse(
            "#bob's thing is #cool yes",
            vec![Hashtag::new("bob", 1, 3), Hashtag::new("cool", 17, 20)],
        );
    }

    // #test-123 = ["test"]
    // #test_123 = ["test_123"]
    // with emoji

    fn assert_parse(text: &'static str, expected_tags: Vec<Hashtag>) {
        println!("Failed text: {}", text);

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
