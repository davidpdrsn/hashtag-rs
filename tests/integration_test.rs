use hashtag::{Hashtag, HashtagParser};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{borrow::Cow, fs::File};

#[test]
fn parsing_stuff_from_json_file() {
    let mut file = File::open("tests/hashtag_tests.json").expect("file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    #[derive(Debug, Serialize, Deserialize)]
    struct HashtagTest {
        text: String,
        hashtags: Vec<DeserializeHashtag>,
    };

    let hashtag_tests: Vec<HashtagTest> =
        serde_json::from_str(&contents).expect("Failed to parse json");

    for test in hashtag_tests {
        assert_parse(
            &test.text.clone(),
            test.hashtags
                .into_iter()
                .map(|h| h.into_hashtag())
                .collect(),
        );
    }
}

fn assert_parse(text: &str, expected_tags: Vec<Hashtag>) {
    println!("Text: {}", text);

    let actual_tags = HashtagParser::new(text).collect::<Vec<_>>();

    println!("actual_tags = {:?}", actual_tags);

    assert_eq!(actual_tags.len(), expected_tags.len());

    actual_tags
        .iter()
        .zip(expected_tags.iter())
        .for_each(|(actual, expected)| {
            assert_eq!(expected, actual);
        });
}

#[derive(Debug, Serialize, Deserialize)]
struct DeserializeHashtag {
    text: String,
    start: usize,
    end: usize,
}

impl DeserializeHashtag {
    fn into_hashtag(self) -> Hashtag<'static> {
        Hashtag {
            text: Cow::from(self.text),
            start: self.start,
            end: self.end,
        }
    }
}
