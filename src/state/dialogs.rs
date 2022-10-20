use super::transactions::Transaction;
use chrono::prelude::*;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    net::UdpSocket,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
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
    pub state: Arc<Mutex<Vec<Dialog>>>,
    pub socket: Arc<Mutex<UdpSocket>>,
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

impl Dialogs {
    pub fn new(port: i32) -> Dialogs {
        Dialogs {
            state: Arc::new(Mutex::new(vec![])),
            socket: Arc::new(Mutex::new(
                UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap(),
            )),
        }
    }

    pub fn get_dialogs(&mut self) -> Result<MutexGuard<Vec<Dialog>>, DialogsLockError> {
        Ok(self.state.lock()?)
    }

    pub fn get_socket(&mut self) -> Result<MutexGuard<UdpSocket>, DialogsLockError> {
        Ok(self.socket.lock()?)
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
