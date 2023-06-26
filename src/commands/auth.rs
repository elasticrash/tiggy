use crate::{config::JSONConfiguration, state::options::SipOptions};
use rand::{distributions::Alphanumeric, Rng};
use rsip::headers::auth::Qop;

#[derive(Debug, Clone)]
pub struct AuthModel {
    pub realm: String,
    pub nonce: String,
    pub qop: Option<Qop>,
}

pub trait Auth {
    fn set_auth(&mut self, conf: &JSONConfiguration, method: &str, auth_model: &AuthModel);
}

impl Auth for SipOptions {
    fn set_auth(&mut self, conf: &JSONConfiguration, method: &str, auth_model: &AuthModel) {
        let ha1 = format!(
            "{}:{}:{}",
            &conf.username, &auth_model.realm, &conf.password
        );
        let ha2 = format!(
            "{}:sip:{}:{}",
            &String::from(method),
            &self.sip_server,
            &self.sip_port
        );
        let cmd5 = if auth_model.qop.is_some() {
            self.qop = true;
            self.nc = Some(1);
            self.cnonce = Some(
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect::<String>(),
            );

            format!(
                "{:x}:{}:0000000{}:{}:{}:{:x}",
                md5::compute(ha1),
                &auth_model.nonce,
                &self.nc.as_ref().unwrap().to_string(),
                &self.cnonce.as_ref().unwrap().to_string(),
                &auth_model.qop.as_ref().unwrap().to_string(),
                md5::compute(ha2)
            )
        } else {
            format!(
                "{:x}:{}:{:x}",
                md5::compute(ha1),
                &auth_model.nonce,
                md5::compute(ha2)
            )
        };

        let md5 = format!("{:x}", md5::compute(cmd5));

        self.nonce = Some(auth_model.nonce.to_string());
        self.md5 = Some(md5);
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        commands::auth::Auth, commands::auth::AuthModel, config::JSONConfiguration,
        state::options::SipOptions,
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
            nonce: None,
            md5: None,
            msg: None,
            cld: None,
            call_id: "it_doesnt_matter".to_string(),
            tag_local: "it_doesnt_matter".to_string(),
            tag_remote: None,
            cnonce: None,
            nc: None,
        };

        options.set_auth(
            &JSONConfiguration {
                username: "1123341004".to_string(),
                password: "123".to_string(),
                sip_server: "not_read_from_this_object".to_string(),
                sip_port: 9999,
                extension: "not_read_from_this_object".to_string(),
                pcap: None,
                reg_timeout: 120,
            },
            &"REGISTER",
            &AuthModel {
                realm: "sip.server.com".to_string(),
                nonce: "YxXVVmMV1CqOO5KBA9b9D4Yi7JNy513z".to_string(),
                qop: None,
            },
        );

        assert_eq!(options.md5.unwrap(), "eb9cb973cb92743c436f8e071c3a73df");
    }
}
