// use crate::types::Request;
use crate::Request;

#[derive(Debug)]
pub struct Send {
    pub receiver: i64,
    pub message: u64,
}

impl Send {
    pub fn new(receiver: i64, message: u64) -> Self {
        if receiver == i64::MAX {
            panic!("Key out of bounds: {}", receiver);
        }
        Send { receiver, message }
    }
}

impl From<Request> for Send {
    fn from(r: Request) -> Self {
        Self {
            receiver: r.receiver,
            message: r.message,
        }
    }
}
