use crate::omq::{Fetch, Send};
use otils::ObliviousOps;
use std::{cmp::Ordering, time::UNIX_EPOCH};

pub const FETCH: u32 = 0;
pub const SEND: u32 = 1;
pub const DUMMY: u32 = 2;

#[derive(Debug, Clone, Copy)]
pub struct Request {
    pub receiver: i64,
    pub req_type: u32,
    pub mark: u32,
    pub volume: usize,
    pub message: u64,
}

impl Request {
    pub fn dummies(receiver: i64, len: usize) -> Vec<Self> {
        (0..len)
            .map(|_| Request {
                receiver: receiver,
                req_type: DUMMY,
                mark: 0,
                volume: 0,
                message: 0,
            })
            .collect()
    }

    pub fn is_fetch(&self) -> bool {
        self.req_type == FETCH
    }

    pub fn should_deliver(&self) -> bool {
        !self.is_fetch() && self.mark == 1
    }

    pub fn should_defer(&self) -> bool {
        self.receiver >= 0 && !self.is_fetch() && self.mark == 0 // this is also hacky
    }
}

impl From<Send> for Request {
    fn from(s: Send) -> Self {
        Request {
            receiver: s.receiver,
            req_type: SEND,
            mark: 0,
            volume: std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as usize,
            message: s.message,
        }
    }
}

impl From<Fetch> for Request {
    fn from(f: Fetch) -> Self {
        Request {
            receiver: f.receiver,
            req_type: FETCH,
            mark: 0,
            volume: f.volume,
            message: 0,
        }
    }
}

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.receiver == other.receiver && self.req_type == other.req_type
    }
}

// TODO this is not oblivious, just add the comparators back into otils later.
impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let receiver_ord = self.receiver.partial_cmp(&other.receiver);
        let type_ord = self.req_type.partial_cmp(&other.req_type);
        let vol_ord = self.volume.partial_cmp(&other.volume);
        match receiver_ord {
            Some(Ordering::Equal) => match type_ord {
                Some(Ordering::Equal) => vol_ord,
                Some(x) => Some(x),
                None => None,
            },
            Some(x) => Some(x),
            None => None,
        }
    }
}

impl ObliviousOps for Request {
    fn oselect(cond: bool, a: Self, b: Self) -> Self {
        Request {
            receiver: i64::oselect(cond, a.receiver, b.receiver),
            req_type: u32::oselect(cond, a.req_type, b.req_type),
            mark: u32::oselect(cond, a.mark, b.mark),
            volume: usize::oselect(cond, a.volume, b.volume),
            message: u64::oselect(cond, a.message, b.message),
        }
    }

    fn oswap(cond: bool, a: &mut Self, b: &mut Self) {
        i64::oswap(cond, &mut a.receiver, &mut b.receiver);
        u32::oswap(cond, &mut a.req_type, &mut b.req_type);
        u32::oswap(cond, &mut a.mark, &mut b.mark);
        usize::oswap(cond, &mut a.volume, &mut b.volume);
        u64::oswap(cond, &mut a.message, &mut b.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let s_less: Request = Send::new(0, 0).into();
        let f_less: Request = Fetch::new(0, 0).into();

        assert!(s_less == s_less);
        assert!(f_less != s_less);
        assert!(f_less == f_less);
    }

    #[test]
    fn test_ord() {
        let s_less: Request = Send::new(0, 0).into();
        let s_great: Request = Send::new(1, 0).into();
        let f_less: Request = Fetch::new(0, 0).into();
        let f_great: Request = Fetch::new(1, 0).into();

        assert!(s_less < s_great);
        assert!(s_great > s_less);
        assert!(s_great == s_great);

        assert!(f_less < f_great);
        assert!(f_great > f_less);
        assert!(f_great == f_great);

        assert!(f_less < s_less);
        assert!(f_less < s_great);
        assert!(f_great < s_great);
        assert!(f_great > s_less);
    }
}
