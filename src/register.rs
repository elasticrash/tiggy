use crate::config::JSONConfiguration;
use crate::models::SIP;
use rsip::{
    headers::{auth, CallId, UntypedHeader},
    message::{HeadersExt},
    Header, SipMessage,
};
use uuid::Uuid;

pub trait SipMessageRegister {
    fn generate_register_request(
        &mut self,
        conf: &JSONConfiguration,
        ip: &String,
    ) -> rsip::SipMessage;
    fn add_authentication(
        &mut self,
        sip: rsip::Request,
        conf: &JSONConfiguration,
        ip: &String,
    ) -> rsip::SipMessage;
}

impl SipMessageRegister for SIP {
    fn generate_register_request(&mut self, conf: &JSONConfiguration, ip: &String) -> SipMessage {
        let mut headers: rsip::Headers = Default::default();

        let base_uri = rsip::Uri {
            auth: None,
            host_with_port: rsip::Domain::from(format!(
                "sip:{}@{}:5060",
                &conf.extension, &conf.sip_server
            ))
            .into(),
            ..Default::default()
        };

        headers.push(
            rsip::typed::Via {
                version: rsip::Version::V2,
                transport: rsip::Transport::Udp,
                uri: rsip::Uri {
                    host_with_port: (rsip::Domain::from(format!("{}:5060", ip))).into(),
                    ..Default::default()
                },
                params: vec![rsip::Param::Branch(rsip::param::Branch::new(
                    "z9hG4bKnashds7",
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
        headers.push(
            Header::CallId(CallId::new(format!("{}@sippy", Uuid::new_v4().to_string()))).into(),
        );
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

        rsip::Request {
            method: rsip::Method::Register,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!("{}:5060", &conf.sip_server)).into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers: headers,
            body: Default::default(),
        }
        .into()
    }
    fn add_authentication(
        &mut self,
        previous: rsip::Request,
        conf: &JSONConfiguration,
        ip: &String,
    ) -> rsip::SipMessage {
        let mut headers: rsip::Headers = Default::default();
        headers.push(previous.via_header().unwrap().clone().into());
        headers.push(previous.max_forwards_header().unwrap().clone().into());
        headers.push(previous.from_header().unwrap().clone().into());
        headers.push(previous.to_header().unwrap().clone().into());
        headers.push(previous.contact_header().unwrap().clone().into());
        headers.push(previous.call_id_header().unwrap().clone().into());

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
                nonce: "YVv5M2Fb+AdIPO8TrN6PYtIuEJ4E4kiA".to_string(),
                uri: rsip::Uri {
                    scheme: Some(rsip::Scheme::Sip),
                    host_with_port: rsip::Domain::from(format!("sip:{}:5060", &conf.sip_server))
                        .into(),
                    ..Default::default()
                },
                response: "asdfsfs".to_string(),
                algorithm: Some(auth::Algorithm::Md5),
                opaque: None,
                qop: None,
            }
            .into(),
        );
        headers.push(rsip::headers::ContentLength::default().into());
        headers.push(rsip::headers::Allow::default().into());

        rsip::Request {
            method: rsip::Method::Register,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!("{}:5060", &conf.sip_server)).into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers: headers,
            body: Default::default(),
        }
        .into()
    }
}
