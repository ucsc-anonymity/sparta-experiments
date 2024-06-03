use crate::request::Request;
use otils::ObliviousOps;

#[derive(Debug)]
pub struct ObliviousMultiQueue {
    num_threads: usize,
    message_store: Vec<Request>,
}

impl ObliviousMultiQueue {
    pub fn new(num_threads: usize) -> Self {
        ObliviousMultiQueue {
            num_threads,
            message_store: Vec::new(),
        }
    }

    pub fn batch_send(&mut self, sends: Vec<Request>) {
        self.message_store.reserve(sends.len());
        self.message_store.extend(sends);
    }

    fn update_store(&mut self, fetches: Vec<Request>, fetch_sum: usize) {
        self.message_store.reserve(fetches.len() + fetch_sum);

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

        self.message_store = otils::sort(std::mem::take(&mut self.message_store), self.num_threads);

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

        otils::compact(
            &mut self.message_store[..],
            |r| r.should_deliver(),
            self.num_threads,
        );
        let deliver: Vec<Request> = self.message_store.drain(0..fetch_sum).collect();
        // for r in deliver.iter() {
        //     println!("{:?}", r);
        // }

        otils::compact(
            &mut self.message_store[..],
            |r| r.should_defer(),
            self.num_threads,
        );
        self.message_store.truncate(final_size);
        // for r in self.message_store.iter() {
        //     println!("{:?}", r);
        // }

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
        let mut o = ObliviousMultiQueue::new(8);

        let sends: Vec<Request> = (0..1048576)
            .map(|x| Request::new_send(0, x.try_into().unwrap()))
            .collect();
        o.batch_send(sends);

        // b.iter(|| 1 + 1);

        b.iter(|| o.batch_fetch(vec![Request::new_fetch(0, 1048575)]));
    }
}
