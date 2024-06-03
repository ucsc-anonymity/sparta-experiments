use std::{cmp::Ordering, time::UNIX_EPOCH};

use otils::Max;

pub const FETCH: u16 = 0;
pub const SEND: u16 = 1;
pub const DUMMY: u16 = 2;

#[derive(Debug)]
pub struct Request {
    pub uid: i32,
    pub req_type: u16,
    pub mark: u16,
    pub volume: usize,
    pub _message: u64,
    pub _dum: [u64; 13],
}

impl Request {
    pub fn new_send(uid: i32, message: u64) -> Self {
        if uid >= i32::MAX {
            panic!("uid: out of bounds.");
        }
        Request {
            uid,
            req_type: SEND,
            mark: 0,
            volume: std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as usize,
            _message: message,
            _dum: [0; 13],
        }
    }

    pub fn new_fetch(uid: i32, volume: usize) -> Self {
        if uid >= i32::MAX {
            panic!("uid: out of bounds.");
        }
        Request {
            uid,
            req_type: FETCH,
            mark: 0,
            volume,
            _message: 0,
            _dum: [0; 13],
        }
    }

    pub fn dummies(uid: i32, len: usize) -> Vec<Self> {
        (0..len)
            .map(|_| Request {
                uid,
                req_type: DUMMY,
                mark: 0,
                volume: 0,
                _message: 0,
                _dum: [0; 13],
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
        !self.is_fetch() && self.mark == 0
    }
}

impl Max for Request {
    fn maximum() -> Self {
        Request {
            uid: i32::MAX,
            req_type: DUMMY,
            mark: 0,
            volume: 0,
            _message: 0,
            _dum: [0; 13],
        }
    }
}

// impl From<Send> for Request {
//     fn from(s: Send) -> Self {
//         Request {
//             receiver: s.receiver,
//             req_type: SEND,
//             mark: 0,
//             volume: std::time::SystemTime::now()
//                 .duration_since(UNIX_EPOCH)
//                 .unwrap()
//                 .as_nanos() as usize,
//             message: s.message,
//         }
//     }
// }

// impl From<Fetch> for Request {
//     fn from(f: Fetch) -> Self {
//         Request {
//             receiver: f.receiver,
//             req_type: FETCH,
//             mark: 0,
//             volume: f.volume,
//             message: 0,
//         }
//     }
// }

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid && self.req_type == other.req_type && self.volume == other.volume
    }
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let receiver_ord = self.uid.partial_cmp(&other.uid);
        let type_ord = self.req_type.partial_cmp(&other.req_type);
        let vol_ord = self.volume.partial_cmp(&other.volume);
        match receiver_ord {
            Some(Ordering::Equal) => match type_ord {
                Some(Ordering::Equal) => vol_ord,
                x => x,
            },
            x => x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let s_less: Request = Request::new_send(0, 0);
        let f_less: Request = Request::new_fetch(0, 0);

        assert!(s_less == s_less);
        assert!(f_less != s_less);
        assert!(f_less == f_less);
    }

    #[test]
    fn test_ord() {
        let s_less: Request = Request::new_send(0, 0);
        let s_great: Request = Request::new_send(1, 0);
        let f_less: Request = Request::new_fetch(0, 0);
        let f_great: Request = Request::new_fetch(1, 0);

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
