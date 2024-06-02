#![feature(test)]

mod request;
use otils::ObliviousOps;
pub use request::Request;

#[derive(Debug)]
pub struct Omq {
    message_store: Vec<Request>,
}

impl Omq {
    pub fn new() -> Self {
        Omq {
            message_store: Vec::new(),
        }
    }

    pub fn batch_send(&mut self, sends: Vec<Request>) {
        for s in sends.iter() {
            println!("{:?}", s);
        }
        println!();
        self.message_store.reserve(sends.len());
        self.message_store.extend(sends);
    }

    fn update_store(&mut self, fetches: Vec<Request>, fetch_sum: usize) {
        let size = self.message_store.len() + fetches.len() + fetch_sum;

        self.message_store.reserve(size - self.message_store.len());

        for fetch in fetches.iter() {
            self.message_store
                .extend(Request::dummies(fetch.uid, fetch.volume));
        }

        self.message_store.extend(fetches);
    }

    pub fn batch_fetch(&mut self, fetches: Vec<Request>) -> Vec<Request> {
        let final_size = self.message_store.len();
        let fetch_sum = fetches.iter().fold(0, |acc, f| acc + f.volume) as usize;
        self.update_store(fetches, fetch_sum);

        self.message_store = otils::sort(std::mem::take(&mut self.message_store), 8);
        println!("sorted");
        for record in self.message_store.iter() {
            println!("{:?}", record);
        }
        println!();

        let mut user_sum: isize = 0;
        let mut prev_user: i32 = -1;
        for request in self.message_store.iter_mut() {
            let same_user = prev_user == request.uid;
            user_sum = isize::oselect(same_user, user_sum, 0);

            let fetch_more = user_sum > 0;
            request.mark = u16::oselect(request.is_fetch(), 0, u16::oselect(fetch_more, 1, 0));

            prev_user = request.uid;
            user_sum += isize::oselect(
                request.is_fetch(),
                request.volume as isize,
                isize::oselect(fetch_more, -1, 0),
            );
        }
        for record in self.message_store.iter() {
            println!("{:?}", record);
        }
        println!();

        otils::compact(&mut self.message_store[..], |r| r.should_deliver(), 8);
        let deliver = self.message_store[0..fetch_sum].to_vec();
        for record in deliver.iter() {
            println!("{:?}", record);
        }
        println!();

        otils::compact(&mut self.message_store[..], |r| r.should_defer(), 8);
        self.message_store.truncate(final_size);
        for record in self.message_store.iter() {
            println!("{:?}", record);
        }
        println!();

        deliver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate test;
    use test::Bencher;

    #[bench]
    fn bench_fetch(b: &mut Bencher) {
        let mut o = Omq::new();

        let sends: Vec<Request> = (0..1048576)
            .map(|x| Request::new_send(0, x.try_into().unwrap()))
            .collect();
        o.batch_send(sends);

        // b.iter(|| 1 + 1);

        b.iter(|| o.batch_fetch(vec![Request::new_fetch(0, 1048575)]));
    }
}
