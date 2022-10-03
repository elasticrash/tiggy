use std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
};

use chrono::prelude::*;
use uuid::Uuid;

use crate::{
    config::JSONConfiguration,
    state::{
        dialogs::{Dialog, Dialogs, Direction, Transactions},
        options::{SelfConfiguration, SipOptions, Verbosity},
        transactions::{Transaction, TransactionType},
    },
    transmissions::sockets::{send, SocketV4}, log::MTLogs,
};

/// preparation for registering the UA,
/// as well as sending the first unauthorized message
pub fn register_ua(
    dialog_state: &Arc<Mutex<Dialogs>>,
    conf: &JSONConfiguration,
    socket: &mut UdpSocket,
    settings: &mut SelfConfiguration,
    logs: &MTLogs,
) {
    let mut locked_state = dialog_state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();
    let now = Utc::now();

    let register = SipOptions {
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
        ip: settings.ip.to_string(),
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
                tr_type: TransactionType::Typical,
            });

            let transaction = transactions.last_mut().unwrap();

            send(
                &SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                },
                socket,
                transaction.local.as_ref().unwrap().to_string(),
                &settings.verbosity,
                logs,
            );
        }
    }
}

pub fn unregister_ua(
    dialog_state: &Arc<Mutex<Dialogs>>,
    conf: &JSONConfiguration,
    socket: &mut UdpSocket,
    vrb: &Verbosity,
    logs: &MTLogs,
) {
    let mut locked_state = dialog_state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    for dg in dialogs.iter_mut() {
        let mut transactions = dg.transactions.get_transactions().unwrap();
        let transaction = transactions.first_mut().unwrap();
        let unregister = transaction.object.unregister();

        send(
            &SocketV4 {
                ip: conf.clone().sip_server,
                port: conf.clone().sip_port,
            },
            socket,
            unregister.to_string(),
            vrb,
            logs,
        );
    }
}
