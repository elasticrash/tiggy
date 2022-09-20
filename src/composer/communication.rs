use crate::config::JSONConfiguration;

pub trait Auth {
    fn set_auth(&mut self, conf: &JSONConfiguration, method: &str);
}
