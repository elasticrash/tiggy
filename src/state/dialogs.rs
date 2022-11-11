use crate::transmissions::sockets::{MpscBase, SocketV4};

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

/// Collection of Dialogs
pub struct Dialogs {
    state: Arc<Mutex<Vec<Dialog>>>,
    sip: Arc<Mutex<(Sender<UdpCommand>, Receiver<UdpCommand>)>>,
    rtp: Arc<Mutex<(Sender<UdpCommand>, Receiver<UdpCommand>)>>,
}

impl Display for Dialog {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.call_id, self.time)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DialogsLockError {
    FailedToLock,
}

impl Error for DialogsLockError {}

impl fmt::Display for DialogsLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> From<PoisonError<T>> for DialogsLockError {
    fn from(_: PoisonError<T>) -> Self {
        DialogsLockError::FailedToLock
    }
}

pub type UdpCommand = MpscBase<SocketV4>;
type SRUdpCommand = (Sender<UdpCommand>, Receiver<UdpCommand>);

impl Dialogs {
    pub fn new(
        (s_a, r_a): (Sender<UdpCommand>, Receiver<UdpCommand>),
        (s_b, r_b): (Sender<UdpCommand>, Receiver<UdpCommand>),
    ) -> Dialogs {
        Dialogs {
            state: Arc::new(Mutex::new(vec![])),
            sip: Arc::new(Mutex::new((s_a, r_a))),
            rtp: Arc::new(Mutex::new((s_b, r_b))),
        }
    }

    pub fn get_dialogs(&mut self) -> Result<MutexGuard<Vec<Dialog>>, DialogsLockError> {
        Ok(self.state.lock()?)
    }

    pub fn get_sip_channel(&mut self) -> Result<MutexGuard<SRUdpCommand>, DialogsLockError> {
        Ok(self.sip.lock()?)
    }

    pub fn get_rtp_channel(&mut self) -> Result<MutexGuard<SRUdpCommand>, DialogsLockError> {
        Ok(self.rtp.lock()?)
    }
}

/// Collection of Transactions
pub struct Transactions {
    pub state: Arc<Mutex<Vec<Transaction>>>,
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
            state: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get_transactions(
        &mut self,
    ) -> Result<MutexGuard<Vec<Transaction>>, TransactionsLockError> {
        Ok(self.state.lock()?)
    }
}
