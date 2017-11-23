extern crate hashtag;

use hashtag::Hashtag;
use std::time::{Instant, Duration};

fn main() {
    let string_with_hashtags = "#rust is #awesome";
    benchmark_small_string(&string_with_hashtags);
    benchmark_big_string(&string_with_hashtags);
}

fn benchmark_big_string(string_with_hashtags: &str) {
    let mut buffer = String::new();
    let strings_to_join = 10_000_000;
    for _ in 0..strings_to_join {
        buffer.push_str(string_with_hashtags);
        buffer.push_str(" ");
    }

    let iterations = 10;
    let times: Vec<Duration> = (0..iterations)
        .map(|i| {
            println!("{} / {}", i + 1, iterations);
            let start = Instant::now();
            let hashtags = Hashtag::parse(&buffer);
            let count = hashtags.len();
            let duration = start.elapsed();
            assert_eq!(count, strings_to_join * 2);
            duration
        })
        .collect();

    let total: Duration = times.iter().sum();
    let avg: Duration = total / (times.len() as u32);
    println!("Big string average: {:?}", avg);
}

fn benchmark_small_string(string_with_hashtags: &str) {
    let iterations = 10_000_000;

    let times: Vec<_> = (0..iterations)
        .map(|i| {
            if (i % 1_000_000) == 0 {
                println!("{} / {}", (i / 1_000_000) + 1, iterations / 1_000_000);
            }

            let start = Instant::now();
            let hashtags = Hashtag::parse(&string_with_hashtags);
            let count = hashtags.len();
            let duration = start.elapsed();
            assert_eq!(count, 2);
            duration
        })
        .collect();

    let total: Duration = times.iter().sum();
    let avg: Duration = total / (times.len() as u32);
    println!("Small string average: {:?}", avg);
}
