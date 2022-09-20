use rsip::headers::{Allow, UntypedHeader, UserAgent};
use rsip::{Header, SipMessage};

use crate::state::options::SipOptions;

use super::helper::get_base_uri;

impl SipOptions {
    pub fn create_ack(&self) -> SipMessage {
        let mut headers: rsip::Headers = Default::default();
        let base_uri = get_base_uri(&self.extension, &self.sip_server, &self.sip_port);

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
                display_name: Some(self.username.to_string()),
                uri: base_uri.clone(),
                params: vec![rsip::Param::Tag(rsip::param::Tag::new(
                    self.tag_remote.as_ref().unwrap(),
                ))],
            }
            .into(),
        );
        headers.push(
            rsip::typed::To {
                display_name: Some(self.cld.as_ref().unwrap().to_string()),
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
                params: vec![rsip::Param::Tag(rsip::param::Tag::new(&self.tag_local))],
            }
            .into(),
        );

        headers.push(rsip::headers::CallId::from(self.call_id.as_str()).into());

        headers.push(
            rsip::typed::Contact {
                display_name: Some(self.username.to_string()),
                uri: base_uri,
                params: Default::default(),
            }
            .into(),
        );
        headers.push(rsip::headers::MaxForwards::from(70).into());
        headers.push(
            rsip::typed::CSeq {
                seq: 1,
                method: rsip::Method::Ack,
            }
            .into(),
        );
        headers.push(rsip::headers::ContentLength::default().into());
        headers.push(Header::Allow(Allow::new(
            "ACK,BYE,CANCEL,INFO,INVITE,NOTIFY,OPTIONS,PRACK,REFER,UPDATE",
        )));
        headers.push(Header::UserAgent(UserAgent::new("Tiggy")));

        let response: SipMessage = rsip::Request {
            method: rsip::Method::Ack,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(format!(
                    "{}@{}:{}",
                    &self.username, &self.sip_server, &self.sip_port
                ))
                .into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers,
            body: Default::default(),
        }
        .into();

        response
    }
}
