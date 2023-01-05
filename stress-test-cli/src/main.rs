use ::futures::future;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use reqwest::Client;

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, thread};

// -t (time, [seconds])
// -c (concurrency, [units])
// -u (url, [string])

lazy_static! {
    static ref CLIENT: Client = reqwest::Client::new();
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut time = 10;
    let mut concurrency = 1;
    let mut url = "http://localhost:3000";

    for (i, item) in args.iter().enumerate() {
        if item == "-t" {
            time = args
                .get(i + 1)
                .expect("provide a value for the -t option")
                .parse()
                .expect("provide an integer");
            continue;
        }

        if item == "-c" {
            concurrency = args
                .get(i + 1)
                .expect("provide a value for the -c option")
                .parse()
                .expect("provide an integer");
            continue;
        }

        if item == "-u" {
            url = args.get(i + 1).expect("provide a value for the -u option");
            continue;
        }
    }

    println!("-t {}", time);
    println!("-c {}", concurrency);

    println!("Started!");
    let requests = start_requests(time, concurrency, url).await;

    let total_requests: i32 = requests.iter().map(|(req_count, _)| req_count).sum();
    let rps: f64 = total_requests as f64 / time as f64;

    let mut avg_response_time_ms: u128 = requests
        .iter()
        .map(|(_, response_time)| response_time)
        .sum();

    avg_response_time_ms /= requests.len() as u128;

    println!("Total requests: {}", total_requests);
    println!("RPS: {}", rps);
    println!("AVG response time ms: {}", avg_response_time_ms);
}

async fn start_requests(time: i32, concurrency: i32, url: &str) -> Vec<(i32, u128)> {
    let count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&count);

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let mut count = count_clone.lock();
        *count += 1;
    });

    let handlers = (0..concurrency).map(|_| start_request(&count, time, url));
    future::join_all(handlers).await
}

async fn start_request(count: &Arc<Mutex<i32>>, time: i32, url: &str) -> (i32, u128) {
    let mut req_count = 0;
    let mut avg_response_time: u128 = 0;

    while *count.lock() <= time {
        let start = Instant::now();
        CLIENT.get(url).send().await.unwrap();

        let elapsed = start.elapsed().as_millis();
        avg_response_time = elapsed;

        req_count += 1;
    }

    (req_count, avg_response_time)
}
