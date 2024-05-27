mod load_balancer;
use load_balancer::LoadBalancer;
use omq::{Fetch, Send};

fn main() {
    let mut l = LoadBalancer::new(5);
    let sends: Vec<Send> = (0..3)
        .map(|x| Send::new(x, x.try_into().unwrap()))
        .collect();

    let fetches: Vec<Fetch> = vec![Fetch::new(0, 3)];

    let indices = l.get_fetch_requests(fetches);
    for i in indices.iter() {
        println!("{:?}", i);
    }

    let indices = l.get_send_requests(sends);
    for i in indices.iter() {
        println!("{:?}", i);
    }
}
