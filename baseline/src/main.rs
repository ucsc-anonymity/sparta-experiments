mod omq;
use crate::omq::{Fetch, Omq, Send};
// use crate::types::{fetch::Fetch, send::Send};

fn main() {
    let mut o = Omq::new();
    let sends: Vec<Send> = (0..8)
        .rev()
        .map(|x| Send::new(x, x.try_into().unwrap()))
        .collect();
    o.batch_send(sends);

    let fetches: Vec<Fetch> = (0..3).rev().map(|x| Fetch::new(x, 2)).collect();

    let deliver = o.batch_fetch(fetches);
    for m in deliver {
        println!("{:?}", m);
    }

    let fetches: Vec<Fetch> = (3..6).rev().map(|x| Fetch::new(x, 1)).collect();
    let deliver = o.batch_fetch(fetches);
    for m in deliver {
        println!("{:?}", m);
    }
}
