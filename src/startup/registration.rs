use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

use chrono::prelude::*;
use rsip::SipMessage;
use uuid::Uuid;

use crate::{
    config::JSONConfiguration,
    state::{
        dialogs::{Direction, Register, State, Transactions},
        options::SipOptions,
        transactions::{Transaction, TransactionType},
    },
    transmissions::sockets::{MpscBase, SocketV4},
};

/// Preparation for registering the UA,
/// as well as sending the first unauthorized message
pub fn register_ua(state: &Arc<Mutex<State>>, conf: &JSONConfiguration, ip: &IpAddr) {
    info!("starting registration process");
    let now = Utc::now();

    let mut register = SipOptions {
        branch: format!(
            "z9hG4bK{}{}{}{}{}{}",
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_millis()
        ),
        extension: conf.extension.to_string(),
        ip: ip.to_string(),
        md5: None,
        sip_port: conf.sip_port.to_string(),
        sip_server: conf.sip_server.to_string(),
        username: conf.username.clone(),
        nonce: None,
        msg: None,
        cld: None,
        call_id: Uuid::new_v4().to_string(),
        tag_local: Uuid::new_v4().to_string(),
        tag_remote: None,
        nc: None,
        cnonce: None,
        qop: false,
    };

    let mut transaction: Option<String> = None;
    {
        let state: Arc<Mutex<State>> = state.clone();
        let mut locked_state = state.lock().unwrap();
        let mut registrations = locked_state.get_registrations().unwrap();

        registrations.push(Register {
            call_id: Uuid::new_v4().to_string(),
            diag_type: Direction::Inbound,
            local_tag: Uuid::new_v4().to_string(),
            remote_tag: None,
            transactions: Transactions::new(),
            time: Local::now(),
        });

        for dg in registrations.iter_mut() {
            if matches!(dg.diag_type, Direction::Inbound) {
                let mut transactions = dg.transactions.get_transactions().unwrap();
                let local_transaction = Transaction {
                    object: register.clone(),
                    local: Some(register.set_initial_register()),
                    remote: None,
                    tr_type: TransactionType::Typical,
                };
                transactions.push(local_transaction.clone());

                register.msg = Some(local_transaction.object.set_initial_register());

                let loop_transaction = transactions.last_mut().unwrap();
                transaction = Some(loop_transaction.local.as_ref().unwrap().to_string());
                break;
            }
        }
    }

    if let Some(..) = transaction {
        let reg_state = state.clone();
        let mut locked_state = reg_state.lock().unwrap();
        let channel = locked_state.get_sip_channel().unwrap();

        info!("sending initial registration");

        channel
            .0
            .send(MpscBase {
                event: Some(SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                    bytes: transaction.unwrap().as_bytes().to_vec(),
                }),
                exit: false,
            })
            .unwrap();
    }
}

/// Keep registration alive
pub fn keep_alive(state: Arc<Mutex<State>>, conf: &JSONConfiguration) {
    let mut sip: Option<SipMessage> = None;
    {
        let reg_state: Arc<Mutex<State>> = state.clone();
        let mut locked_state = reg_state.lock().unwrap();
        let mut registrations = locked_state.get_registrations().unwrap();

        if let Some(dg) = registrations.iter_mut().next() {
            let mut transactions = dg.transactions.get_transactions().unwrap();
            let transaction = transactions.last_mut().unwrap();
            sip = Some(transaction.object.keep_alive());
            transaction.local = sip.clone();
        }
    }

    if let Some(..) = sip {
        let locked_socket = state;
        let mut unlocked_socket = locked_socket.lock().unwrap();
        let channel = unlocked_socket.get_sip_channel().unwrap();

        channel
            .0
            .send(MpscBase {
                event: Some(SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                    bytes: sip.unwrap().to_string().as_bytes().to_vec(),
                }),
                exit: false,
            })
            .unwrap();
    }
}

/// Sends the registration again with Expires 0
pub fn unregister_ua(state: Arc<Mutex<State>>, conf: &JSONConfiguration) {
    let mut sip: Option<SipMessage> = None;
    {
        let reg_state: Arc<Mutex<State>> = state.clone();
        let mut locked_state = reg_state.lock().unwrap();
        let mut registrations = locked_state.get_registrations().unwrap();

        if let Some(dg) = registrations.iter_mut().next() {
            let mut transactions = dg.transactions.get_transactions().unwrap();
            let transaction = transactions.last_mut().unwrap();
            sip = Some(transaction.object.unregister());
            transaction.local = sip.clone();
        }
    }

    if let Some(..) = sip {
        let locked_socket = state;
        let mut unlocked_socket = locked_socket.lock().unwrap();
        let channel = unlocked_socket.get_sip_channel().unwrap();

        channel
            .0
            .send(MpscBase {
                event: Some(SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                    bytes: sip.unwrap().to_string().as_bytes().to_vec(),
                }),
                exit: false,
            })
            .unwrap();
    }
}
