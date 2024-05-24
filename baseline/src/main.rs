mod omq;
mod request;

use crate::omq::Omq;
use crate::request::{Fetch, Send};

fn main() {
    let mut o = Omq::new();
    let sends: Vec<Send> = (0..8)
        .rev()
        .map(|x| Send::new(x, x.try_into().unwrap()))
        .collect();

    let fetch: Vec<Fetch> = (0..3)
        .rev()
        .map(|x| Fetch::new(x, (x + 1).try_into().unwrap()))
        .collect();
    o.process_batch(sends, fetch);
}
