use std::{cell::RefCell, convert::TryFrom, net::IpAddr};

use rsip::{Method, Request, Response, SipMessage, StatusCode};

use crate::{commands::invite::Invite, config::JSONConfiguration};

use super::state::OutboundInit;

pub fn outbound_start(conf: &JSONConfiguration, ip: &IpAddr) -> RefCell<OutboundInit> {
    let invite: Invite = Invite {
        extension: conf.extension.to_string(),
        username: conf.username.clone(),
        sip_server: conf.sip_server.to_string(),
        sip_port: conf.sip_port.to_string(),
        ip: ip.to_string(),
        msg: None,
        cld: None,
    };

    return RefCell::new(OutboundInit {
        inv: invite.clone(),
    });
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
pub fn outbound_response_flow(response: &Response, state: &RefCell<OutboundInit>) -> StatusCode {

    match response.status_code {
        StatusCode::Trying => todo!(),
        StatusCode::Unauthorized => todo!(),
        StatusCode::OK => todo!(),
        _ => todo!(),
    }
    return response.status_code.clone();
}
