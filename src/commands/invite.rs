use crate::composer::header_extension::PartialHeaderClone;
use rsip::headers::{UntypedHeader, UserAgent};
use rsip::{headers::auth, Header, SipMessage};
use uuid::Uuid;

use crate::composer::communication::{Auth, Call, Trying};
use crate::config::JSONConfiguration;
use crate::helper::auth::calculate_md5;

#[derive(Clone)]
pub struct Invite {
    pub extension: String,
    pub username: String,
    pub sip_server: String,
    pub sip_port: String,
    pub ip: String,
    pub msg: Option<SipMessage>,
    pub cld: Option<String>,
    pub md5: Option<String>,
    pub nonce: Option<String>,
}

impl Auth for Invite {
    fn set_auth(&mut self, conf: &JSONConfiguration) {
        let md5 = calculate_md5(
            &conf.username,
            &conf.password,
            &format!("{}", &self.sip_server),
            &self.extension,
            &self.sip_server,
            &self.sip_port,
            &self.nonce.as_ref().unwrap(),
            &String::from("INVITE"),
        );
        self.md5 = Some(md5);
    }
}

impl Call for Invite {
    fn init(&self, destination: String) -> SipMessage {
        let mut headers: rsip::Headers = Default::default();
        let base_uri = rsip::Uri {
            auth: None,
            host_with_port: rsip::Domain::from(format!(
                "sip:{}@{}:{}",
                &self.extension, &self.sip_server, &self.sip_port
            ))
            .into(),
            ..Default::default()
        };

        headers.push(
            rsip::typed::Via {
                version: rsip::Version::V2,
                transport: rsip::Transport::Udp,
                uri: rsip::Uri {
                    host_with_port: (rsip::Domain::from(format!(
                        "{}:{}",
                        &self.ip, &self.sip_port
                    )))
                    .into(),
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
                display_name: Some(format!("{}", &self.username.to_string(),)),
                uri: base_uri.clone(),
                params: vec![rsip::Param::Tag(rsip::param::Tag::new(
                    Uuid::new_v4().to_string(),
                ))],
            }
            .into(),
        );
        headers.push(
            rsip::typed::To {
                display_name: Some(format!("{}", &self.cld.as_ref().unwrap().to_string(),)),
                uri: rsip::Uri {
                    auth: None,
                    host_with_port: rsip::Domain::from(format!(
                        "sip:{}@{}:{}",
                        &self.cld.as_ref().unwrap().to_string(),
                        &self.sip_server,
                        &self.sip_port
                    ))
                    .into(),
                    ..Default::default()
                },
                params: Default::default(),
            }
            .into(),
        );

        headers.push(rsip::headers::CallId::from(Uuid::new_v4().to_string()).into());

        headers.push(
            rsip::typed::Contact {
                display_name: Some(format!("{}", &self.username.to_string())),
                uri: base_uri,
                params: Default::default(),
            }
            .into(),
        );
        headers.push(rsip::headers::MaxForwards::from(70).into());
        headers.push(
            rsip::typed::CSeq {
                seq: 1,
                method: rsip::Method::Invite,
            }
            .into(),
        );
        headers.push(
            rsip::headers::Allow::from(
                "ACK,BYE,CANCEL,INFO,INVITE,NOTIFY,OPTIONS,PRACK,REFER,UPDATE",
            )
            .into(),
        );

        headers.push(Header::UserAgent(UserAgent::new("Tiggy")).into());

        headers.push(
            rsip::typed::Via {
                version: rsip::Version::V2,
                transport: rsip::Transport::Udp,
                uri: rsip::Uri {
                    host_with_port: (rsip::Domain::from(format!(
                        "{}:{}",
                        &self.ip, &self.sip_port
                    )))
                    .into(),
                    ..Default::default()
                },
                params: vec![rsip::Param::Branch(rsip::param::Branch::new(
                    "z9hG4bKnashds8",
                ))],
            }
            .into(),
        );

        let mut body = "v=0\r\n".to_string();
        body.push_str(
            &(format!("o=tggVCE 226678890 391916715 IN IP4 {}\r\n", &self.ip)).to_string(),
        );
        body.push_str("s=tggVCE Audio Call\r\n");
        body.push_str(&(format!("c=IN IP4 {}\r\n", &self.ip)).to_string());
        body.push_str("t=0 0\r\n");
        body.push_str("m=audio 40024 RTP/AVP 0 8 96\r\n");
        body.push_str("a=rtpmap:0 PCMU/8000\r\n");
        body.push_str("a=rtpmap:8 PCMA/8000\r\n");
        body.push_str("a=rtpmap:96 telephone-event/8000\r\n");
        body.push_str("a=fmtp:96 0-15\r\n");

        headers.push(rsip::headers::ContentType::from("application/sdp").into());

        headers.push(rsip::headers::ContentLength::from(body.len().to_string()).into());

        let response: SipMessage = rsip::Request {
            method: rsip::Method::Invite,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!(
                    "{}@{}:{}",
                    destination, &self.sip_server, &self.sip_port
                ))
                .into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers: headers,
            body: body.as_bytes().to_vec(),
        }
        .into();

        response
    }
}

impl Trying for Invite {
    fn attempt(&self) -> SipMessage {
        let headers = &mut self.msg.as_ref().unwrap().partial_header_clone();

        headers.push(rsip::headers::ContentType::from("application/sdp").into());

        headers.push(
            rsip::typed::Authorization {
                scheme: auth::Scheme::Digest,
                username: self.username.to_string(),
                realm: format!("{}", &self.sip_server),
                nonce: self.nonce.as_ref().unwrap().to_string(),
                uri: rsip::Uri {
                    scheme: Some(rsip::Scheme::Sip),
                    host_with_port: rsip::Domain::from(format!(
                        "{}@{}:{}",
                        &self.extension, &self.sip_server, &self.sip_port
                    ))
                    .into(),
                    ..Default::default()
                },
                response: self.md5.as_ref().unwrap().to_string(),
                algorithm: Some(auth::Algorithm::Md5),
                opaque: None,
                qop: None,
            }
            .into(),
        );

        let request: SipMessage = rsip::Request {
            method: rsip::Method::Invite,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!(
                    "{}@{}:{}",
                    self.cld.as_ref().unwrap(),
                    &self.sip_server,
                    &self.sip_port
                ))
                .into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers: headers.clone(),
            body: self.msg.as_ref().unwrap().body().clone(),
        }
        .into();

        request
    }
}
