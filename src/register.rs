use crate::config::JSONConfiguration;
use crate::message::SipMessageAttributes;
use crate::message::SIP;

pub trait SipMessageRegister {
    fn create_register_message(&mut self, conf: &JSONConfiguration, ip: &String) -> Self;
    fn add_auth(
        &mut self,
        user: &str,
        pass: &str,
        realm: &str,
        uri: &str,
        nonce: &str,
        nc: &str,
    ) -> Self;
}

impl SipMessageRegister for SIP<'_> {
    fn create_register_message(&mut self, conf: &JSONConfiguration, ip: &String) -> Self {
        self.command = Box::leak(format!("REGISTER sip:{} SIP/2.0", ip).into_boxed_str());
        self.content_length = "Content -Length: 0";
        self.to = Box::leak(format!("To: sip:{}@{}", conf.username, ip).into_boxed_str());
        self.from = Box::leak(format!("From: sip:{}@{}", conf.username, ip).into_boxed_str());
        self.contact = Box::leak(
            format!("Contact: sip:{}@{};transport=UDP", conf.username, ip).into_boxed_str(),
        );
        self.cseq = "CSeq: 1 REGISTER";
        self.call_id =
            Box::leak(format!("Call-ID:{}@{}", &SIP::generate_call_id(), ip).into_boxed_str());
        self.via =
            "Via: SIP/2.0/UDP 185.28.212.48;transport=UDP;branch=57ffd673319367006160043a8bad5ab5";
        self.user_agent = "User-Agent: sippy 0.2.5";
        self.allow = "Allow: INVITE,CANCEL,BYE,MESSAGE";

        return *self;
    }
    fn add_auth(
        &mut self,
        user: &str,
        pass: &str,
        realm: &str,
        uri: &str,
        nonce: &str,
        nc: &str,
    ) -> Self {
        let ha1 = md5::compute(&format!("{}:{}:{}", user, realm, pass));
        let ha2 = md5::compute(format!("REGISTER:{}", uri));
        let digest = format!(
            "{:x}:{}:{:08}:{:x}:auth:{:x}",
            ha1,
            nonce,
            nc,
            generate_cnonce(),
            ha2
        );
        let pass = md5::compute(digest);

        self.set_by_key(
            "Authorization",
            Box::leak(format!("response={:x}", pass).into_boxed_str()),
        );
        return *self;
    }
}

fn generate_cnonce() -> md5::Digest {
    md5::compute(rand::random::<[u8; 16]>())
}
