use rsip::{
    prelude::{HeadersExt, ToTypedHeader},
    Header, Headers, SipMessage,
};

pub trait CustomHeaderExtension {
    fn get_via_header_array(&self) -> Vec<&Header>;
    fn get_record_route_header_array(&self) -> Vec<&Header>;
    fn push_many(&mut self, new_headers: Vec<&Header>);
}

impl CustomHeaderExtension for Headers {
    fn get_via_header_array(&self) -> Vec<&Header> {
        self.iter()
            .filter(|h| matches!(h, Header::Via(_)))
            .collect()
    }
    fn get_record_route_header_array(&self) -> Vec<&Header> {
        self.iter()
            .filter(|h| matches!(h, Header::RecordRoute(_)))
            .collect()
    }

    fn push_many(&mut self, new_headers: Vec<&Header>) {
        for hd in new_headers {
            self.push(hd.clone());
        }
    }
}

pub trait PartialHeaderClone {
    fn partial_header_clone(&self) -> Headers;
}

impl PartialHeaderClone for SipMessage {
    fn partial_header_clone(&self) -> Headers {
        let mut headers: Headers = Default::default();
        headers.push(self.via_header().unwrap().clone().into());
        headers.push(self.max_forwards_header().unwrap().clone().into());
        headers.push(self.from_header().unwrap().clone().into());
        headers.push(self.to_header().unwrap().clone().into());
        headers.push(self.contact_header().unwrap().clone().into());
        headers.push(self.call_id_header().unwrap().clone().into());
        headers.push(self.user_agent_header().unwrap().clone().into());

        let cseq = self.cseq_header().unwrap().typed().unwrap();

        headers.push(
            rsip::typed::CSeq {
                seq: cseq.seq + 1,
                method: cseq.method,
            }
            .into(),
        );

        headers.push(
            rsip::headers::Allow::from(
                "ACK,BYE,CANCEL,INFO,INVITE,NOTIFY,OPTIONS,PRACK,REFER,UPDATE",
            )
            .into(),
        );

        headers.push(rsip::headers::ContentLength::from(self.body().len().to_string()).into());
        headers
    }
}
