use crate::{
    commands::{helper::get_remote_tag, ok::ok},
    composer::{communication::Auth, header_extension::CustomHeaderExtension},
    config::JSONConfiguration,
    log::{self, print_msg},
    state::{
        dialogs::{Dialog, Dialogs, Direction, Transactions},
        options::{SelfConfiguration, SipOptions, Verbosity},
        transactions::{Transaction, TransactionType},
    },
    transmissions::sockets::{send, SocketV4},
};

use chrono::prelude::*;
use rsip::{
    header_opt,
    message::HasHeaders,
    prelude::{HeadersExt, ToTypedHeader},
    typed::{Via, WwwAuthenticate},
    Header, Method, Request, Response, SipMessage, StatusCode,
};
use std::{
    collections::VecDeque,
    convert::TryFrom,
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

pub fn outbound_configure(
    conf: &JSONConfiguration,
    ip: &IpAddr,
    destination: &str,
    dialog_state: &Arc<Mutex<Dialogs>>,
) {
    let mut locked_state = dialog_state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    let call_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let invite = SipOptions {
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
        username: conf.username.clone(),
        sip_server: conf.sip_server.to_string(),
        sip_port: conf.sip_port.to_string(),
        ip: ip.to_string(),
        msg: None,
        cld: Some(destination.to_string()),
        md5: None,
        nonce: None,
        call_id: call_id.clone(),
        tag_local: Uuid::new_v4().to_string(),
        tag_remote: None,
    };

    let dialog = Dialog {
        call_id: call_id.clone(),
        diag_type: Direction::Outbound,
        local_tag: Uuid::new_v4().to_string(),
        remote_tag: None,
        transactions: Transactions::new(),
        time: Local::now(),
    };
    dialogs.push(dialog);
    for dg in dialogs.iter_mut() {
        if dg.call_id == call_id {
            let mut transactions = dg.transactions.get_transactions().unwrap();
            let mut transaction = Transaction {
                object: invite.clone(),
                local: Some(invite.set_initial_register()),
                remote: None,
                send: 0,
                tr_type: TransactionType::Typical,
            };
            transaction.object.msg = Some(transaction.object.clone().set_initial_invite());
            transactions.push(transaction);
        }
    }
}

/// Sends the Intial invite for an outbound call
// TODO pass identifier for the call
pub fn outbound_start(
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<Dialogs>>,
    vrb: &Verbosity,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let mut locked_state = state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    print_msg(format!("number of dialogs {}: ", dialogs.len()), vrb, logs);

    for dg in dialogs.iter_mut().rev() {
        if matches!(dg.diag_type, Direction::Outbound) {
            let mut tr = dg.transactions.get_transactions().unwrap();

            print_msg(format!("number of transactions {}: ", tr.len()), vrb, logs);

            let mut transaction = tr.last_mut().unwrap();
            transaction.local = transaction.object.set_initial_invite().into();

            send(
                &SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                },
                transaction.local.clone().unwrap().to_string(),
                socket,
                vrb,
                logs,
            );
            break;
        }
    }
}

pub fn process_request_outbound(
    request: &Request,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    _state: &Arc<Mutex<Dialogs>>,
    settings: &mut SelfConfiguration,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let via: Via = request.via_header().unwrap().typed().unwrap();

    log::slog(
        format!("received outbound request, {}", request.method).as_str(),
        logs,
    );

    match request.method {
        Method::Ack => todo!(),
        Method::Bye => {
            settings.flow = Direction::Inbound;
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    conf,
                    &settings.ip.clone().to_string(),
                    request,
                    rsip::Method::Bye,
                    false,
                )
                .to_string(),
                socket,
                &settings.verbosity,
                logs,
            );
        }
        Method::Cancel => todo!(),
        Method::Info => todo!(),
        Method::Invite => todo!(),
        Method::Message => todo!(),
        Method::Notify => todo!(),
        Method::Options => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    conf,
                    &settings.ip.clone().to_string(),
                    request,
                    rsip::Method::Options,
                    false,
                )
                .to_string(),
                socket,
                &settings.verbosity,
                logs,
            );
        }
        Method::PRack => todo!(),
        Method::Publish => todo!(),
        Method::Refer => todo!(),
        Method::Register => todo!(),
        Method::Subscribe => todo!(),
        Method::Update => todo!(),
    }
}
pub fn process_response_outbound(
    response: &Response,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<Dialogs>>,
    settings: &mut SelfConfiguration,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let mut locked_state = state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    log::slog(
        format!("received outbound response, {}", response.status_code).as_str(),
        logs,
    );

    match response.status_code {
        StatusCode::Trying => {}
        StatusCode::Unauthorized => {
            let auth = WwwAuthenticate::try_from(
                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                    .unwrap()
                    .clone(),
            )
            .unwrap();
            for dg in dialogs.iter_mut().rev() {
                if matches!(dg.diag_type, Direction::Outbound) {
                    let mut tr = dg.transactions.get_transactions().unwrap();
                    let mut transaction = tr.last_mut().unwrap();
                    transaction.object.nonce = Some(auth.nonce);
                    transaction.object.set_auth(conf, "INVITE");
                    transaction.object.msg = Some(transaction.local.clone().unwrap());
                    transaction.send = 1;

                    send(
                        &SocketV4 {
                            ip: conf.clone().sip_server,
                            port: conf.clone().sip_port,
                        },
                        transaction.object.push_auth_to_invite().to_string(),
                        socket,
                        &settings.verbosity,
                        logs,
                    );
                    break;
                }
            }
        }
        StatusCode::Ringing => {
            for dg in dialogs.iter_mut().rev() {
                if matches!(dg.diag_type, Direction::Outbound) {
                    let mut tr = dg.transactions.get_transactions().unwrap();
                    let mut transaction = tr.last_mut().unwrap();
                    transaction.remote = Some(SipMessage::Response(response.clone()));
                    break;
                }
            }
        }
        StatusCode::BusyHere => {}
        StatusCode::SessionProgress => {}
        StatusCode::OK => {
            for dg in dialogs.iter_mut().rev() {
                if matches!(dg.diag_type, Direction::Outbound) {
                    let mut tr = dg.transactions.get_transactions().unwrap();
                    let mut transaction = tr.last_mut().unwrap();
                    if transaction.local.is_some() && transaction.remote.is_some() {
                        let hstr = response.clone().to_header().unwrap().to_string();
                        let remote_tag = get_remote_tag(&hstr);
                        let now = Utc::now();

                        let ack = SipOptions {
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
                            username: conf.username.clone(),
                            sip_server: conf.sip_server.to_string(),
                            sip_port: conf.sip_port.to_string(),
                            ip: settings.ip.to_string(),
                            msg: None,
                            cld: transaction.object.cld.clone(),
                            call_id: transaction.object.call_id.clone(),
                            tag_local: transaction.object.tag_local.clone(),
                            tag_remote: Some(remote_tag.to_string()),
                            md5: None,
                            nonce: None,
                        };

                        transaction.send += 1;

                        let via_from_invite =
                            transaction.local.as_ref().unwrap().via_header().unwrap();
                        let cseq_count = transaction.local.as_ref().unwrap().cseq_header().unwrap();
                        let contact = response.contact_header().unwrap();

                        send(
                            &SocketV4 {
                                ip: conf.clone().sip_server,
                                port: conf.clone().sip_port,
                            },
                            ack.create_ack(
                                via_from_invite,
                                response.headers.get_record_route_header_array().clone(),
                                contact,
                                cseq_count,
                            )
                            .to_string(),
                            socket,
                            &settings.verbosity,
                            logs,
                        );
                    }
                    break;
                }
            }
        }
        _ => todo!(),
    }
}
