use crate::composer::header_extension::CustomHeaderExtension::{self};
use crate::config::JSONConfiguration;
use rsip::headers::{Allow, ContentLength, ContentType, ToTypedHeader, UntypedHeader, UserAgent};
use rsip::param::Tag;
use rsip::Request;
use rsip::{message::HeadersExt, Header, SipMessage};
use rsip::{Method, Param};

use super::helper::{get_base_uri, get_fake_sdp, get_remote_tag};

pub fn ok(
    conf: &JSONConfiguration,
    ip: &str,
    req: &Request,
    method: Method,
    sdp: bool,
) -> rsip::SipMessage {
    let mut headers: rsip::Headers = Default::default();
    let base_uri = get_base_uri(
        &conf.extension,
        &conf.sip_server,
        &conf.sip_port.to_string(),
    );

    headers.push_many(req.headers.get_via_header_array());
    headers.push_many(req.headers.get_record_route_header_array());

    if req.max_forwards_header().is_ok() {
        headers.push(req.max_forwards_header().unwrap().clone().into())
    }

    headers.push(req.from_header().unwrap().clone().into());
    let to = req.to_header().unwrap().typed().unwrap();
    let cseq = req.cseq_header().unwrap().typed().unwrap();

    let hstr = req.clone().from_header().unwrap().to_string();
    let remote_tag = get_remote_tag(&hstr);

    headers.push(
        rsip::typed::To {
            display_name: to.display_name.clone(),
            uri: to.uri,
            params: vec![Param::Tag(Tag::new(remote_tag))],
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

    let fake_sdp_body = get_fake_sdp(ip);

    headers.push(Header::ContentLength(ContentLength::new(
        fake_sdp_body.len().to_string(),
    )));

    let response: SipMessage = rsip::Response {
        status_code: rsip::StatusCode::OK,
        version: rsip::Version::V2,
        headers,
        body: if sdp {
            fake_sdp_body.as_bytes().to_vec()
        } else {
            Default::default()
        },
    }
    .into();

    response
}
