use crate::{
    commands::{ok::ok, register::Register, trying::trying},
    composer::communication::{Auth, Start, Trying},
    config::JSONConfiguration,
    log,
    transmissions::sockets::{send, SocketV4},
};
use rsip::{
    header_opt,
    headers::ToTypedHeader,
    message::HasHeaders,
    message::HeadersExt,
    typed::{Via, WwwAuthenticate},
    Header, Method, Request, Response, SipMessage, StatusCode,
};
use uuid::Uuid;
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

    RefCell::new(InboundInit {
        reg: register.clone(),
        msg: register.set().to_string(),
    })
}

pub fn inbound_request_flow(
    msg: &SipMessage,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    ip: &IpAddr,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> Method {
    let request = Request::try_from(msg.clone()).unwrap();
    let via: Via = request.via_header().unwrap().typed().unwrap();

    log::slog(
        format!("received inbound request, {}", request.method).as_str(),
        logs,
    );

    match request.method {
        rsip::Method::Register => {}
        rsip::Method::Ack => {}
        rsip::Method::Bye => {
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
        rsip::Method::Cancel => {}
        rsip::Method::Info => {}
        rsip::Method::Invite => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                trying(conf, &ip.clone().to_string(), &request).to_string(),
                socket,
                silent,
                logs,
            );
            thread::sleep(Duration::from_secs(1));
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    conf,
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
        rsip::Method::PRack => {}
        rsip::Method::Publish => {}
        rsip::Method::Refer => {}
        rsip::Method::Subscribe => {}
        rsip::Method::Update => {}
    }
    request.method
}

pub fn inbound_response_flow(
    response: &Response,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &RefCell<InboundInit>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> StatusCode {
    let mut state_ref = state.borrow_mut();
    let msg: SipMessage = SipMessage::try_from(state_ref.msg.clone()).unwrap();

    log::slog(
        format!("received inbound response, {}", response.status_code).as_str(),
        logs,
    );

    match response.status_code {
        StatusCode::Unauthorized => {
            let auth = WwwAuthenticate::try_from(
                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            state_ref.reg.nonce = Some(auth.nonce);
            state_ref.reg.set_auth(conf);
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
    response.status_code.clone()
}
