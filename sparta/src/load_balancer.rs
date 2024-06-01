pub use crate::record::{Record, SubmapRequest};
use crate::ObliviousMap;
use fastapprox::fast;
use otils::{self, ObliviousOps};
use std::{cmp, f64::consts::E};

const LAMBDA: usize = 128;

pub struct BalanceRecord(Record);

impl BalanceRecord {}

impl PartialEq for BalanceRecord {
    fn eq(&self, other: &Self) -> bool {}
}

impl PartialOrd for BalanceRecord {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {}
}

pub struct LoadBalancer {
    num_users: i64,
    num_threads: usize,
    num_submaps: usize,

    pub user_store: Vec<Record>,
    pub omaps: Vec<ObliviousMap>,
}

impl LoadBalancer {
    pub fn new(num_users: i64, num_threads: usize, num_submaps: usize) -> Self {
        let mut user_store = Vec::new();
        user_store.reserve(num_users as usize);
        user_store.extend((0..num_users).map(|i| Record::new(i)));

        let mut omaps = Vec::new();
        omaps.reserve(num_submaps as usize);
        omaps.extend(
            (0..num_submaps).map(|_| ObliviousMap::new(num_threads / num_submaps as usize)),
        );

        LoadBalancer {
            num_users,
            num_threads,
            num_submaps,
            user_store,
            omaps,
        }
    }

    fn pad_size(&self, num_requests: f64) -> usize {
        let num_submaps = self.num_submaps as f64;
        let mu = num_requests / num_submaps;
        let gamma = (num_submaps + 2_f64.powf(LAMBDA as f64)).ln();
        let rhs = (gamma / mu - 1_f64) / E;
        num_requests
            .min(mu * E.powf(fast::lambertw(rhs as f32) as f64 + 1_f64))
            .ceil() as usize
    }

    fn update_with_sends(&mut self, sends: Vec<Record>) {
        let mut size = (self.user_store.len() + sends.len()).next_power_of_two();
        size -= self.user_store.len();
        self.user_store.reserve(size);
        size -= sends.len();

        self.user_store.extend(sends);

        self.user_store.extend((0..size).map(|_| Record::max()));
    }

    fn update_with_fetches(&mut self, fetches: Vec<Record>, num_fetches: usize) {
        let mut size = (self.user_store.len() + num_fetches).next_power_of_two();
        size -= self.user_store.len();
        self.user_store.reserve(size);

        size -= num_fetches;
        for fetch in fetches.into_iter() {
            self.user_store.extend(fetch.dummies());
        }

        self.user_store.extend((0..size).map(|_| Record::max()));
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

            record.idx = u32::oselect(is_user_store, 0, record.get_idx(idx));
            record.map = (record.idx % self.num_submaps) as u8;
            record.last_send = idx;

            if let Some(next_record) = user_store_iter.peek() {
                is_same_u = record.uid == next_record.uid;
            } else {
                is_same_u = false;
            }
            record.mark = u16::oselect(is_same_u, 0, 1);
        }
    }

    fn construct_fetch_indices(&mut self) {
        let mut idx: u32 = 0;
        let mut is_same_u: bool;

        let mut user_store_iter = self.user_store.iter_mut().peekable();
        while let Some(record) = user_store_iter.next() {
            let is_user_store = record.is_user_store();

            idx = u32::oselect(is_user_store, record.last_fetch, idx + 1);

            record.idx = u32::oselect(is_user_store, 0, record.get_idx(idx));
            record.map = (record.idx % self.num_submaps) as u8;
            record.last_fetch = idx;

            if let Some(next_record) = user_store_iter.peek() {
                is_same_u = record.uid == next_record.uid;
            } else {
                is_same_u = false;
            }
            record.mark = u16::oselect(is_same_u, 0, 1);
        }
    }

    pub fn get_send_requests(&mut self, sends: Vec<Record>) -> Vec<Record> {
        let num_requests = sends.len();
        self.update_with_sends(sends);

        otils::sort(&mut self.user_store[..], self.num_threads);
        self.construct_send_indices();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_request(),
            self.num_threads,
        );
        let requests = self.user_store[0..num_requests].to_vec();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_new_user_store(),
            self.num_threads,
        );
        self.user_store.truncate(self.num_users as usize);
        self.user_store.iter_mut().for_each(|r| {
            r.set_user_store();
        });

        requests
    }

    pub fn get_fetch_requests(&mut self, fetches: Vec<Record>) -> Vec<Record> {
        let num_requests = fetches
            .iter()
            .fold(0, |acc, fetch| acc + fetch.data as usize);
        self.update_with_fetches(fetches, num_requests);

        otils::sort(&mut self.user_store[..], self.num_threads);
        self.construct_fetch_indices();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_request(),
            self.num_threads,
        );
        let deliver = self.user_store[0..num_requests].to_vec();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_new_user_store(),
            self.num_threads,
        );
        self.user_store.truncate(self.num_users as usize);
        self.user_store.iter_mut().for_each(|r| {
            r.set_user_store();
        });

        deliver
    }

    pub fn pad_for_submap(&self, requests: Vec<Record>, submap_size: usize) -> Vec<SubmapRequest> {
        let num_submaps = self.num_submaps as usize;
        let mut remaining = (requests.len() + num_submaps * submap_size).next_power_of_two();
        remaining -= requests.len();

        let mut requests: Vec<SubmapRequest> = requests.into_iter().map(|r| r.into()).collect();
        requests.reserve(remaining);

        for idx in 0..num_submaps {
            requests.extend(SubmapRequest::dummies(
                submap_size,
                idx as u32,
                self.num_submaps,
            ));
        }
        remaining -= num_submaps * submap_size;
        requests.extend((0..remaining).map(|_| Record::max().into()));
        requests
    }

    pub fn get_submap_requests(
        &self,
        requests: Vec<Record>,
        submap_size: usize,
    ) -> Vec<SubmapRequest> {
        let mut requests = self.pad_for_submap(requests, submap_size);

        otils::sort(&mut requests[..], self.num_threads); // sort by omap, then by dummy

        let mut prev_map = self.num_submaps;
        let mut remaining_marks = submap_size as i32;
        for request in requests.iter_mut() {
            let submap = request.value.map as u32;
            remaining_marks = i32::oselect(submap != prev_map, submap_size as i32, remaining_marks);
            request.value.mark = u16::oselect(remaining_marks > 0, 1, 0);
            remaining_marks += i32::oselect(remaining_marks > 0, -1, 0);
            prev_map = submap;
        }

        otils::compact(&mut requests[..], |r| r.value.mark == 1, self.num_threads);
        requests
    }

    pub fn batch_send(&mut self, sends: Vec<Record>) {
        let requests = self.get_send_requests(sends);
        let submap_size = self.pad_size(requests.len() as f64);
        let mut requests: Vec<Record> = self
            .get_submap_requests(requests, submap_size)
            .into_iter()
            .map(|r| r.value)
            .collect();

        for idx in 0..self.num_submaps {
            let batch = requests.drain(0..submap_size).collect();
            self.omaps[idx].batch_send(batch);
        }
    }

    pub fn batch_fetch(&mut self, fetches: Vec<Record>) -> Vec<Record> {
        let requests = self.get_fetch_requests(fetches);
        let submap_size = self.pad_size(requests.len() as f64);
        let mut requests: Vec<Record> = self
            .get_submap_requests(requests, submap_size)
            .into_iter()
            .map(|r| r.value)
            .collect();

        let mut responses: Vec<Record> = Vec::new();
        responses.reserve(submap_size * self.num_submaps);
        for idx in 0..self.num_submaps {
            let batch = requests.drain(0..submap_size).collect();
            responses.extend(self.omaps[idx].batch_fetch(batch));
        }

        responses
    }
}
