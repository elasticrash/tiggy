use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use chrono::prelude::*;

use super::transactions::Transaction;

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

pub struct Dialogs {
    pub state: Arc<Mutex<Vec<Dialog>>>,
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
    pub fn new() -> Dialogs {
        Dialogs {
            state: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get_dialogs(&mut self) -> Result<MutexGuard<Vec<Dialog>>, DialogsLockError> {
        Ok(self.state.lock()?)
    }

    pub fn patch(&mut self, patch: impl Fn(&mut Vec<Dialog>)) -> Result<(), DialogsLockError> {
        let mut guard = self.state.lock()?;
        patch(&mut guard);
        Ok(())
    }
}

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

    pub fn patch(
        &mut self,
        patch: impl Fn(&mut Vec<Transaction>),
    ) -> Result<(), TransactionsLockError> {
        let mut guard = self.state.lock()?;
        patch(&mut guard);
        Ok(())
    }
}
