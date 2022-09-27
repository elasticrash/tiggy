use crate::{composer::communication::Auth, config::JSONConfiguration, state::options::SipOptions};

impl Auth for SipOptions {
    fn set_auth(&mut self, conf: &JSONConfiguration, method: &str) {
        let ha1 = format!(
            "{}:{}:{}",
            &conf.username,
            &self.sip_server.to_string(),
            &conf.password
        );
        let ha2 = format!(
            "{}:sip:{}@{}:{}",
            &String::from(method),
            &self.extension,
            &self.sip_server,
            &self.sip_port
        );

        let cmd5 = format!(
            "{:x}:{}:{:x}",
            md5::compute(ha1),
            self.nonce.as_ref().unwrap(),
            md5::compute(ha2)
        );
        let md5 = format!("{:x}", md5::compute(cmd5));

        self.md5 = Some(md5);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        composer::communication::Auth, config::JSONConfiguration, state::options::SipOptions,
    };

    #[test]
    fn md5_from_config() {
        let mut options = SipOptions {
            username: "this_is_read_from_conf".to_string(),
            extension: "1004".to_string(),
            sip_server: "sip.server.com".to_string(),
            sip_port: "5060".to_string(),
            branch: "it_doesnt_matter".to_string(),
            ip: "it_doesnt_matter".to_string(),
            md5: None,
            nonce: Some("YxXVVmMV1CqOO5KBA9b9D4Yi7JNy513z".to_string()),
            msg: None,
            cld: None,
            call_id: "it_doesnt_matter".to_string(),
            tag_local: "it_doesnt_matter".to_string(),
            tag_remote: None,
        };

        options.set_auth(
            &JSONConfiguration {
                username: "1123341004".to_string(),
                password: "123".to_string(),
                sip_server: "not_read_from_this_object".to_string(),
                sip_port: 9999,
                extension: "not_read_from_this_object".to_string(),
            },
            &"REGISTER",
        );

        assert_eq!(options.md5.unwrap(), "dab6dae59c1e00a003c4d28748e66894");
    }
}
