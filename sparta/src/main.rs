mod load_balancer;
mod omap;
mod record;

use load_balancer::LoadBalancer;
// use omap::ObliviousMap;
use record::Record;

fn main() {
    let mut l = LoadBalancer::new(5, 8, 4);
    let sends: Vec<Record> = (0..3)
        .map(|x| Record::send(x, x.try_into().unwrap()))
        .collect();

    println!("--- SEND ---\n");
    l.batch_send(sends);

    let fetches: Vec<Record> = vec![Record::fetch(0, 3)];

    println!("--- FETCH ---\n");
    let responses = l.batch_fetch(fetches);
    for response in responses.iter() {
        println!("{:?}", response);
    }
}
