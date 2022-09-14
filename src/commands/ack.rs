use rsip::headers::{Allow, UntypedHeader, UserAgent};
use rsip::{Header, SipMessage};

use crate::composer::communication::Start;

use super::helper::get_base_uri;

#[derive(Clone)]
pub struct Ack {
    pub extension: String,
    pub username: String,
    pub sip_server: String,
    pub sip_port: String,
    pub ip: String,
    pub msg: Option<SipMessage>,
    pub cld: Option<String>,
    pub call_id: String,
    pub tag_local: String,
    pub tag_remote: String,
}

impl Start for Ack {
    fn set(&self) -> SipMessage {
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
                params: vec![rsip::Param::Tag(rsip::param::Tag::new(&self.tag_remote))],
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

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use nom::{
        bytes::complete::{tag, take_until},
        error::Error,
        sequence::tuple,
    };
    use rsip::{prelude::HeadersExt, SipMessage};

    #[test]
    fn response_test() {
        let response =  concat!(
            "SIP/2.0 200 OK\r\n",
            "Via: SIP/2.0/UDP 172.26.2.11:5060;rport=8762;received=63.32.37.93;branch=z9hG4bKnashds8\r\n",
            "From: 441354961002 <sip:441354961002@register.staging.cloudcall.com>;tag=7be4516f-ec4e-4d31-9038-1f93533cd73d\r\n",
            "To: 441354961001 <sip:441354961001@register.staging.cloudcall.com>;tag=ACU-18fb315-205d63da\r\n",
            "Call-ID: fe4e50cd-a4c7-4bea-8549-fc24a00499ce\r\n",
            "CSeq: 2 INVITE\r\n",
            "Contact: <sip:185.28.212.1:5060>\r\n",
            "Record-Route: <sip:185.28.212.48;lr;ftag=7be4516f-ec4e-4d31-9038-1f93533cd73d;did=d6d.7f21;nat=yes>\r\n",
            "Supported: replaces,sdp-anat\r\n",
            "Allow: INVITE, ACK, CANCEL, BYE, PRACK, UPDATE, REFER, NOTIFY\r\n",
            "Server: CloudCall SBC/v.7.20A.258.457\r\n",
            "Content-Type: application/sdp\r\n",
            "Content-Length: 177\r\n",
            "X-H323-Conf-ID: 3082375746-2355046909-2925081530-3933867422\r\n",
            "X-Recording-Info: session-id=1f658c8f-31cd-4585-aadd-8028883c47a8;conf-id=3082375746-2355046909-2925081530-3933867422\r\n\r\n");

        let parse_response = SipMessage::try_from(response);

        assert!(parse_response.is_ok());
        let hstr = parse_response
            .clone()
            .unwrap()
            .from_header()
            .unwrap()
            .to_string();
        println!("{:?}", hstr.split(";").collect::<Vec<&str>>()[1]);

        println!("{:?}", rem);
    }
}
