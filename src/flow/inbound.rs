use crate::{
    composer::{
        communication::{Call, Trying},
        messages::{ok, trying},
        registration::Register,
    },
    config::JSONConfiguration,
    sockets::{send, SocketV4},
};
use rsip::{
    header_opt,
    headers::ToTypedHeader,
    message::HasHeaders,
    message::HeadersExt,
    typed::{Via, WwwAuthenticate},
    Header, Request, Response, SipMessage, StatusCode,
};
use std::{
    cell::RefCell,
    collections::VecDeque,
    convert::TryFrom,
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use super::state::InboundInit;

pub fn inbound_start<'a>(conf: &'a JSONConfiguration, ip: &'a IpAddr) -> RefCell<InboundInit> {
    let register: Register = Register {
        branch: "z9hG4bKnashds8".to_string(),
        extension: conf.extension.to_string(),
        ip: ip.to_string(),
        md5: None,
        password: conf.password.to_string(),
        sip_port: conf.sip_port.to_string(),
        sip_server: conf.sip_server.to_string(),
        username: conf.username.clone(),
        realm: None,
        nonce: None,
        msg: None,
    };

    return RefCell::new(InboundInit {
        reg: register.clone(),
        msg: register.clone().ask().to_string(),
    });
}

pub fn inbound_request_flow(
    msg: &SipMessage,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    ip: &IpAddr,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let request = Request::try_from(msg.clone()).unwrap();
    let via: Via = request.via_header().unwrap().typed().unwrap();

    match request.clone().method {
        rsip::Method::Register => {}
        rsip::Method::Ack => {}
        rsip::Method::Bye => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    &conf,
                    &ip.clone().to_string(),
                    &request,
                    rsip::Method::Bye,
                    false,
                )
                .to_string(),
                socket,
                silent,
                &logs,
            );
        }
        rsip::Method::Cancel => {}
        rsip::Method::Info => {}
        rsip::Method::Invite => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                trying(&conf, &ip.clone().to_string(), &request).to_string(),
                socket,
                silent,
                &logs,
            );
            thread::sleep(Duration::from_secs(1));
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    &conf,
                    &ip.clone().to_string(),
                    &request,
                    rsip::Method::Invite,
                    true,
                )
                .to_string(),
                socket,
                silent,
                logs,
            );
        }
        rsip::Method::Message => {}
        rsip::Method::Notify => {}
        rsip::Method::Options => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    &conf,
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
        rsip::Method::PRack => {}
        rsip::Method::Publish => {}
        rsip::Method::Refer => {}
        rsip::Method::Subscribe => {}
        rsip::Method::Update => {}
    }
}

pub fn inbound_response_flow(
    response: &Response,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &RefCell<InboundInit>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let mut state_ref = state.borrow_mut();
    let msg: SipMessage = SipMessage::try_from(state_ref.msg.clone()).unwrap();

    match response.status_code {
        StatusCode::Unauthorized => {
            let auth = WwwAuthenticate::try_from(
                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            state_ref.reg.nonce = Some(auth.nonce);
            state_ref.reg.realm = Some(auth.realm);
            state_ref.reg.calculate_md5();
            state_ref.reg.msg = Some(msg);

            send(
                &SocketV4 {
                    ip: conf.clone().sip_server,
                    port: conf.clone().sip_port,
                },
                state_ref.reg.attempt().to_string(),
                socket,
                silent,
                logs,
            );
        }
        StatusCode::Trying => {}
        StatusCode::OK => {}
        _ => {}
    }
}
