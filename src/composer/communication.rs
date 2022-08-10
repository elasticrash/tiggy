use rsip::SipMessage;

use crate::config::JSONConfiguration;

pub trait Call {
    fn ask(&self) -> SipMessage;
}

pub trait Trying {
    fn attempt(&self) -> SipMessage;
}

pub trait Auth {
    fn set_auth(&mut self, conf: &JSONConfiguration);
}
