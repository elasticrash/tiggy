use crate::transmissions::sockets::MpscBase;

use super::transactions::Transaction;
use chrono::prelude::*;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex, MutexGuard, PoisonError,
    },
};
/// SIP dialog
pub struct Dialog {
    pub diag_type: Direction,
    pub call_id: String,
    pub local_tag: String,
    pub remote_tag: Option<String>,
    pub transactions: Transactions,
    pub time: DateTime<Local>,
}

pub type Register = Dialog;

pub enum Direction {
    Inbound,
    Outbound,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Direction::Inbound => write!(f, "Inbound"),
            Direction::Outbound => write!(f, "Outbound"),
        }
    }
}

/// Collection of State
pub struct State {
    dialog: Arc<Mutex<Vec<Dialog>>>,
    reg: Arc<Mutex<Vec<Register>>>,
    sip: Arc<Mutex<(Sender<UdpCommand>, Receiver<UdpCommand>)>>,
    rtp: Arc<Mutex<(Sender<UdpCommand>, Receiver<UdpCommand>)>>,
}

impl Display for Dialog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.call_id, self.time)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StateLockError {
    FailedToLock,
}

impl Error for StateLockError {}

impl fmt::Display for StateLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> From<PoisonError<T>> for StateLockError {
    fn from(_: PoisonError<T>) -> Self {
        StateLockError::FailedToLock
    }
}

pub type UdpCommand = MpscBase<Vec<u8>>;
type SRUdpCommand = (Sender<UdpCommand>, Receiver<UdpCommand>);

impl State {
    pub fn new(
        (s_a, r_a): (Sender<UdpCommand>, Receiver<UdpCommand>),
        (s_b, r_b): (Sender<UdpCommand>, Receiver<UdpCommand>),
    ) -> State {
        State {
            dialog: Arc::new(Mutex::new(vec![])),
            reg: Arc::new(Mutex::new(vec![])),
            sip: Arc::new(Mutex::new((s_a, r_a))),
            rtp: Arc::new(Mutex::new((s_b, r_b))),
        }
    }

    pub fn get_dialogs(&mut self) -> Result<MutexGuard<Vec<Dialog>>, StateLockError> {
        Ok(self.dialog.lock()?)
    }

    pub fn get_registrations(&mut self) -> Result<MutexGuard<Vec<Register>>, StateLockError> {
        Ok(self.reg.lock()?)
    }

    pub fn get_sip_channel(&mut self) -> Result<MutexGuard<SRUdpCommand>, StateLockError> {
        Ok(self.sip.lock()?)
    }

    #[allow(dead_code)]
    pub fn get_rtp_channel(&mut self) -> Result<MutexGuard<SRUdpCommand>, StateLockError> {
        Ok(self.rtp.lock()?)
    }
}

/// Collection of Transactions
pub struct Transactions {
    pub dialog: Arc<Mutex<Vec<Transaction>>>,
}

#[derive(Debug, Copy, Clone)]
pub enum TransactionsLockError {
    FailedToLock,
}

impl Error for TransactionsLockError {}

impl fmt::Display for TransactionsLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> From<PoisonError<T>> for TransactionsLockError {
    fn from(_: PoisonError<T>) -> Self {
        TransactionsLockError::FailedToLock
    }
}

impl Transactions {
    pub fn new() -> Transactions {
        Transactions {
            dialog: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get_transactions(
        &mut self,
    ) -> Result<MutexGuard<Vec<Transaction>>, TransactionsLockError> {
        Ok(self.dialog.lock()?)
    }
}
