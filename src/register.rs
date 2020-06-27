use crate::config::JSONConfiguration;
use crate::message::SipMessageAttributes;
use crate::message::SIP;

pub trait SipMessageRegister {
    fn create_register_message(&self, conf: &JSONConfiguration, ip: &String) -> Self;
}

impl SipMessageRegister for SIP<'_> {
    fn create_register_message(&self, conf: &JSONConfiguration, ip: &String) -> Self {
        Self {
        command : Box::leak(format!("REGISTER sip:{} SIP/2.0", ip).into_boxed_str()),
        content_length : "Content-Length: 0",
        to :  Box::leak(format!("To: sip:{}@{}", conf.username, ip).into_boxed_str()),
        from :  Box::leak(format!("From: sip:{}@{}", conf.username, ip).into_boxed_str()),
        contact :  Box::leak(format!("Contact: sip:{}@{};transport=UDP", conf.username, ip).into_boxed_str()),
        cseq : "CSeq: 445 REGISTER",
        call_id :  Box::leak(format!("Call-ID:{}@{}", &SIP::generate_call_id(), ip).into_boxed_str()),
        via :
         "Via: SIP/2.0/UDP 185.28.212.48;transport=UDP;branch=57ffd673319367006160043a8bad5ab5",
        user_agent : "User-Agent: sippy 0.2.5",
        allow : "Allow: INVITE,CANCEL,BYE,MESSAGE"
        }
    }
}
