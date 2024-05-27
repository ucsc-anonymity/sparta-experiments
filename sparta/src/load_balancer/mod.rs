mod user_record;
use user_record::UserRecord;

use omq::{Fetch, Send};
use otils::{self, ObliviousOps};
use std::cmp;

pub struct LoadBalancer {
    num_users: i64,
    user_store: Vec<UserRecord>,
}

impl LoadBalancer {
    pub fn new(num_users: i64) -> Self {
        let mut user_store = Vec::new();
        user_store.reserve(num_users as usize);
        user_store.extend((0..num_users).map(|i| UserRecord::new(i)));
        LoadBalancer {
            num_users,
            user_store,
        }
    }

    fn update_with_sends(&mut self, sends: Vec<Send>) {
        let mut size = (self.user_store.len() + sends.len()).next_power_of_two();
        size -= self.user_store.len();
        self.user_store.reserve(size);
        size -= sends.len();

        self.user_store
            .extend(sends.into_iter().map(|s| UserRecord::from_send(s)));

        self.user_store.extend((0..size).map(|_| UserRecord::max()));
    }

    fn construct_send_indices(&mut self) {
        let mut idx: u32 = 0;
        let mut is_same_u: bool;

        let mut user_store_iter = self.user_store.iter_mut().peekable();
        while let Some(record) = user_store_iter.next() {
            let is_user_store = record.is_user_store();

            idx = u32::oselect(
                is_user_store,
                cmp::max(record.last_fetch, record.last_send),
                idx + 1,
            );

            record.idx = u32::oselect(is_user_store, 0, idx);
            record.last_send = idx;

            if let Some(next_record) = user_store_iter.peek() {
                is_same_u = record.uid == next_record.uid;
            } else {
                is_same_u = false;
            }
            record.mark = u16::oselect(is_same_u, 0, 1);
        }
    }

    pub fn get_send_requests(&mut self, sends: Vec<Send>) -> Vec<UserRecord> {
        let num_requests = sends.len();
        self.update_with_sends(sends);

        otils::osort(&mut self.user_store[..], 8);
        self.construct_send_indices();

        otils::ocompact(&mut self.user_store[..], |r| r.is_request(), 8);
        let requests = self.user_store[0..num_requests].to_vec();

        otils::ocompact(&mut self.user_store[..], |r| r.is_new_user_store(), 8);
        self.user_store.truncate(self.num_users as usize);
        self.user_store.iter_mut().for_each(|r| {
            r.set_user_store();
        });
        for record in self.user_store.iter() {
            println!("{:?}", record);
        }
        requests
    }

    fn update_with_fetches(&mut self, fetches: Vec<Fetch>, num_fetches: usize) {
        let mut size = (self.user_store.len() + num_fetches).next_power_of_two();
        size -= self.user_store.len();
        self.user_store.reserve(size);

        size -= num_fetches;
        for fetch in fetches.into_iter() {
            self.user_store.extend(UserRecord::from_fetch(fetch));
        }

        self.user_store.extend((0..size).map(|_| UserRecord::max()));
    }

    fn construct_fetch_indices(&mut self) {
        let mut idx: u32 = 0;
        let mut is_same_u: bool;

        let mut user_store_iter = self.user_store.iter_mut().peekable();
        while let Some(record) = user_store_iter.next() {
            let is_user_store = record.is_user_store();

            idx = u32::oselect(is_user_store, record.last_fetch, idx + 1);

            record.idx = u32::oselect(is_user_store, 0, idx);
            record.last_fetch = idx;

            if let Some(next_record) = user_store_iter.peek() {
                is_same_u = record.uid == next_record.uid;
            } else {
                is_same_u = false;
            }
            record.mark = u16::oselect(is_same_u, 0, 1);
        }
    }

    pub fn get_fetch_requests(&mut self, fetches: Vec<Fetch>) -> Vec<UserRecord> {
        let num_requests = fetches.iter().fold(0, |acc, x| acc + x.volume as usize);
        self.update_with_fetches(fetches, num_requests);

        otils::osort(&mut self.user_store[..], 8);
        self.construct_fetch_indices();

        otils::ocompact(&mut self.user_store[..], |r| r.is_request(), 8);
        let deliver = self.user_store[0..num_requests].to_vec();

        otils::ocompact(&mut self.user_store[..], |r| r.is_new_user_store(), 8);
        self.user_store.truncate(self.num_users as usize);
        self.user_store.iter_mut().for_each(|r| {
            r.set_user_store();
        });
        for record in self.user_store.iter() {
            println!("{:?}", record);
        }
        deliver
    }

    // pub fn batch_fetch(&mut self, fetches: Vec<Fetch>) -> Vec<Send> {
    //     let deliver_size = fetches.iter().fold(0, |acc, x| acc + x.volume as usize);
    // }
}
