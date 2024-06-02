use blake3;
use otils::Max;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum RecordType {
    User,
    Fetch,
    Send,
    Dummy,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub uid: i64,
    pub idx: u32,
    pub map: u8,

    pub rec_type: RecordType,
    pub mark: u16,

    pub last_fetch: u32,
    pub last_send: u32,

    pub data: u64,

    pub _dum: [u64; 12],
}

impl Record {
    pub fn new(uid: i64, type_rec: RecordType, data: u64, map: u8, idx: u32) -> Self {
        Record {
            uid,
            idx,
            map,
            rec_type: type_rec,
            mark: 0,
            last_fetch: 0,
            last_send: 0,
            data,
            _dum: [0; 12],
        }
    }

    pub fn send(uid: i64, message: u64) -> Self {
        Record::new(uid, RecordType::Send, message, 0, 0)
    }

    pub fn fetch(uid: i64, message: u64) -> Self {
        Record::new(uid, RecordType::Fetch, message, 0, 0)
    }

    pub fn is_user_store(&self) -> bool {
        self.rec_type == RecordType::User
    }

    pub fn is_fetch(&self) -> bool {
        self.rec_type == RecordType::Fetch
    }

    pub fn is_send(&self) -> bool {
        self.rec_type == RecordType::Send
    }
}

#[derive(Clone)]
pub struct IndexRecord(pub Record);

impl IndexRecord {
    pub fn new(uid: i64, rec_type: RecordType) -> Self {
        IndexRecord(Record::new(uid, rec_type, 0, 0, 0))
    }

    pub fn dummy_fetches(&self) -> Vec<Self> {
        (0..self.0.data)
            .map(|_| IndexRecord::new(self.0.uid, RecordType::Fetch))
            .collect()
    }

    pub fn get_idx(&self, idx: u32) -> u32 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.0.uid.to_ne_bytes());
        hasher.update(&idx.to_ne_bytes());
        let hash = hasher.finalize();
        u32::from_ne_bytes(<[u8; 4]>::try_from(&hash.as_bytes()[0..4]).unwrap())
    }

    pub fn is_request(&self) -> bool {
        self.0.rec_type != RecordType::User
    }

    pub fn is_updated_user_store(&self) -> bool {
        self.0.mark == 1 && self.0.uid != i64::MAX
    }

    pub fn set_user_store(&mut self) {
        self.0.rec_type = RecordType::User;
    }
}

impl PartialEq for IndexRecord {
    fn eq(&self, other: &Self) -> bool {
        self.0.uid == other.0.uid && self.0.rec_type == other.0.rec_type
    }
}

impl PartialOrd for IndexRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let uid_ord = self.0.uid.partial_cmp(&other.0.uid);
        let type_ord = self.0.rec_type.partial_cmp(&other.0.rec_type);
        match uid_ord {
            Some(Ordering::Equal) => type_ord,
            x => x,
        }
    }
}

impl Max for IndexRecord {
    fn maximum() -> Self {
        IndexRecord(Record::new(i64::MAX, RecordType::Dummy, 0, 0, 0))
    }
}

pub struct SubmapRecord(pub Record);

impl SubmapRecord {
    pub fn dummy_send(num_requests: usize, map: u8) -> Vec<Self> {
        (0..num_requests)
            .map(|_| SubmapRecord(Record::new(0, RecordType::Dummy, 0, map, u32::MAX)))
            .collect()
    }

    pub fn dummy_fetch(num_requests: usize, map: u8) -> Vec<Self> {
        (0..num_requests)
            .map(|_| SubmapRecord(Record::new(0, RecordType::Fetch, 0, map, u32::MAX)))
            .collect()
    }
}

impl PartialEq for SubmapRecord {
    fn eq(&self, other: &Self) -> bool {
        self.0.map == other.0.map && self.0.rec_type == other.0.rec_type
    }
}

impl PartialOrd for SubmapRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let map_ord = self.0.map.partial_cmp(&other.0.map);
        let idx_ord = self.0.idx.partial_cmp(&other.0.idx);
        match map_ord {
            Some(Ordering::Equal) => idx_ord,
            x => x,
        }
    }
}

impl Max for SubmapRecord {
    fn maximum() -> Self {
        SubmapRecord(Record::new(0, RecordType::Dummy, 0, u8::MAX, 0))
    }
}
