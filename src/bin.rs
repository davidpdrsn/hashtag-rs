extern crate hashtag;

use hashtag::Hashtag;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;

pub fn main() {
    let start_read_file = Instant::now();
    let mut file = File::open("sample_tags.txt").expect("Failed to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let duration_read_file = start_read_file.elapsed();

    let mut count = 0;
    let start = Instant::now();
    for line in contents.lines() {
        let tags = Hashtag::parse(line);
        count += tags.len();
    }
    let duration = start.elapsed();

    println!("Found {:?} total hashtags", count);
    println!("Reading the file took {:?}", duration_read_file);
    println!("Parsing took {:?}", duration);
}
