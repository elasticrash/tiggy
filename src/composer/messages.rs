use crate::composer::header_extension::CustomHeaderExtension::{self};
use crate::config::JSONConfiguration;
use rsip::headers::{Allow, ContentLength, ContentType, ToTypedHeader, UntypedHeader, UserAgent};
use rsip::param::Tag;
use rsip::Request;
use rsip::{message::HeadersExt, Header, SipMessage};
use rsip::{Method, Param};
use std::fmt::Write;
use uuid::Uuid;

pub fn ok(
    conf: &JSONConfiguration,
    ip: &String,
    req: &Request,
    method: Method,
    sdp: bool,
) -> rsip::SipMessage {
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

    headers.push_many(req.headers.get_via_header_array());
    headers.push_many(req.headers.get_record_route_header_array());
    headers.push(req.via_header().unwrap().clone().into());

    if req.max_forwards_header().is_ok() {
        headers.push(req.max_forwards_header().unwrap().clone().into())
    }

    headers.push(req.from_header().unwrap().clone().into());
    let to = req.to_header().unwrap().typed().unwrap();
    let cseq = req.cseq_header().unwrap().typed().unwrap();

    headers.push(
        rsip::typed::To {
            display_name: to.display_name.clone(),
            uri: to.uri,
            params: vec![Param::Tag(Tag::new(Uuid::new_v4().to_string()))],
        }
        .into(),
    );
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
            seq: cseq.seq,
            method,
        }
        .into(),
    );
    headers.push(Header::Allow(Allow::new(
        "ACK,BYE,CANCEL,INFO,INVITE,NOTIFY,OPTIONS,PRACK,REFER,UPDATE",
    )));
    headers.push(Header::UserAgent(UserAgent::new("Tiggy")));
    headers.push(Header::ContentType(ContentType::new("application/sdp")));

    let mut body = "v=0\r\n".to_string();
    let _ = write!(body, "o=tggVCE 226678890 391916715 IN IP4 {}\r\n", ip);
    body.push_str("s=tggVCE Audio Call\r\n");
    let _ = write!(body, "c=IN IP4 {}\r\n", ip);
    body.push_str("t=0 0\r\n");
    body.push_str("m=audio 40024 RTP/AVP 0 8 96\r\n");
    body.push_str("a=rtpmap:0 PCMU/8000\r\n");
    body.push_str("a=rtpmap:8 PCMA/8000\r\n");
    body.push_str("a=rtpmap:96 telephone-event/8000\r\n");
    body.push_str("a=fmtp:96 0-15\r\n");

    headers.push(Header::ContentLength(ContentLength::new(
        body.len().to_string(),
    )));

    let response: SipMessage = rsip::Response {
        status_code: rsip::StatusCode::OK,
        version: rsip::Version::V2,
        headers,
        body: if sdp {
            body.as_bytes().to_vec()
        } else {
            Default::default()
        },
    }
    .into();

    response
}

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
