use crate::record::{IndexRecord, Record, RecordType};
use otils::{Max, ObliviousOps};
use std::cmp::Ordering;

struct MapRecord(Record);

impl MapRecord {
    fn dummy_send(idx: u32) -> Self {
        MapRecord(Record::new(0, RecordType::Dummy, 0, 0, idx))
    }

    fn should_deliver(&self) -> bool {
        !self.0.is_fetch() && self.0.mark == 1
    }

    fn should_defer(&self) -> bool {
        !self.0.is_fetch() && self.0.mark == 0
    }
}

impl PartialEq for MapRecord {
    fn eq(&self, other: &Self) -> bool {
        self.0.idx == other.0.idx && self.0.rec_type == other.0.rec_type
    }
}

impl PartialOrd for MapRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let idx_ord = self.0.idx.partial_cmp(&other.0.idx);
        let type_ord = self.0.rec_type.partial_cmp(&other.0.rec_type);
        match idx_ord {
            Some(Ordering::Equal) => type_ord,
            x => x,
        }
    }
}

impl Max for MapRecord {
    fn maximum() -> Self {
        MapRecord(Record::new(0, RecordType::Dummy, 0, 0, u32::MAX))
    }
}

pub struct ObliviousMap {
    num_threads: usize,
    message_store: Vec<MapRecord>,
}

impl ObliviousMap {
    pub fn new(num_threads: usize) -> Self {
        let message_store = Vec::new();
        ObliviousMap {
            num_threads,
            message_store,
        }
    }

    pub fn batch_send(&mut self, requests: Vec<Record>) {
        self.message_store.reserve(requests.len());
        self.message_store
            .extend(requests.into_iter().map(|r| MapRecord(r)));
    }

    fn update_with_fetches(&mut self, requests: Vec<Record>) {
        self.message_store.reserve(2 * requests.len());

        // add padding for fetches
        self.message_store.extend(
            requests
                .iter()
                .map(|record| MapRecord::dummy_send(record.idx)),
        );

        // add fetches
        self.message_store
            .extend(requests.into_iter().map(|r| MapRecord(r)));
    }

    pub fn batch_fetch(&mut self, requests: Vec<Record>) -> Vec<IndexRecord> {
        let final_size = self.message_store.len();
        let num_requests = requests.len();

        self.update_with_fetches(requests);

        self.message_store = otils::sort(std::mem::take(&mut self.message_store), self.num_threads);

        let mut prev_idx = u32::MAX;
        let mut remaining = 0;
        for record in self.message_store.iter_mut() {
            remaining = i32::oselect(prev_idx == record.0.idx, remaining, 0);
            record.0.mark = u16::oselect(record.0.is_fetch(), 0, u16::oselect(remaining > 0, 1, 0));

            prev_idx = record.0.idx;
            remaining += i32::oselect(record.0.is_fetch(), 1, i32::oselect(remaining > 0, -1, 0));
        }

        otils::compact(
            &mut self.message_store[..],
            |r| r.should_deliver(),
            self.num_threads,
        );

        let response: Vec<IndexRecord> = self
            .message_store
            .drain(0..num_requests)
            .map(|r| IndexRecord(r.0))
            .collect();

        otils::compact(
            &mut self.message_store[..],
            |record| record.should_defer(),
            self.num_threads,
        );
        self.message_store.truncate(final_size);

        response
    }
}
