#[derive(Clone)]
pub struct Transaction {
    pub tr_type: TransactionType,
    pub local: bool,
    pub remote: bool,
}

pub trait Reset {
    fn reset(&mut self);
}

impl Reset for Transaction {
    fn reset(&mut self) {
        self.local = false;
        self.remote = false;
    }
}

#[derive(Clone)]
pub enum TransactionType {
    Typical,
    Invite,
    Ack
}