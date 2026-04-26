use crate::transmissions::sockets::{MpscBase, SocketV4};

use super::transactions::Transaction;
use chrono::prelude::*;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    net::IpAddr,
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

#[derive(Clone)]
pub struct RtpTarget {
    pub ip: IpAddr,
    pub port: u16,
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
    rtp_active: bool,
    pending_rtp_target: Option<RtpTarget>,
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

pub type UdpCommand = MpscBase<SocketV4>;
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
            rtp_active: false,
            pending_rtp_target: None,
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

    pub fn is_rtp_active(&self) -> bool {
        self.rtp_active
    }

    pub fn set_rtp_active(&mut self, active: bool) {
        self.rtp_active = active;
    }

    pub fn set_pending_rtp_target(&mut self, target: Option<RtpTarget>) {
        self.pending_rtp_target = target;
    }

    pub fn clear_pending_rtp_target(&mut self) {
        self.pending_rtp_target = None;
    }

    pub fn take_pending_rtp_target(&mut self) -> Option<RtpTarget> {
        self.pending_rtp_target.take()
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
