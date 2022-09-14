use rsip::headers::{Allow, UntypedHeader, UserAgent};
use rsip::{headers::auth, Header, SipMessage};

use crate::composer::communication::{Auth, Start, Trying};
use crate::composer::header_extension::PartialHeaderClone;
use crate::config::JSONConfiguration;
use crate::helper::auth::calculate_md5;

use super::helper::{get_base_uri, get_from, get_to, get_via, get_contact};

#[derive(Clone)]
pub struct Register {
    pub username: String,
    pub extension: String,
    pub sip_server: String,
    pub sip_port: String,
    pub branch: String,
    pub ip: String,
    pub md5: Option<String>,
    pub nonce: Option<String>,
    pub msg: Option<SipMessage>,
    pub cld: Option<String>,
    pub call_id: String,
    pub tag_local: String,
    pub tag_remote: Option<String>,
}

impl Auth for Register {
    fn set_auth(&mut self, conf: &JSONConfiguration) {
        let md5 = calculate_md5(
            &conf.username,
            &conf.password,
            &self.sip_server.to_string(),
            &self.extension,
            &self.sip_server,
            &self.sip_port,
            self.nonce.as_ref().unwrap(),
            &String::from("REGISTER"),
        );
        self.md5 = Some(md5);
    }
}

impl Start for Register {
    fn set(&self) -> SipMessage {
        let mut headers: rsip::Headers = Default::default();

        let base_uri = get_base_uri(&self.extension, &self.sip_server, &self.sip_port);

        headers.push(get_via(&self.ip, &self.sip_port));
        headers.push(get_from(&self.username, &self.tag_local, base_uri));
        headers.push(get_to(
            &self.username,
            &self.extension,
            &self.sip_server,
            &self.sip_port,
        ));
        headers.push(rsip::headers::CallId::from(self.call_id.as_str()).into());
        headers.push(get_contact(
            &self.username,
            &self.username,
            &self.ip,
            &self.sip_port,
        ));
        headers.push(rsip::headers::MaxForwards::from(70).into());
        headers.push(
            rsip::typed::CSeq {
                seq: 1,
                method: rsip::Method::Register,
            }
            .into(),
        );
        headers.push(Header::Allow(Allow::new(
            "ACK,BYE,CANCEL,INFO,INVITE,NOTIFY,OPTIONS,PRACK,REFER,UPDATE",
        )));
        headers.push(Header::UserAgent(UserAgent::new("Tiggy")));
        headers.push(rsip::headers::ContentLength::default().into());

        let request: SipMessage = rsip::Request {
            method: rsip::Method::Register,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!(
                    "{}:{}",
                    &self.sip_server, &self.sip_port
                ))
                .into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers,
            body: Default::default(),
        }
        .into();

        request
    }
}

impl Trying for Register {
    fn attempt(&self) -> SipMessage {
        let headers = &mut self.msg.as_ref().unwrap().partial_header_clone();

        headers.push(
            rsip::typed::Authorization {
                scheme: auth::Scheme::Digest,
                username: self.username.to_string(),
                realm: self.sip_server.to_string(),
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
        headers.push(Header::Allow(Allow::new(
            "ACK,BYE,CANCEL,INFO,INVITE,NOTIFY,OPTIONS,PRACK,REFER,UPDATE",
        )));

        let request: SipMessage = rsip::Request {
            method: rsip::Method::Register,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!(
                    "{}:{}",
                    &self.sip_server, &self.sip_port
                ))
                .into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers: headers.clone(),
            body: Default::default(),
        }
        .into();

        request
    }
}
