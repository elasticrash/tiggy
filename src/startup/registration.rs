use std::sync::{Arc, Mutex};

use chrono::prelude::*;
use rsip::SipMessage;
use uuid::Uuid;

use crate::{
    config::JSONConfiguration,
    state::{
        dialogs::{Dialog, Dialogs, Direction, Transactions},
        options::{SelfConfiguration, SipOptions},
        transactions::{Transaction, TransactionType},
    },
    transmissions::sockets::SocketV4,
};

/// Preparation for registering the UA,
/// as well as sending the first unauthorized message
pub fn register_ua(
    dialog_state: &Arc<Mutex<Dialogs>>,
    conf: &JSONConfiguration,
    settings: &mut SelfConfiguration,
) {
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

    let mut transaction: Option<String> = None;
    {
        let state: Arc<Mutex<Dialogs>> = dialog_state.clone();
        let mut locked_state = state.lock().unwrap();
        let mut dialogs = locked_state.get_dialogs().unwrap();

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

                let loop_transaction = transactions.last_mut().unwrap();
                transaction = Some(loop_transaction.local.as_ref().unwrap().to_string());
                break;
            }
        }
    }

    if let Some(..) = transaction {
        let state = dialog_state.clone();
        let mut locked_state = state.lock().unwrap();
        let tx = locked_state.get_sender().unwrap();
        tx.send(SocketV4 {
            ip: conf.clone().sip_server,
            port: conf.clone().sip_port,
            bytes: transaction.unwrap().as_bytes().to_vec(),
        })
        .unwrap();
    }
}

/// Sends the registration again with Expires 0
pub fn unregister_ua(dialog_state: &Arc<Mutex<Dialogs>>, conf: &JSONConfiguration) {
    let mut sip: Option<SipMessage> = None;
    {
        let state: Arc<Mutex<Dialogs>> = dialog_state.clone();
        let mut locked_state = state.lock().unwrap();
        let mut dialogs = locked_state.get_dialogs().unwrap();

        if let Some(dg) = dialogs.iter_mut().next() {
            let mut transactions = dg.transactions.get_transactions().unwrap();
            let transaction = transactions.first_mut().unwrap();
            sip = Some(transaction.object.unregister());
        }
    }

    if let Some(..) = sip {
        let locked_socket = dialog_state.clone();
        let mut unlocked_socket = locked_socket.lock().unwrap();
        let tx = unlocked_socket.get_sender().unwrap();
        tx.send(SocketV4 {
            ip: conf.clone().sip_server,
            port: conf.clone().sip_port,
            bytes: sip.unwrap().to_string().as_bytes().to_vec(),
        })
        .unwrap();
    }
}
