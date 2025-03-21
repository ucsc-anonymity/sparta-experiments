use crate::omap::ObliviousMap;
pub use crate::record::{IndexRecord, Record, RecordType, SubmapRecord};
use fastapprox::fast;
use otils::{self, Max, ObliviousOps};
use rayon::ThreadPool;
use std::{
    cmp,
    f64::consts::E,
    sync::{Arc, Mutex},
    // time::UNIX_EPOCH,
};

const LAMBDA: usize = 128;

pub struct LoadBalancer {
    num_users: i64,
    num_submaps: usize,
    num_threads: usize,
    pool: ThreadPool,
    pub user_store: Vec<IndexRecord>,
    pub submaps: Vec<ObliviousMap>,
}

impl LoadBalancer {
    pub fn new(num_users: i64, num_threads: usize, num_submaps: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        let mut user_store = Vec::new();
        user_store.reserve(num_users as usize);
        user_store.extend((0..num_users).map(|i| IndexRecord::new(i, RecordType::User)));

        let mut submaps = Vec::with_capacity(num_submaps as usize);
        submaps.extend((0..num_submaps).map(|_| ObliviousMap::new()));

        LoadBalancer {
            num_users,
            num_submaps,
            num_threads,

            pool,
            user_store,
            submaps,
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

    pub fn pad_for_submap(
        &self,
        mut requests: Vec<SubmapRecord>,
        submap_size: usize,
        is_send: bool,
    ) -> Vec<SubmapRecord> {
        requests.reserve(self.num_submaps * submap_size);

        for submap in 0..self.num_submaps {
            if is_send {
                requests.extend(SubmapRecord::dummy_send(submap_size, submap as u8));
            } else {
                requests.extend(SubmapRecord::dummy_fetch(submap_size, submap as u8));
            }
        }
        requests
    }

    pub fn get_submap_requests(
        &self,
        requests: Vec<IndexRecord>,
        submap_size: usize,
        is_send: bool,
    ) -> Vec<SubmapRecord> {
        let requests: Vec<SubmapRecord> = requests.into_iter().map(|r| SubmapRecord(r.0)).collect();

        let mut requests = self.pad_for_submap(requests, submap_size, is_send);

        requests = otils::sort(requests, &self.pool, self.num_threads); // sort by omap, then by dummy

        let mut prev_map = self.num_submaps;
        let mut remaining_marks = submap_size as i32;
        for request in requests.iter_mut() {
            let submap = request.0.map as u32;
            remaining_marks = i32::oselect(
                submap != prev_map as u32,
                submap_size as i32,
                remaining_marks,
            );
            request.0.mark = u16::oselect(remaining_marks > 0, 1, 0);
            remaining_marks += i32::oselect(remaining_marks > 0, -1, 0);
            prev_map = submap as usize;
        }

        otils::compact(
            &mut requests[..],
            |r| r.0.mark == 1,
            &self.pool,
            self.num_threads,
        );
        requests.truncate(self.num_submaps * submap_size);
        requests
    }

    fn propagate_send_indices(&mut self) {
        let mut idx: u32 = 0;
        let mut is_same_u: bool;

        let mut user_store_iter = self.user_store.iter_mut().peekable();
        while let Some(record) = user_store_iter.next() {
            let is_user_store = record.0.is_user_store();

            idx = u32::oselect(
                is_user_store,
                cmp::max(record.0.last_fetch, record.0.last_send),
                idx + 1,
            );

            record.0.idx = u32::oselect(is_user_store, 0, record.get_idx(idx));
            record.0.map = (record.0.idx % (self.num_submaps as u32)) as u8;
            record.0.last_send = idx;

            if let Some(next_record) = user_store_iter.peek() {
                is_same_u = record.0.uid == next_record.0.uid;
            } else {
                is_same_u = false;
            }
            record.0.mark = u16::oselect(is_same_u, 0, 1);
        }
    }

    pub fn get_send_indices(&mut self, sends: Vec<IndexRecord>) -> Vec<IndexRecord> {
        let num_requests = sends.len();
        self.user_store.reserve(num_requests);
        self.user_store.extend(sends);

        self.user_store = otils::sort(
            std::mem::take(&mut self.user_store),
            &self.pool,
            self.num_threads,
        );
        self.propagate_send_indices();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_request(),
            &self.pool,
            self.num_threads,
        );
        let requests = self.user_store.drain(0..num_requests).collect();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_updated_user_store(),
            &self.pool,
            self.num_threads,
        );

        self.user_store.truncate(self.num_users as usize);
        self.user_store.iter_mut().for_each(|r| {
            r.set_user_store();
        });

        requests
    }

    pub fn batch_send(&mut self, sends: Vec<Record>) {
        let sends = sends.into_iter().map(|r| IndexRecord(r)).collect();
        let requests = self.get_send_indices(sends);
        let submap_size = self.pad_size(requests.len() as f64);
        let mut requests: Vec<Record> = self
            .get_submap_requests(requests, submap_size, true)
            .into_iter()
            .map(|r| r.0)
            .collect();

        let mut remaining_submaps = &mut self.submaps[..];

        self.pool.scope(|s| {
            for _ in 0..self.num_submaps {
                let (submap, rest_submaps) = remaining_submaps.split_at_mut(1);
                remaining_submaps = rest_submaps;

                let batch = requests.drain(0..submap_size).collect();
                s.spawn(|_| submap[0].batch_send(batch));
            }

            // let (submap, rest_submaps) = remaining_submaps.split_at_mut(1);
            // remaining_submaps = rest_submaps;

            // let batch = requests.drain(0..submap_size).collect();
            // submap[0].batch_send(batch);
        });
    }

    fn update_with_fetches(&mut self, fetches: Vec<IndexRecord>, num_fetches: usize) {
        self.user_store.reserve(num_fetches);
        for fetch in fetches.into_iter() {
            self.user_store.extend(fetch.dummy_fetches());
        }
    }

    fn propagate_fetch_indices(&mut self) {
        let mut idx: u32 = 0;
        let mut is_same_u: bool;

        let mut user_store_iter = self.user_store.iter_mut().peekable();
        while let Some(record) = user_store_iter.next() {
            let is_user_store = record.0.is_user_store();

            idx = u32::oselect(is_user_store, record.0.last_fetch, idx + 1);

            record.0.idx = u32::oselect(is_user_store, 0, record.get_idx(idx));
            record.0.map = (record.0.idx % (self.num_submaps as u32)) as u8;
            record.0.last_fetch = idx;

            if let Some(next_record) = user_store_iter.peek() {
                is_same_u = record.0.uid == next_record.0.uid;
            } else {
                is_same_u = false;
            }
            record.0.mark = u16::oselect(is_same_u, 0, 1);
        }
    }

    pub fn get_fetch_indices(
        &mut self,
        fetches: Vec<IndexRecord>,
        num_requests: usize,
    ) -> Vec<IndexRecord> {
        self.update_with_fetches(fetches, num_requests);

        self.user_store = otils::sort(
            std::mem::take(&mut self.user_store),
            &self.pool,
            self.num_threads,
        );
        self.propagate_fetch_indices();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_request(),
            &self.pool,
            self.num_threads,
        );
        let deliver = self.user_store.drain(0..num_requests).collect();

        otils::compact(
            &mut self.user_store[..],
            |r| r.is_updated_user_store(),
            &self.pool,
            self.num_threads,
        );

        self.user_store.truncate(self.num_users as usize);
        self.user_store.iter_mut().for_each(|r| {
            r.set_user_store();
        });

        deliver
    }

    pub fn batch_fetch(&mut self, fetches: Vec<Record>) -> (Vec<Record>, usize) {
        let num_requests = fetches
            .iter()
            .fold(0, |acc, fetch| acc + fetch.data as usize);
        let fetches = fetches.into_iter().map(|r| IndexRecord(r)).collect();

        // let t1 = std::time::SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap()
        //     .as_secs_f64();

        let requests = self.get_fetch_indices(fetches, num_requests);

        let submap_size = self.pad_size(requests.len() as f64);

        let mut requests: Vec<Record> = self
            .get_submap_requests(requests, submap_size, false)
            .into_iter()
            .map(|r| r.0)
            .collect();

        let remaining_submaps = &mut self.submaps[..];
        let responses: Arc<Mutex<Vec<IndexRecord>>> = Arc::new(Mutex::new(Vec::with_capacity(
            submap_size * self.num_submaps,
        )));

        // let t2 = std::time::SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap()
        //     .as_secs_f64();

        let (submap, _rest_submaps) = remaining_submaps.split_at_mut(1);
        let batch = requests.drain(0..submap_size).collect();
        {
            let responses = Arc::clone(&responses);
            let response = submap[0].batch_fetch(batch, &self.pool, self.num_threads);
            let mut responses = responses.lock().unwrap();
            responses.extend(response);
            responses
                .extend((0..submap_size * (self.num_submaps - 1)).map(|_| IndexRecord::maximum()));
        }

        // let t3 = std::time::SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap()
        //     .as_secs_f64();

        let mutex = Arc::into_inner(responses).unwrap();
        let mut responses: Vec<IndexRecord> = mutex.into_inner().unwrap();
        responses = otils::sort(responses, &self.pool, self.num_threads);
        otils::compact(
            &mut responses,
            |r| r.0.is_send(),
            &self.pool,
            self.num_threads,
        );

        // let t4 = std::time::SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap()
        //     .as_secs_f64();

        (
            responses.drain(0..num_requests).map(|r| r.0).collect(),
            submap_size * self.num_submaps,
        )
    }
}
