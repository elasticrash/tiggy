use phf::phf_map;
use phf::Map;

static MAP: Map<&'static str, &'static str> = phf_map! {
    "Content-Length"=>"content_length",
    "To"=>"to",
    "From"=>"from",
    "Contact"=>"contact",
    "CSeq"=>"cseq",
    "Call-ID"=>"call_id",
    "Via"=>"via",
    "User-Agent"=>"user_agent",
    "Allow"=> "allow",
};

#[derive(Copy, Clone, Debug)]
pub struct SIP<'a> {
    pub command: &'a str,
    pub content_length: &'a str,
    pub to: &'a str,
    pub from: &'a str,
    pub contact: &'a str,
    pub cseq: &'a str,
    pub call_id: &'a str,
    pub via: &'a str,
    pub user_agent: &'a str,
    pub allow: &'a str,
}

pub trait SipMessageAttributes {
    fn generate_sip(&self) -> String;
    fn empty() -> Self;
    fn blank() -> Self;
    fn set_by_key(self: &mut Self, key: &str, value: &str);
    fn generate_call_id() -> String;
}

impl SipMessageAttributes for SIP<'_> {
    fn generate_sip(&self) -> String {
        let carrier = "\r\n";
        let msg = vec![
            self.command.to_string(),
            self.content_length.to_string(),
            self.to.to_string(),
            self.from.to_string(),
            self.contact.to_string(),
            self.cseq.to_string(),
            self.call_id.to_string(),
            self.via.to_string(),
            self.user_agent.to_string(),
            self.allow.to_string(),
            carrier.to_string(),
        ];

        return msg.join(carrier);
    }
    fn empty() -> Self {
        return SIP {
            command: "",
            content_length: "Content-Length:",
            to: "To:",
            from: "From:",
            contact: "Contact:",
            cseq: "CSeq:",
            call_id: "Call-ID:",
            via: "Via:",
            user_agent: "User-Agent:",
            allow: "Allow:",
        };
    }
    fn set_by_key<'a>(&mut self, key: &str, value: &str) {
        match MAP.get(key).cloned() {
            Some(data) => {
                if data == "command" {
                    self.command = Box::leak(
                        format!("{}{}", self.command.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "content_length" {
                    self.content_length = Box::leak(
                        format!(
                            "{}{}",
                            self.content_length.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "to" {
                    self.to = Box::leak(
                        format!("{}{}", self.to.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "from" {
                    self.from = Box::leak(
                        format!("{}{}", self.from.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "contact" {
                    self.contact = Box::leak(
                        format!("{}{}", self.contact.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "call_id" {
                    self.call_id = Box::leak(
                        format!("{}{}", self.call_id.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "via" {
                    self.via = Box::leak(
                        format!("{}{}", self.via.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "user_agent" {
                    self.user_agent = Box::leak(
                        format!(
                            "{}{}",
                            self.user_agent.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "allow" {
                    self.allow = Box::leak(
                        format!("{}{}", self.allow.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
            }
            None => {}
        }
    }
    fn generate_call_id() -> String {
        return format!("{:x}", md5::compute(rand::random::<[u8; 16]>()));
    }
    fn blank() -> Self {
        return SIP {
            command: "",
            content_length: "",
            to: "",
            from: "",
            contact: "",
            cseq: "",
            call_id: "",
            via: "",
            user_agent: "",
            allow: "",
        };
    }
}
