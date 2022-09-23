use crate::config::JSONConfiguration;
use rsip::headers::{Allow, UntypedHeader, UserAgent};
use rsip::Request;
use rsip::{message::HeadersExt, Header, SipMessage};

use super::helper::{get_base_uri, get_via};

pub fn trying(conf: &JSONConfiguration, ip: &str, req: &Request) -> rsip::SipMessage {
    let mut headers: rsip::Headers = Default::default();
    let base_uri = get_base_uri(
        &conf.extension,
        &conf.sip_server,
        &conf.sip_port.to_string(),
    );

    headers.push(get_via(ip, &conf.sip_port.to_string()));
    headers.push(req.max_forwards_header().unwrap().clone().into());
    headers.push(req.from_header().unwrap().clone().into());
    headers.push(req.to_header().unwrap().clone().into());
    headers.push(
        rsip::typed::Contact {
            display_name: Some(conf.username.to_string()),
            uri: base_uri,
            params: Default::default(),
        }
        .into(),
    );
    headers.push(req.call_id_header().unwrap().clone().into());
    headers.push(
        rsip::typed::CSeq {
            seq: 1,
            method: rsip::Method::Invite,
        }
        .into(),
    );
    headers.push(Header::Allow(Allow::new(
        "ACK,BYE,CANCEL,INFO,INVITE,NOTIFY,OPTIONS,PRACK,REFER,UPDATE",
    )));
    headers.push(Header::UserAgent(UserAgent::new("Tiggy")));

    let response: SipMessage = rsip::Response {
        status_code: rsip::StatusCode::Trying,
        version: rsip::Version::V2,
        headers,
        body: Default::default(),
    }
    .into();

    response
}
