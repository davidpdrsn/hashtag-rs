extern crate hashtag;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use hashtag::Hashtag;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn parsing_stuff_from_json_file() {
    let mut file = File::open("tests/hashtag_tests.json").expect("file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(
        "something went wrong reading the file",
    );

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    struct HashtagTest {
        text: String,
        hashtags: Vec<Hashtag>,
    };

    let hashtag_tests: Vec<HashtagTest> =
        serde_json::from_str(&contents).expect("Failed to parse json");

    for test in hashtag_tests {
        assert_parse(&test.text.clone(), test.hashtags);
    }
}

fn assert_parse(text: &str, expected_tags: Vec<Hashtag>) {
    println!("Text: {}", text);

    let actual_tags = Hashtag::parse(text);

    println!("actual_tags = {:?}", actual_tags);

    assert_eq!(actual_tags.len(), expected_tags.len());

    actual_tags.iter().zip(expected_tags.iter()).for_each(
        |(a, b)| {
            assert_eq!(a, b);
        },
    );
}
