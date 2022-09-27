use crate::composer::header_extension::PartialHeaderClone;
use crate::state::options::SipOptions;
use rsip::headers::{Allow, UntypedHeader, UserAgent};
use rsip::{headers::auth, Header, SipMessage};

use super::helper::{get_base_uri, get_contact, get_from, get_to, get_via};

impl SipOptions {
    pub fn set_initial_register(&self) -> SipMessage {
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
    pub fn unregister(&self) -> SipMessage {
        let headers = &mut self.msg.as_ref().unwrap().partial_header_clone();
        headers.push(rsip::headers::Expires::from(0).into());
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

impl SipOptions {
    pub fn push_auth_to_register(&self) -> SipMessage {
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
