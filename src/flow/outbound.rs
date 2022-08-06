use std::net::IpAddr;

use crate::{commands::invite::Invite, config::JSONConfiguration};

pub fn outbound_start(conf: &JSONConfiguration, ip: &IpAddr) {
    let mut _invite: Invite = Invite {
        extension: conf.extension.to_string(),
        username: conf.username.clone(),
        sip_server: conf.sip_server.to_string(),
        sip_port: conf.sip_port.to_string(),
        ip: ip.to_string(),
        msg: None,
        cld: None,
    };

    todo!()
}
