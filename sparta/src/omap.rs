use crate::record::{Record, RecordType};
use otils::ObliviousOps;
use std::cmp::Ordering;

struct MapRecord(Record);

impl MapRecord {
    fn dummies(len: usize) -> Vec<Self> {
        (0..len).map(|_| MapRecord(Record::max())).collect()
    }

    fn fetch_pad(record: Record) -> Self {
        let mut record = Self(record);
        record.0.type_rec = RecordType::Send;
        record
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
        self.0.idx == other.0.idx && self.0.type_rec == other.0.type_rec
    }
}

impl PartialOrd for MapRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let idx_ord = self.0.idx.partial_cmp(&other.0.idx);
        let type_ord = self.0.type_rec.partial_cmp(&other.0.type_rec);
        match idx_ord {
            Some(Ordering::Equal) => type_ord,
            x => x,
        }
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
        let mut remaining = (self.message_store.len() + 2 * requests.len()).next_power_of_two();
        remaining -= self.message_store.len() + 2 * requests.len();
        self.message_store.reserve(remaining);

        // add padding for fetches
        self.message_store.extend(
            requests
                .iter()
                .map(|record| MapRecord::fetch_pad(record.clone())),
        );

        // add fetches
        self.message_store
            .extend(requests.into_iter().map(|r| MapRecord(r)));

        // add padding to next power of two
        self.message_store.extend(MapRecord::dummies(remaining));
    }

    pub fn batch_fetch(&mut self, requests: Vec<Record>) -> Vec<Record> {
        let original_size = self.message_store.len();
        let num_requests = requests.len();

        self.update_with_fetches(requests);

        otils::sort(&mut self.message_store[..], self.num_threads);

        let mut prev_fetch = 0;
        for record in self.message_store.iter_mut() {
            record.0.mark = u16::oselect(prev_fetch == 1, 1, 0);
            prev_fetch = i32::oselect(record.0.is_fetch(), 1, 0)
        }

        otils::compact(
            &mut self.message_store[..],
            |record| record.should_deliver(),
            self.num_threads,
        );
        let response: Vec<Record> = self
            .message_store
            .drain(0..num_requests)
            .map(|r| r.0)
            .collect();

        otils::compact(
            &mut self.message_store[..],
            |record| record.should_defer(),
            self.num_threads,
        );
        self.message_store.truncate(original_size);
        response
    }
}
