use nom::{
    bytes::complete::{tag, take_until},
    error::Error,
    sequence::tuple,
};
use std::fmt::Write;

pub fn get_base_uri(number: &str, server: &str, port: &str) -> rsip::Uri {
    rsip::Uri {
        auth: None,
        host_with_port: rsip::Domain::from(format!("sip:{}@{}:{}", number, server, port)).into(),
        ..Default::default()
    }
}
use chrono::prelude::*;

pub fn get_via(ip: &str, port: &str) -> rsip::Header {
    let now = Utc::now();

    rsip::typed::Via {
        version: rsip::Version::V2,
        transport: rsip::Transport::Udp,
        uri: rsip::Uri {
            host_with_port: (rsip::Domain::from(format!("{}:{}", ip, port))).into(),
            ..Default::default()
        },
        params: vec![rsip::Param::Branch(rsip::param::Branch::new(format!(
            "z9hG4bK{}{}{}{}{}{}",
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_millis()
        )))],
    }
    .into()
}

pub fn get_from(username: &str, tag: &str, base_uri: rsip::Uri) -> rsip::Header {
    rsip::typed::From {
        display_name: Some(username.to_string()),
        uri: base_uri,
        params: vec![rsip::Param::Tag(rsip::param::Tag::new(tag))],
    }
    .into()
}

pub fn get_to(username: &str, did: &str, server: &str, port: &str) -> rsip::Header {
    rsip::typed::To {
        display_name: Some(username.to_string()),
        uri: rsip::Uri {
            auth: None,
            host_with_port: rsip::Domain::from(format!("sip:{}@{}:{}", did, server, port)).into(),
            ..Default::default()
        },
        params: Default::default(),
    }
    .into()
}

pub fn get_contact(username: &str, did: &str, server: &str, port: &str) -> rsip::Header {
    rsip::typed::Contact {
        display_name: Some(username.to_string()),
        uri: rsip::Uri {
            host_with_port: (rsip::Domain::from(format!("sip:{}@{}:{}", did, server, port))).into(),
            ..Default::default()
        },
        params: Default::default(),
    }
    .into()
}

pub fn get_fake_sdp(ip: &str) -> String {
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

    body
}

pub fn get_remote_tag(hstr: &str) -> &str {
    let (rem, (_, _, _, _)): (&str, (&str, &str, &str, &str)) = tuple((
        take_until::<&str, &str, Error<&str>>(";"),
        tag(";"),
        take_until("="),
        tag("="),
    ))(hstr)
    .unwrap();

    rem
}
