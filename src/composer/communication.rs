use rsip::SipMessage;

pub trait Answer {
    fn answering(&self) ->  SipMessage;
}

pub trait Ask {
    fn asking(&self) ->  SipMessage;
}