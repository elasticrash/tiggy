use rsip::SipMessage;

#[derive(Clone)]
pub struct Register {
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

pub fn get_base_uri(number: &str, server: &str, port: &str) -> rsip::Uri {
    rsip::Uri {
        auth: None,
        host_with_port: rsip::Domain::from(format!("sip:{}@{}:{}", number, server, port)).into(),
        ..Default::default()
    }
}

pub fn get_via(ip: &str, port: &str) -> rsip::Header {
    rsip::typed::Via {
        version: rsip::Version::V2,
        transport: rsip::Transport::Udp,
        uri: rsip::Uri {
            host_with_port: (rsip::Domain::from(format!("{}:{}", ip, port))).into(),
            ..Default::default()
        },
        params: vec![rsip::Param::Branch(rsip::param::Branch::new(
            "z9hG4bKnashds8",
        ))],
    }
    .into()
}

pub fn get_from(username: &str, tag: &str, base_uri: rsip::Uri) -> rsip::Header {
    rsip::typed::From {
        display_name: Some(username.to_string()),
        uri: base_uri,
        params: vec![rsip::Param::Tag(rsip::param::Tag::new(tag))],
    }
    .into()
}

pub fn get_to(username: &str, did: &str, server: &str, port: &str) -> rsip::Header {
    rsip::typed::To {
        display_name: Some(username.to_string()),
        uri: rsip::Uri {
            auth: None,
            host_with_port: rsip::Domain::from(format!(
                "sip:{}@{}:{}",
                did.to_string(),
                server,
                port
            ))
            .into(),
            ..Default::default()
        },
        params: Default::default(),
    }
    .into()
}

pub fn get_contact(username: &str, did: &str, server: &str, port: &str) -> rsip::Header {
    rsip::typed::Contact {
        display_name: Some(username.to_string()),
        uri: rsip::Uri {
            host_with_port: (rsip::Domain::from(format!("sip:{}@{}:{}", did, server, port))).into(),
            ..Default::default()
        },
        params: Default::default(),
    }
    .into()
}
