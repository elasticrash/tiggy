use phf::phf_map;
use phf::Map;

macro_rules! box_value {
    ($input:expr, $key:expr , $value:expr) => {
        Box::leak(
            format!(
                "{}:{}{}",
                $key,
                $input.clone().to_string(),
                $value.to_string()
            )
            .into_boxed_str(),
        )
    };
}

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

#[derive(Copy, Clone, Debug)]
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
                        format!("{}{}", self.command.clone().to_string(), value.to_string())
                            .into_boxed_str(),
                    );
                }
                if data == "content_length" {
                    self.content_length = box_value!(self.content_length, key, value);
                }
                if data == "to" {
                    self.to = box_value!(self.to, key, value);
                }
                if data == "from" {
                    self.from = box_value!(self.from, key, value);
                }
                if data == "contact" {
                    self.contact = box_value!(self.contact, key, value);
                }
                if data == "cseq" {
                    self.cseq = box_value!(self.cseq, key, value);
                }
                if data == "call_id" {
                    self.call_id = box_value!(self.call_id, key, value);
                }
                if data == "via" {
                    self.via = box_value!(self.via, key, value);
                }
                if data == "user_agent" {
                    self.user_agent = box_value!(self.user_agent, key, value)
                }
                if data == "allow" {
                    self.allow = box_value!(self.allow, key, value);
                }
                if data == "www_authenticate" {
                    self.www_authenticate = box_value!(self.www_authenticate, key, value);
                }
                if data == "authorization" {
                    self.authorization = box_value!(self.authorization, key, value);
                }
            }
            None => {}
        }
    }
    fn generate_call_id() -> String {
        return format!("{:x}", md5::compute(rand::random::<[u8; 16]>()));
    }
}
