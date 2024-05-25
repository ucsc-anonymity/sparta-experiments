#[derive(Debug)]
pub struct Fetch {
    pub receiver: i64,
    pub volume: usize,
}

impl Fetch {
    pub fn new(receiver: i64, volume: usize) -> Self {
        Fetch { receiver, volume }
    }
}
