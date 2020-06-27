use phf::phf_map;
use phf::Map;

static MAP: Map<&'static str, &'static str> = phf_map! {
    "Command"=>"command",
    "Content-Length"=>"content_length",
    "To"=>"to",
    "From"=>"from",
    "Contact"=>"contact",
    "CSeq"=>"cseq",
    "Call-ID"=>"call_id",
    "Via"=>"via",
    "User-Agent"=>"user_agent",
    "Allow"=> "allow",
    "WWW-Authenticate"=> "www_authenticate",
    "Authorization"=>"authorization"
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
    pub www_authenticate: &'a str,
    pub authorization: &'a str,
}

pub struct WWWAuthenticate<'a> {
    pub realm: &'a str,
    pub nonce: &'a str,
}

impl<'a> Default for SIP<'a> {
    fn default() -> SIP<'a> {
        SIP {
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
            www_authenticate: "",
            authorization: "",
        }
    }
}

pub trait SipMessageAttributes {
    fn generate_sip(&self) -> String;
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
            self.www_authenticate.to_string(),
            self.authorization.to_string(),
            carrier.to_string(),
        ];

        return msg
            .into_iter()
            .filter(|m| m != "")
            .collect::<Vec<String>>()
            .join(carrier);
    }
    fn set_by_key<'a>(&mut self, key: &str, value: &str) {
        match MAP.get(key).cloned() {
            Some(data) => {
                if data == "command" {
                    self.command = Box::leak(
                        format!(
                            "{}{}",
                            self.command.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "content_length" {
                    self.content_length = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.content_length.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "to" {
                    self.to = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.to.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "from" {
                    self.from = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.from.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "contact" {
                    self.contact = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.contact.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "cseq" {
                    self.cseq = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.cseq.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "call_id" {
                    self.call_id = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.call_id.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "via" {
                    self.via = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.via.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "user_agent" {
                    self.user_agent = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.user_agent.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "allow" {
                    self.allow = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.allow.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "www_authenticate" {
                    self.www_authenticate = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.www_authenticate.clone().to_string(),
                            value.to_string()
                        )
                        .into_boxed_str(),
                    );
                }
                if data == "authorization" {
                    self.authorization = Box::leak(
                        format!(
                            "{}:{}{}",
                            key,
                            self.authorization.clone().to_string(),
                            value.to_string()
                        )
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
}
