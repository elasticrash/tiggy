use crate::{
    commands::invite::Invite,
    composer::communication::{Auth, Call, Trying},
    config::JSONConfiguration,
    sockets::{send, SocketV4},
};
use rsip::{
    header_opt, message::HasHeaders, typed::WwwAuthenticate, Header, Method, Request, Response,
    SipMessage, StatusCode,
};
use std::{
    cell::RefCell,
    collections::VecDeque,
    convert::TryFrom,
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
};

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
    };

    return RefCell::new(OutboundInit {
        inv: invite.clone(),
        msg: invite.clone().ask().to_string(),
    });
}

pub fn outbound_start(
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &RefCell<OutboundInit>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let state_ref = state.borrow();
    send(
        &SocketV4 {
            ip: conf.clone().sip_server,
            port: conf.clone().sip_port,
        },
        state_ref.inv.ask().to_string(),
        socket,
        silent,
        logs,
    );
}

pub fn outbound_request_flow(msg: &SipMessage) -> Method {
    let request = Request::try_from(msg.clone()).unwrap();
    match request.clone().method {
        Method::Ack => todo!(),
        Method::Bye => todo!(),
        Method::Cancel => todo!(),
        Method::Info => todo!(),
        Method::Invite => todo!(),
        Method::Message => todo!(),
        Method::Notify => todo!(),
        Method::Options => todo!(),
        Method::PRack => todo!(),
        Method::Publish => todo!(),
        Method::Refer => todo!(),
        Method::Register => todo!(),
        Method::Subscribe => todo!(),
        Method::Update => todo!(),
    }
}
pub fn outbound_response_flow(
    response: &Response,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &RefCell<OutboundInit>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> StatusCode {
    let mut state_ref = state.borrow_mut();
    let msg: SipMessage = SipMessage::try_from(state_ref.msg.clone()).unwrap();

    match response.status_code {
        StatusCode::Trying => todo!(),
        StatusCode::Unauthorized => {
            let auth = WwwAuthenticate::try_from(
                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            state_ref.inv.nonce = Some(auth.nonce);
            state_ref.inv.set_auth(&conf);
            state_ref.inv.msg = Some(msg);

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
        StatusCode::OK => todo!(),
        _ => todo!(),
    }
    return response.status_code.clone();
}
