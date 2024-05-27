use omq::{Fetch, Omq, Send};

fn main() {
    let mut o = Omq::new();
    let sends: Vec<Send> = (0..6)
        .map(|x| Send::new(0, x.try_into().unwrap()))
        .collect();
    o.batch_send(sends);

    let fetches: Vec<Fetch> = vec![Fetch::new(0, 3)];

    let deliver = o.batch_fetch(fetches);
    for m in deliver {
        println!("{:?}", m);
    }

    let fetches: Vec<Fetch> = vec![Fetch::new(0, 3)];

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
