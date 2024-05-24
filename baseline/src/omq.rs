use crate::request::{Fetch, Request, Send, FETCH};
use otils::ObliviousOps;

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

    fn update_store(&mut self, sends: Vec<Send>, fetches: Vec<Fetch>, fetch_sum: usize) {
        let mut size = (self.message_store.len() + sends.len() + fetches.len() + fetch_sum)
            .next_power_of_two();

        size -= self.message_store.len();
        self.message_store.reserve(size);

        size -= fetch_sum + fetches.len() + sends.len();
        for fetch in fetches.iter() {
            self.message_store
                .extend(Request::dummies(fetch.receiver, fetch.volume));
        }

        self.message_store.extend(
            fetches
                .into_iter()
                .map(|x| <Fetch as Into<Request>>::into(x)),
        );

        self.message_store
            .extend(sends.into_iter().map(|x| <Send as Into<Request>>::into(x)));

        self.message_store.extend(Request::dummies(-1, size));
    }

    pub fn process_batch(&mut self, sends: Vec<Send>, fetches: Vec<Fetch>) {
        let final_size = self.message_store.len() + sends.len();
        let fetch_sum = fetches.iter().fold(0, |acc, f| acc + f.volume) as usize;

        self.update_store(sends, fetches, fetch_sum);

        otils::osort(&mut self.message_store[..], 8);

        let mut user_sum: isize = 0;
        let mut prev_user: i64 = -1;
        for request in self.message_store.iter_mut() {
            let same_user = prev_user == request.receiver;
            user_sum = isize::oselect(same_user, user_sum, 0);

            let is_fetch = request.req_type == FETCH;
            let fetch_more = user_sum > 0;
            request.mark = u32::oselect(is_fetch, 0, u32::oselect(fetch_more, 1, 0));

            prev_user = request.receiver;
            user_sum += isize::oselect(
                is_fetch,
                request.volume as isize,
                isize::oselect(fetch_more, -1, 0),
            );
        }

        println!("message store");
        for request in self.message_store.iter() {
            println!("{:?}", request);
        }

        let deliver = otils::ofilter(
            self.message_store.clone(),
            |r| r.req_type != FETCH && r.mark == 1,
            fetch_sum,
            8,
        );

        println!("deliver");
        for request in deliver.iter() {
            println!("{:?}", request);
        }

        self.message_store = otils::ofilter(
            self.message_store.clone(),
            |r| r.receiver >= 0 && r.req_type != FETCH && r.mark == 0,
            final_size,
            8,
        );

        println!("new message store");
        for request in self.message_store.iter() {
            println!("{:?}", request);
        }

        // otils::ofilter(data, f, num_matches, threads)
    }
}
