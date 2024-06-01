mod load_balancer;
mod omap;
mod record;
use load_balancer::LoadBalancer;
use omap::ObliviousMap;
use record::Record;

fn main() {
    let mut l = LoadBalancer::new(5, 8, 4);
    let sends: Vec<Record> = (0..3)
        .map(|x| Record::new_send(x, x.try_into().unwrap()))
        .collect();

    // l.batch_send(sends);

    let fetches: Vec<Record> = vec![Record::new_fetch(0, 3)];

    let indices = l.batch_fetch(fetches);
    for i in indices.iter() {
        // println!("{:?}", i);
    }
}
