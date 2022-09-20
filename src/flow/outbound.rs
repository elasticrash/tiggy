use crate::{
    commands::{helper::get_remote_tag, ok::ok},
    composer::communication::Auth,
    config::JSONConfiguration,
    log,
    state::{
        dialogs::{Dialog, Dialogs, Direction, Transactions},
        options::SipOptions,
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
    state: &Arc<Mutex<Dialogs>>,
) {
    let invite = SipOptions {
        branch: "z9hG4bKnashds8".to_string(),
        extension: conf.extension.to_string(),
        username: conf.username.clone(),
        sip_server: conf.sip_server.to_string(),
        sip_port: conf.sip_port.to_string(),
        ip: ip.to_string(),
        msg: None,
        cld: Some(destination.to_string()),
        md5: None,
        nonce: None,
        call_id: Uuid::new_v4().to_string(),
        tag_local: Uuid::new_v4().to_string(),
        tag_remote: None,
    };

    let mut locked_state = state.lock().unwrap();

    locked_state.state.lock().unwrap().push(Dialog {
        call_id: Uuid::new_v4().to_string(),
        diag_type: Direction::Outbound,
        local_tag: Uuid::new_v4().to_string(),
        remote_tag: None,
        transactions: Transactions::new(),
        time: Local::now(),
    });

    let mut dialogs = locked_state.get_dialogs().unwrap();

    for dg in dialogs.iter_mut() {
        if matches!(dg.diag_type, Direction::Outbound) {
            let mut transactions = dg.transactions.get_transactions().unwrap();
            transactions.push(Transaction {
                object: invite.clone(),
                local: Some(invite.set_initial_invite()),
                remote: None,
                send: 0,
                tr_type: TransactionType::Typical,
            })
        }
    }
}

pub fn outbound_start(
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<Dialogs>>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let mut locked_state = state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    for dg in dialogs.iter_mut() {
        if matches!(dg.diag_type, Direction::Outbound) {
            let mut tr = dg.transactions.get_transactions().unwrap();
            let mut transaction = tr.last_mut().unwrap();
            transaction.local = transaction.object.set_initial_invite().into();

            send(
                &SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                },
                transaction.local.clone().unwrap().to_string(),
                socket,
                silent,
                logs,
            );
        }
    }
}

pub fn outbound_request_flow(
    msg: &SipMessage,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    ip: &IpAddr,
    _state: &Arc<Mutex<Dialogs>>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> Method {
    let request = Request::try_from(msg.clone()).unwrap();
    let via: Via = request.via_header().unwrap().typed().unwrap();

    log::slog(
        format!("received outbound request, {}", request.method).as_str(),
        logs,
    );

    match request.method {
        Method::Ack => todo!(),
        Method::Bye => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    conf,
                    &ip.clone().to_string(),
                    &request,
                    rsip::Method::Bye,
                    false,
                )
                .to_string(),
                socket,
                silent,
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
                    &ip.clone().to_string(),
                    &request,
                    rsip::Method::Options,
                    false,
                )
                .to_string(),
                socket,
                silent,
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
    request.method
}
pub fn outbound_response_flow(
    response: &Response,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    ip: &IpAddr,
    state: &Arc<Mutex<Dialogs>>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> StatusCode {
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
            for dg in dialogs.iter_mut() {
                if matches!(dg.diag_type, Direction::Outbound) {
                    let mut tr = dg.transactions.get_transactions().unwrap();
                    let mut transaction = tr.last_mut().unwrap();
                    transaction.object.nonce = Some(auth.nonce.clone());
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
                        silent,
                        logs,
                    );
                }
            }
        }
        StatusCode::Ringing => {
            for dg in dialogs.iter_mut() {
                if matches!(dg.diag_type, Direction::Outbound) {
                    let mut tr = dg.transactions.get_transactions().unwrap();
                    let mut transaction = tr.last_mut().unwrap();
                    transaction.remote = Some(SipMessage::Response(response.clone()));
                }
            }
        }
        StatusCode::OK => {
            for dg in dialogs.iter_mut() {
                if matches!(dg.diag_type, Direction::Outbound) {
                    let mut tr = dg.transactions.get_transactions().unwrap();
                    let mut transaction = tr.last_mut().unwrap();
                    if transaction.local.is_some() && transaction.remote.is_some() {
                        let hstr = response.clone().to_header().unwrap().to_string();
                        let remote_tag = get_remote_tag(&hstr);

                        let ack = SipOptions {
                            branch: "z9hG4bKnashds8".to_string(),
                            extension: conf.extension.to_string(),
                            username: conf.username.clone(),
                            sip_server: conf.sip_server.to_string(),
                            sip_port: conf.sip_port.to_string(),
                            ip: ip.to_string(),
                            msg: None,
                            cld: transaction.object.cld.clone(),
                            call_id: transaction.object.call_id.clone(),
                            tag_local: transaction.object.tag_local.clone(),
                            tag_remote: Some(remote_tag.to_string()),
                            md5: None,
                            nonce: None,
                        };

                        transaction.send += 1;
                        send(
                            &SocketV4 {
                                ip: conf.clone().sip_server,
                                port: conf.clone().sip_port,
                            },
                            ack.create_ack().to_string(),
                            socket,
                            silent,
                            logs,
                        );
                    }
                }
            }
        }
        _ => todo!(),
    }
    response.status_code.clone()
}
