extern crate hashtag;

use hashtag::Hashtag;
use std::time::{Instant, Duration};

fn main() {
    let string_with_hashtags = "#rust is #awesome";
    let mut buffer = String::new();
    let strings_to_join = 10_000_000;
    for _ in 0..strings_to_join {
        buffer.push_str(string_with_hashtags);
        buffer.push_str(" ");
    }

    let iterations = 10;
    let times: Vec<Duration> = (0..iterations)
        .map(|i| {
            println!("Started parsing round {}", i + 1);
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

    println!("Iterations: {}", iterations);
    println!("Average: {:?}", avg);
}

// Baseline
// Duration { secs: 4, nanos: 248856955 }
