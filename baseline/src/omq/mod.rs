mod types;
use otils::ObliviousOps;
pub use types::{Fetch, Request, Send};

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

    fn update_store(&mut self, fetches: Vec<Fetch>, fetch_sum: usize) {
        let mut size = (self.message_store.len() + fetches.len() + fetch_sum).next_power_of_two();

        size -= self.message_store.len();
        self.message_store.reserve(size);

        size -= fetch_sum + fetches.len();
        for fetch in fetches.iter() {
            self.message_store
                .extend(Request::dummies(fetch.receiver, fetch.volume));
        }

        self.message_store.extend(
            fetches
                .into_iter()
                .map(|x| <Fetch as Into<Request>>::into(x)),
        );

        self.message_store.extend(Request::dummies(-1, size)); // TODO this is hacky
    }

    pub fn batch_send(&mut self, sends: Vec<Send>) {
        let requests = sends
            .into_iter()
            .map(|send| <Send as Into<Request>>::into(send));
        self.message_store.extend(requests);
    }

    pub fn batch_fetch(&mut self, fetches: Vec<Fetch>) -> Vec<Send> {
        let final_size = self.message_store.len();
        let fetch_sum = fetches.iter().fold(0, |acc, f| acc + f.volume) as usize;
        self.update_store(fetches, fetch_sum);

        otils::osort(&mut self.message_store[..], 8);

        let mut user_sum: isize = 0;
        let mut prev_user: i64 = -1;
        for request in self.message_store.iter_mut() {
            let same_user = prev_user == request.receiver;
            user_sum = isize::oselect(same_user, user_sum, 0);

            let fetch_more = user_sum > 0;
            request.mark = u32::oselect(request.is_fetch(), 0, u32::oselect(fetch_more, 1, 0));

            prev_user = request.receiver;
            user_sum += isize::oselect(
                request.is_fetch(),
                request.volume as isize,
                isize::oselect(fetch_more, -1, 0),
            );
        }

        otils::ocompact(&mut self.message_store[..], |r| r.should_deliver(), 8);
        let deliver = self.message_store[0..fetch_sum].to_vec();

        otils::ocompact(&mut self.message_store[..], |r| r.should_defer(), 8);
        self.message_store.truncate(final_size);

        deliver.into_iter().map(|x| x.into()).collect()
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
        let sends: Vec<Send> = (0..0x100000)
            .map(|x| Send::new(0, x.try_into().unwrap()))
            .collect();
        o.batch_send(sends);

        b.iter(|| o.batch_fetch(vec![Fetch::new(0, 0x80000)]));
    }
}
