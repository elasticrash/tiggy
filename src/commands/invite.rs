use crate::composer::header_extension::PartialHeaderClone;
use rsip::headers::{UntypedHeader, UserAgent};
use rsip::{headers::auth, Header, SipMessage};

use crate::composer::communication::{Auth, Call, Trying};
use crate::config::JSONConfiguration;
use crate::helper::auth::calculate_md5;

use super::helper::{get_base_uri, get_contact, get_fake_sdp, get_from, get_to, get_via};

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
    pub call_id: String,
    pub tag_local: String,
    pub tag_remote: Option<String>,
}

impl Auth for Invite {
    fn set_auth(&mut self, conf: &JSONConfiguration) {
        let md5 = calculate_md5(
            &conf.username,
            &conf.password,
            &self.sip_server,
            &self.extension,
            &self.sip_server,
            &self.sip_port,
            self.nonce.as_ref().unwrap(),
            &String::from("INVITE"),
        );
        self.md5 = Some(md5);
    }
}

impl Call for Invite {
    fn init(&self, destination: String) -> SipMessage {
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
                    destination, &self.sip_server, &self.sip_port
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

impl Trying for Invite {
    fn attempt(&self) -> SipMessage {
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
