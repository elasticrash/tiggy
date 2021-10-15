use crate::config::JSONConfiguration;
use rsip::{
    headers::{auth, CallId, UntypedHeader, UserAgent},
    message::HeadersExt,
    typed::WwwAuthenticate,
    Header, SipMessage,
};
use uuid::Uuid;


pub fn ok(_conf: &JSONConfiguration,_ipp: &String) -> rsip::SipMessage {
    todo!();
}
