use crate::transmissions::sockets::SocketV4;

use super::transactions::Transaction;
use chrono::prelude::*;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{mpsc::Sender, Arc, Mutex, MutexGuard, PoisonError},
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
    pub sender: Arc<Mutex<Sender<SocketV4>>>,
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
    pub fn new(rs: Sender<SocketV4>) -> Dialogs {
        Dialogs {
            state: Arc::new(Mutex::new(vec![])),
            sender: Arc::new(Mutex::new(rs)),
        }
    }

    pub fn get_dialogs(&mut self) -> Result<MutexGuard<Vec<Dialog>>, DialogsLockError> {
        Ok(self.state.lock()?)
    }

    pub fn get_sender(&mut self) -> Result<MutexGuard<Sender<SocketV4>>, DialogsLockError> {
        Ok(self.sender.lock()?)
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
