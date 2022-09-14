use crate::{
    commands::{ack::Ack, invite::Invite, ok::ok},
    composer::communication::{Auth, Call, Start, Trying},
    config::JSONConfiguration,
    log,
    state::transactions::{Reset, Transaction, TransactionType},
    transmissions::sockets::{send, SocketV4},
};
use nom::{
    bytes::complete::{tag, take_until},
    error::Error,
    sequence::tuple,
};
use rsip::{
    header_opt,
    message::HasHeaders,
    prelude::{HeadersExt, ToTypedHeader},
    typed::{Via, WwwAuthenticate},
    Header, Method, Request, Response, SipMessage, StatusCode,
};
use std::{
    cell::RefCell,
    collections::VecDeque,
    convert::TryFrom,
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use super::state::OutboundInit;

pub fn outbound_configure(conf: &JSONConfiguration, ip: &IpAddr) -> RefCell<OutboundInit> {
    let invite: Invite = Invite {
        extension: conf.extension.to_string(),
        username: conf.username.clone(),
        sip_server: conf.sip_server.to_string(),
        sip_port: conf.sip_port.to_string(),
        ip: ip.to_string(),
        msg: None,
        cld: Some(conf.username.clone()),
        md5: None,
        nonce: None,
        call_id: Uuid::new_v4().to_string(),
        tag_local: Uuid::new_v4().to_string(),
        tag_remote: None,
    };

    RefCell::new(OutboundInit {
        inv: invite.clone(),
        msg: invite.init("".to_string()).to_string(),
        transaction: Transaction {
            tr_type: TransactionType::Invite,
            local: false,
            remote: false,
        },
    })
}

pub fn outbound_start(
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &RefCell<OutboundInit>,
    silent: bool,
    destination: String,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let mut state_ref = state.borrow_mut();
    state_ref.transaction.local = true;
    send(
        &SocketV4 {
            ip: conf.clone().sip_server,
            port: conf.clone().sip_port,
        },
        state_ref.inv.init(destination).to_string(),
        socket,
        silent,
        logs,
    );
}

pub fn outbound_request_flow(
    msg: &SipMessage,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    ip: &IpAddr,
    _state: &RefCell<OutboundInit>,
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
    state: &RefCell<OutboundInit>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> StatusCode {
    let mut state_ref = state.borrow_mut();
    let msg: SipMessage = SipMessage::try_from(state_ref.msg.clone()).unwrap();

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

            state_ref.inv.nonce = Some(auth.nonce);
            state_ref.inv.set_auth(conf);
            state_ref.inv.msg = Some(msg);
            state_ref.transaction.local = true;

            send(
                &SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                },
                state_ref.inv.attempt().to_string(),
                socket,
                silent,
                logs,
            );
        }
        StatusCode::Ringing => {}
        StatusCode::OK => {
            if state_ref.transaction.local {
                state_ref.transaction.remote = true;
            }

            if state_ref.transaction.local && state_ref.transaction.remote {
                let hstr = response.clone().to_header().unwrap().to_string();
                let (rem, (_, _, _, _)): (&str, (&str, &str, &str, &str)) =
                    tuple((
                        take_until::<&str, &str, Error<&str>>(";"),
                        tag(";"),
                        take_until("="),
                        tag("="),
                    ))(&*hstr)
                    .unwrap();

                let ack: Ack = Ack {
                    extension: conf.extension.to_string(),
                    username: conf.username.clone(),
                    sip_server: conf.sip_server.to_string(),
                    sip_port: conf.sip_port.to_string(),
                    ip: ip.to_string(),
                    msg: None,
                    cld: state_ref.inv.cld.clone(),
                    call_id: state_ref.inv.call_id.clone(),
                    tag_local: state_ref.inv.tag_local.clone(),
                    tag_remote: rem.to_string(),
                };

                send(
                    &SocketV4 {
                        ip: conf.clone().sip_server,
                        port: conf.clone().sip_port,
                    },
                    ack.set().to_string(),
                    socket,
                    silent,
                    logs,
                );
            }

            state_ref.transaction.reset();
        }
        _ => todo!(),
    }
    response.status_code.clone()
}
