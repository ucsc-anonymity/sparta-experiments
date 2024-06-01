use blake3;
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

    pub map: u8,
    pub type_rec: RecordType,
    pub mark: u16,
    pub idx: u32,

    pub last_fetch: u32,
    pub last_send: u32,

    pub data: u64,

    pub _dum: [u64; 12],
}

impl Record {
    pub fn new(uid: i64) -> Self {
        Record {
            uid,
            map: 0,
            type_rec: RecordType::User,
            mark: 0,
            idx: 0,
            last_fetch: 0,
            last_send: 0,
            data: 0,
            _dum: [0; 12],
        }
    }

    pub fn new_send(uid: i64, message: u64) -> Self {
        Record {
            uid,
            map: 0,
            type_rec: RecordType::Send,
            mark: 0,
            idx: 0,
            last_fetch: 0,
            last_send: 0,
            data: message,
            _dum: [0; 12],
        }
    }

    pub fn new_fetch(uid: i64, volume: u64) -> Self {
        Record {
            uid,
            map: 0,
            type_rec: RecordType::Fetch,
            mark: 0,
            idx: 0,
            last_fetch: 0,
            last_send: 0,
            data: volume,
            _dum: [0; 12],
        }
    }

    pub fn dummies(&self) -> Vec<Self> {
        (0..self.data)
            .map(|_| Record::new_fetch(self.uid, 0))
            .collect()
    }

    pub fn max() -> Self {
        Record {
            uid: Self::max_uid(),
            map: 0,
            type_rec: RecordType::User,
            mark: 0,
            idx: 0,
            last_fetch: 0,
            last_send: 0,
            data: 0,
            _dum: [0; 12],
        }
    }

    pub fn max_uid() -> i64 {
        i64::MAX
    }

    pub fn is_request(&self) -> bool {
        self.type_rec != RecordType::User
    }

    pub fn is_new_user_store(&self) -> bool {
        self.mark == 1 && self.uid != i64::MAX
    }

    pub fn is_user_store(&self) -> bool {
        self.type_rec == RecordType::User
    }

    pub fn is_fetch(&self) -> bool {
        self.type_rec == RecordType::Fetch
    }

    pub fn set_user_store(&mut self) {
        self.type_rec = RecordType::User;
    }

    pub fn get_idx(&mut self, idx: u32) -> u32 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.uid.to_ne_bytes());
        hasher.update(&idx.to_ne_bytes());
        let hash = hasher.finalize();
        u32::from_ne_bytes(<[u8; 4]>::try_from(&hash.as_bytes()[0..4]).unwrap())
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid && self.type_rec == other.type_rec
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let user_ord = self.uid.partial_cmp(&other.uid);
        let type_ord = self.type_rec.partial_cmp(&other.type_rec);
        match user_ord {
            Some(Ordering::Equal) => type_ord,
            x => x,
        }
    }
}

#[derive(Debug)]
pub struct SubmapRequest {
    pub value: Record,
}

impl SubmapRequest {
    pub fn dummies(num: usize, idx: u32, num_submaps: u32) -> Vec<Self> {
        (0..num)
            .map(|_| {
                let mut m = Record::max();
                m.map = (idx % num_submaps) as u8;
                m.into()
            })
            .collect()
    }
}

impl From<Record> for SubmapRequest {
    fn from(value: Record) -> Self {
        SubmapRequest { value }
    }
}

impl PartialEq for SubmapRequest {
    fn eq(&self, other: &Self) -> bool {
        self.value.idx == other.value.idx && self.value.uid == other.value.uid
    }
}
impl PartialOrd for SubmapRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let map_ord = self.value.map.partial_cmp(&other.value.map);
        let uid_ord = self.value.uid.partial_cmp(&other.value.uid);
        match map_ord {
            Some(Ordering::Equal) => uid_ord,
            x => x,
        }
    }
}
