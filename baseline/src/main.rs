use omq::{Omq, Request};

fn main() {
    let mut o = Omq::new();

    let sends: Vec<Request> = (0..0x8)
        .map(|x| Request::new_send(0, x.try_into().unwrap()))
        .collect();
    o.batch_send(sends);
    // let mut v = vec![Request::new_fetch(0, 1); 0x200000];

    // let start = Instant::now();
    // sort(&mut v[..], 8);
    o.batch_fetch(vec![Request::new_fetch(0, 0x7)]);
    //     let time = start.elapsed();
    //     println!("{:?}", time);
}
