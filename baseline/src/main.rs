use std::time::UNIX_EPOCH;

use omq::{Omq, Request};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let num_sends = args[1].parse::<usize>().unwrap();
    let num_fetches = args[2].parse::<usize>().unwrap();
    let num_threads = args[3].parse::<usize>().unwrap();

    let mut o = Omq::new(num_threads);

    let sends: Vec<Request> = (0..num_sends)
        .map(|x| Request::new_send(0, x.try_into().unwrap()))
        .collect();
    o.batch_send(sends);
    let results: Vec<u128> = (0..15)
        .map(|_| {
            let start = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let _ = o.batch_fetch(vec![Request::new_fetch(0, num_fetches)]);
            let end = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            end - start
        })
        .collect();
    for idx in 5..results.len() {
        print!("{}\t", results[idx]);
    }
    println!();
}
