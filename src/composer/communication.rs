use rsip::SipMessage;

pub trait Call {
    fn ask(&self) -> SipMessage;
}

pub trait Trying {
    fn attempt(&self) -> SipMessage;
}
