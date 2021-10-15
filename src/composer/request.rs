use crate::config::JSONConfiguration;
use rsip::{
    headers::{auth, CallId, UntypedHeader, UserAgent},
    message::HeadersExt,
    typed::WwwAuthenticate,
    Header, SipMessage,
};
use uuid::Uuid;

pub fn unauthorized_register_request(conf: &JSONConfiguration, ip: &String) -> SipMessage {
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
    headers.push(
        rsip::typed::From {
            display_name: Some(format!("{}", conf.username.to_string(),)),
            uri: base_uri.clone(),
            params: vec![rsip::Param::Tag(rsip::param::Tag::new("a73kszlfl"))],
        }
        .into(),
    );
    headers.push(rsip::headers::MaxForwards::from(70).into());
    headers.push(
        rsip::typed::To {
            display_name: Some(format!("{}", conf.username.to_string(),)),
            uri: base_uri.clone(),
            params: Default::default(),
        }
        .into(),
    );
    headers
        .push(Header::CallId(CallId::new(format!("{}@sippy", Uuid::new_v4().to_string()))).into());
    headers.push(
        rsip::typed::CSeq {
            seq: 1,
            method: rsip::Method::Register,
        }
        .into(),
    );

    headers.push(
        rsip::typed::Contact {
            display_name: Some(format!("{}", conf.username.to_string(),)),
            uri: base_uri,
            params: Default::default(),
        }
        .into(),
    );
    headers.push(rsip::headers::ContentLength::default().into());
    headers.push(rsip::headers::Allow::default().into());
    headers.push(Header::UserAgent(UserAgent::new("Sippy")).into());

    let request: SipMessage = rsip::Request {
        method: rsip::Method::Register,
        uri: rsip::Uri {
            scheme: Some(rsip::Scheme::Sip),
            host_with_port: rsip::Domain::from(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                .into(),
            ..Default::default()
        },
        version: rsip::Version::V2,
        headers: headers,
        body: Default::default(),
    }
    .into();

    request
}
pub fn authorized_register_request(
    msg: &SipMessage,
    conf: &JSONConfiguration,
    auth: &WwwAuthenticate,
) -> rsip::SipMessage {
    let ha1 = format!("{}:{}:{}", conf.username, auth.realm, conf.password);
    let ha2 = format!(
        "{}:sip:{}@{}:{}",
        "REGISTER".to_string(),
        conf.extension,
        conf.sip_server,
        conf.sip_port
    );

    let cmd5 = format!(
        "{:x}:{}:{:x}",
        md5::compute(ha1),
        auth.nonce,
        md5::compute(ha2)
    );
    let md5 = format!("{:x}", md5::compute(cmd5));

    let mut headers: rsip::Headers = Default::default();
    headers.push(msg.via_header().unwrap().clone().into());
    headers.push(msg.max_forwards_header().unwrap().clone().into());
    headers.push(msg.from_header().unwrap().clone().into());
    headers.push(msg.to_header().unwrap().clone().into());
    headers.push(msg.contact_header().unwrap().clone().into());
    headers.push(msg.call_id_header().unwrap().clone().into());

    headers.push(
        rsip::typed::CSeq {
            seq: 2,
            method: rsip::Method::Register,
        }
        .into(),
    );
    headers.push(
        rsip::typed::Authorization {
            scheme: auth::Scheme::Digest,
            username: conf.username.to_string(),
            realm: format!("{}", conf.sip_server),
            nonce: auth.clone().nonce,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!(
                    "{}@{}:{}",
                    &conf.extension, &conf.sip_server, &conf.sip_port
                ))
                .into(),
                ..Default::default()
            },
            response: md5.to_string(),
            algorithm: Some(auth::Algorithm::Md5),
            opaque: None,
            qop: None,
        }
        .into(),
    );
    headers.push(rsip::headers::ContentLength::default().into());
    headers.push(rsip::headers::Allow::default().into());

    let request: SipMessage = rsip::Request {
        method: rsip::Method::Register,
        uri: rsip::Uri {
            scheme: Some(rsip::Scheme::Sip),
            host_with_port: rsip::Domain::from(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                .into(),
            ..Default::default()
        },
        version: rsip::Version::V2,
        headers: headers,
        body: Default::default(),
    }
    .into();

    request
}
