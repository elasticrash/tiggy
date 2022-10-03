use rsip::headers::{CSeq, Contact, UntypedHeader, UserAgent, Via};
use rsip::{Header, SipMessage};

use crate::state::options::SipOptions;

use super::helper::get_base_uri;

impl SipOptions {
    pub fn create_ack(
        &self,
        via: &Via,
        rr: Vec<&Header>,
        cnt: &Contact,
        _cseq: &CSeq,
    ) -> SipMessage {
        let mut headers: rsip::Headers = Default::default();
        let base_uri = get_base_uri(&self.extension, &self.sip_server, &self.sip_port);

        headers.push(Header::Via(via.clone()));
        headers.push(
            rsip::typed::From {
                display_name: Some(self.username.to_string()),
                uri: base_uri.clone(),
                params: vec![rsip::Param::Tag(rsip::param::Tag::new(&self.tag_local))],
            }
            .into(),
        );

        let lroute = rr.last().unwrap().to_string();
        let (_, route_value) = lroute.split_at(14).to_owned();

        headers.push(rsip::Header::Route(rsip::headers::Route::new(
            route_value.to_string(),
        )));

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
                params: vec![rsip::Param::Tag(rsip::param::Tag::new(
                    self.tag_remote.as_ref().unwrap(),
                ))],
            }
            .into(),
        );

        headers.push(rsip::headers::CallId::from(self.call_id.as_str()).into());
        headers.push(
            rsip::typed::CSeq {
                seq: 2,
                method: rsip::Method::Ack,
            }
            .into(),
        );
        headers.push(
            rsip::typed::Contact {
                display_name: Some(self.username.to_string()),
                uri: base_uri,
                params: Default::default(),
            }
            .into(),
        );
        headers.push(rsip::headers::MaxForwards::from(70).into());

        headers.push(Header::UserAgent(UserAgent::new("Tiggy")));
        headers.push(rsip::headers::ContentLength::default().into());

        let mut l_contact = cnt.to_string();
        let (_, contact_value) = l_contact.split_at_mut(14);

        let response: SipMessage = rsip::Request {
            method: rsip::Method::Ack,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                host_with_port: rsip::Domain::from(rem_last(contact_value).to_string()).into(),
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

fn rem_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}

#[cfg(test)]
mod tests {
    use crate::commands::ack::rem_last;

    #[test]
    fn remove_last() {
        let test_value = "test1";
        assert_eq!("test", rem_last(test_value));
    }
}
