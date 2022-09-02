use rsip::SipMessage;

use crate::config::JSONConfiguration;

pub trait Start {
    fn set(&self) -> SipMessage;
}

pub trait Call {
    fn init(&self, destination: String) -> SipMessage;
}

pub trait Trying {
    fn attempt(&self) -> SipMessage;
}

pub trait Auth {
    fn set_auth(&mut self, conf: &JSONConfiguration);
}
