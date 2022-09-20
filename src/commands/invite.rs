use crate::composer::header_extension::PartialHeaderClone;
use crate::state::options::SipOptions;
use rsip::headers::{UntypedHeader, UserAgent};
use rsip::{headers::auth, Header, SipMessage};

use super::helper::{get_base_uri, get_contact, get_fake_sdp, get_from, get_to, get_via};

impl SipOptions {
    pub fn set_initial_invite(&self) -> SipMessage {
        let mut headers: rsip::Headers = Default::default();
        let base_uri = get_base_uri(&self.extension, &self.sip_server, &self.sip_port);

        headers.push(get_via(&self.ip, &self.sip_port));
        headers.push(get_from(&self.username, &self.tag_local, base_uri));
        headers.push(get_to(
            self.cld.as_ref().unwrap(),
            self.cld.as_ref().unwrap(),
            &self.sip_server,
            &self.sip_port,
        ));
        headers.push(rsip::headers::CallId::from(self.call_id.as_str()).into());
        headers.push(get_contact(
            &self.username,
            &self.extension,
            &self.sip_server,
            &self.sip_port,
        ));
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

        headers.push(Header::UserAgent(UserAgent::new("Tiggy")));

        let fake_sdp_body = get_fake_sdp(&self.ip);

        headers.push(rsip::headers::ContentType::from("application/sdp").into());
        headers.push(rsip::headers::ContentLength::from(fake_sdp_body.len().to_string()).into());

        let response: SipMessage = rsip::Request {
            method: rsip::Method::Invite,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!(
                    "{}@{}:{}",
                    &self.cld.as_ref().unwrap(),
                    &self.sip_server,
                    &self.sip_port
                ))
                .into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers,
            body: fake_sdp_body.as_bytes().to_vec(),
        }
        .into();

        response
    }
}

impl SipOptions {
    pub fn push_auth_to_invite(&self) -> SipMessage {
        let headers = &mut self.msg.as_ref().unwrap().partial_header_clone();

        headers.push(rsip::headers::ContentType::from("application/sdp").into());

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
