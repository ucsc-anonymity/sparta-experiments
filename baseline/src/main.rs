#![feature(test)]

mod omq;
use crate::omq::{Fetch, Omq, Send};
// use crate::types::{fetch::Fetch, send::Send};

fn main() {
    let mut o = Omq::new();
    let sends: Vec<Send> = (0..100)
        .map(|x| Send::new(0, x.try_into().unwrap()))
        .collect();
    o.batch_send(sends);

    let fetches: Vec<Fetch> = vec![Fetch::new(0, 20)];

    let deliver = o.batch_fetch(fetches);
    for m in deliver {
        println!("{:?}", m);
    }

    // let fetches: Vec<Fetch> = (3..6).rev().map(|x| Fetch::new(x, 1)).collect();
    // let deliver = o.batch_fetch(fetches);
    // for m in deliver {
    //     println!("{:?}", m);
    // }
}
