use crate::config::JSONConfiguration;
use rsip::headers::{Allow, UntypedHeader, UserAgent};
use rsip::Request;
use rsip::{message::HeadersExt, Header, SipMessage};

pub fn trying(conf: &JSONConfiguration, ip: &String, req: &Request) -> rsip::SipMessage {
    let mut headers: rsip::Headers = Default::default();
    let base_uri = rsip::Uri {
        auth: None,
        host_with_port: rsip::Domain::from(format!(
            "sip:{}@{}:{}",
            &conf.extension, &conf.sip_server, &conf.sip_port
        ))
        .into(),
        ..Default::default()
    };

    headers.push(
        rsip::typed::Via {
            version: rsip::Version::V2,
            transport: rsip::Transport::Udp,
            uri: rsip::Uri {
                host_with_port: (rsip::Domain::from(format!("{}:{}", ip, &conf.sip_port))).into(),
                ..Default::default()
            },
            params: vec![rsip::Param::Branch(rsip::param::Branch::new(
                "z9hG4bKnashds8",
            ))],
        }
        .into(),
    );
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
