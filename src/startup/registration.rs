use std::{
    collections::VecDeque,
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
};

use chrono::prelude::*;
use uuid::Uuid;

use crate::{
    config::JSONConfiguration,
    state::{
        dialogs::{Dialog, Dialogs, Direction, Transactions},
        options::SipOptions,
        transactions::{Transaction, TransactionType},
    },
    transmissions::sockets::{send, SocketV4},
};

/// preparation for registering the UA,
/// as well as sending the first unauthorized message
pub fn register_ua(
    dialog_state: &Arc<Mutex<Dialogs>>,
    conf: &JSONConfiguration,
    ip: &IpAddr,
    socket: &mut UdpSocket,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let mut locked_state = dialog_state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    let register = SipOptions {
        branch: "z9hG4bKtiggyD".to_string(),
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
    };

    dialogs.push(Dialog {
        call_id: Uuid::new_v4().to_string(),
        diag_type: Direction::Inbound,
        local_tag: Uuid::new_v4().to_string(),
        remote_tag: None,
        transactions: Transactions::new(),
        time: Local::now(),
    });

    for dg in dialogs.iter_mut() {
        if matches!(dg.diag_type, Direction::Inbound) {
            let mut transactions = dg.transactions.get_transactions().unwrap();
            transactions.push(Transaction {
                object: register.clone(),
                local: Some(register.set_initial_register()),
                remote: None,
                send: 0,
                tr_type: TransactionType::Typical,
            });

            let transaction = transactions.last_mut().unwrap();

            send(
                &SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                },
                transaction.local.as_ref().unwrap().to_string(),
                socket,
                silent,
                logs,
            );
        }
    }
}
