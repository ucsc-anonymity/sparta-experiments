// use crate::types::Request;
use crate::omq::Request;

#[derive(Debug)]
pub struct Send {
    pub receiver: i64,
    pub message: u64,
}

impl Send {
    pub fn new(receiver: i64, message: u64) -> Self {
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