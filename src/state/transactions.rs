use super::options::SipOptions;
use rsip::SipMessage;

/// SIP transaction model
#[derive(Clone)]
pub struct Transaction {
    pub tr_type: TransactionType,
    pub local: Option<SipMessage>,
    pub remote: Option<SipMessage>,
    pub object: SipOptions,
}

pub trait Reset {
    fn reset(&mut self);
}

impl Reset for Transaction {
    fn reset(&mut self) {
        self.local = None;
        self.remote = None;
    }
}

#[derive(Clone)]
pub enum TransactionType {
    Typical,
    Invite,
    Ack,
}

impl Transaction {}
