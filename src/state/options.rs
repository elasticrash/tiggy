use std::net::IpAddr;

use super::dialogs::Direction;
use rsip::SipMessage;

#[derive(Clone)]
pub struct SipOptions {
    pub username: String,
    pub extension: String,
    pub sip_server: String,
    pub sip_port: String,
    pub branch: String,
    pub ip: String,
    pub md5: Option<String>,
    pub nonce: Option<String>,
    pub msg: Option<SipMessage>,
    pub cld: Option<String>,
    pub call_id: String,
    pub tag_local: String,
    pub tag_remote: Option<String>,
}

pub struct SelfConfiguration {
    pub ip: IpAddr,
    pub verbosity: Verbosity,
    pub flow: Direction,
}

pub enum Verbosity {
    Diagnostic,
    Minimal,
    Quiet,
}
