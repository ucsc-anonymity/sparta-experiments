mod load_balancer;
mod omap;
mod record;

use std::time::UNIX_EPOCH;

use load_balancer::LoadBalancer;
use record::Record;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let num_users = args[1].parse::<usize>().unwrap();
    let num_sends = args[2].parse::<usize>().unwrap();
    let num_fetches = args[3].parse::<usize>().unwrap();
    let num_threads = args[4].parse::<usize>().unwrap();
    let num_maps = args[5].parse::<usize>().unwrap();

    let mut l = LoadBalancer::new(num_users as i64, num_threads, num_maps);
    let sends: Vec<Record> = (0..num_sends)
        .map(|x| Record::send(x as i64, x.try_into().unwrap()))
        .collect();

    l.batch_send(sends);

    let results: Vec<u128> = (0..15)
        .map(|_| {
            let start = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let _ = l.batch_fetch(vec![Record::fetch(0, num_fetches as u64)]);
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
