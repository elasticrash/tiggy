use rsip::SipMessage;

pub struct SIP {
    pub history: Vec<SipMessage>,
}

pub struct SocketV4 {
    pub ip: String,
    pub port: u16,
}
