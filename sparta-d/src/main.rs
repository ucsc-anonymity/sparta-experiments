mod load_balancer;
mod omap;
mod record;

use clap::Parser;
use load_balancer::LoadBalancer;
use record::Record;
use std::time::UNIX_EPOCH;

const RTT: f64 = 0.160; // 160ms
const BPS: f64 = 125000000.0; //bytes per second

/// Baseline oblivious sort based multiqueue.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of send requests to store in the database.
    sends: usize,

    /// Number of messages to fetch from the database.
    fetches: u64,

    /// Total number of threads available.
    threads: usize,

    /// Number of users in the user store.
    users: usize,

    /// Number of submaps.
    maps: usize,

    /// Total number of runs.
    #[arg(short, long, default_value = "1")]
    runs: usize,

    /// Number of runs before measurements are recorded.
    #[arg(short, long, default_value = "0")]
    warmup_runs: usize,
}

fn main() {
    let args = Args::parse();

    let mut l = LoadBalancer::new(args.users as i64, args.threads, args.maps);
    let sends: Vec<Record> = (0..args.sends)
        .map(|x| Record::send(0 as i64, x.try_into().unwrap()))
        .collect();

    l.batch_send(sends);
    let mut net_size = 0;

    let results: Vec<f64> = (0..(args.runs + args.warmup_runs))
        .map(|_| {
            let start = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let (_responses, ns) = l.batch_fetch(vec![Record::fetch(0, args.fetches)]);
            net_size = ns;
            let end = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            // for response in responses.iter() {
            //     println!("{:?}", response);
            // }

            end - start + RTT + ((net_size * std::mem::size_of::<Record>() * 2) as f64 / BPS)
        })
        .collect();

    print!("{}\t", args.sends);
    for result in results[..].iter() {
        print!(
            "{}\t",
            *result,
            // *result - RTT - ((net_size * std::mem::size_of::<Record>() * 2) as f64 / BPS)
        );
    }
    println!();
}
