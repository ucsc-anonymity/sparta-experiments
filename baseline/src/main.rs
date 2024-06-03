#![feature(test)]
mod omq;
mod request;

use clap::Parser;
use omq::ObliviousMultiQueue;
use request::Request;
use std::time::UNIX_EPOCH;

/// Baseline oblivious sort based multiqueue.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of send requests to store in the database.
    sends: usize,

    /// Number of messages to fetch from the database.
    fetches: usize,

    /// Total number of threads available.
    threads: usize,

    /// Total number of runs.
    #[arg(short, long, default_value = "1")]
    runs: usize,

    /// Number of runs before measurements are recorded.
    #[arg(short, long, default_value = "0")]
    warmup_runs: usize,
}

fn main() {
    let args = Args::parse();

    let mut o = ObliviousMultiQueue::new(args.threads);

    let sends: Vec<Request> = (0..args.sends)
        .map(|x| Request::new_send(0, x.try_into().unwrap()))
        .collect();
    o.batch_send(sends);
    let results: Vec<u128> = (0..(args.runs + args.warmup_runs))
        .map(|_| {
            let start = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let _ = o.batch_fetch(vec![Request::new_fetch(0, args.fetches)]);
            let end = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            end - start
        })
        .collect();

    print!("{}\t", args.sends);
    for result in results[args.warmup_runs..].iter() {
        print!("{}\t", *result as f64 / 1000000000.0);
    }
    println!();
}
