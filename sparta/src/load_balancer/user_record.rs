use omq::{Fetch, Send};
use otils::{self, ObliviousOps};
use std::cmp::Ordering;

const USER_STORE: u16 = 0;
const FETCH: u16 = 1;
const SEND: u16 = 2;

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub uid: i64,
    pub type_rec: u16,
    pub mark: u16,
    pub last_fetch: u32,
    pub last_send: u32,
    pub idx: u32,
    pub message: u64,
}

impl UserRecord {
    pub fn new(uid: i64) -> Self {
        UserRecord {
            uid,
            type_rec: USER_STORE,
            mark: 0,
            last_fetch: 0,
            last_send: 0,
            idx: 0,
            message: 0,
        }
    }

    pub fn max() -> Self {
        UserRecord {
            uid: i64::MAX,
            type_rec: USER_STORE,
            mark: 0,
            last_fetch: 0,
            last_send: 0,
            idx: 0,
            message: 0,
        }
    }

    pub fn from_fetch(f: Fetch) -> Vec<Self> {
        (0..f.volume)
            .map(|_| UserRecord {
                uid: f.receiver,
                type_rec: FETCH,
                mark: 0,
                last_fetch: 0,
                last_send: 0,
                idx: 0,
                message: 0,
            })
            .collect()
    }

    pub fn from_send(s: Send) -> Self {
        UserRecord {
            uid: s.receiver,
            type_rec: SEND,
            mark: 0,
            last_fetch: 0,
            last_send: 0,
            idx: 0,
            message: s.message,
        }
    }

    pub fn is_request(&self) -> bool {
        self.type_rec != USER_STORE
    }

    pub fn is_new_user_store(&self) -> bool {
        self.mark == 1 && self.uid != i64::MAX
    }

    pub fn is_user_store(&self) -> bool {
        self.type_rec == USER_STORE
    }

    pub fn is_fetch(&self) -> bool {
        self.type_rec == FETCH
    }

    pub fn set_user_store(&mut self) {
        self.type_rec = USER_STORE;
    }
}

impl PartialEq for UserRecord {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid && self.type_rec == other.type_rec
    }
}

impl PartialOrd for UserRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let user_ord = self.uid.partial_cmp(&other.uid);
        let type_ord = self.type_rec.partial_cmp(&other.type_rec);
        match user_ord {
            Some(Ordering::Equal) => type_ord,
            x => x,
        }
    }
}

impl ObliviousOps for UserRecord {
    fn oselect(cond: bool, a: Self, b: Self) -> Self {
        UserRecord {
            uid: i64::oselect(cond, a.uid, b.uid),
            type_rec: u16::oselect(cond, a.type_rec, b.type_rec),
            mark: u16::oselect(cond, a.mark, b.mark),
            last_fetch: u32::oselect(cond, a.last_fetch, b.last_fetch),
            last_send: u32::oselect(cond, a.last_send, b.last_send),
            idx: u32::oselect(cond, a.idx, b.idx),
            message: u64::oselect(cond, a.message, b.message),
        }
    }

    fn oswap(cond: bool, a: &mut Self, b: &mut Self) {
        i64::oswap(cond, &mut a.uid, &mut b.uid);
        u16::oswap(cond, &mut a.type_rec, &mut b.type_rec);
        u16::oswap(cond, &mut a.mark, &mut b.mark);
        u32::oswap(cond, &mut a.last_fetch, &mut b.last_fetch);
        u32::oswap(cond, &mut a.last_send, &mut b.last_send);
        u32::oswap(cond, &mut a.idx, &mut b.idx);
        u64::oswap(cond, &mut a.message, &mut b.message);
    }
}
