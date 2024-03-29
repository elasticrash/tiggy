extern crate serde;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Clone, Debug)]
pub struct JSONConfiguration {
    pub username: String,
    pub password: String,
    pub sip_server: String,
    pub sip_port: u16,
    pub extension: String,
    pub pcap: Option<String>,
    pub reg_timeout: i8,
}

pub fn read(filename: &str) -> serde_json::Result<JSONConfiguration> {
    match File::open(filename) {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap();
            let config: JSONConfiguration = serde_json::from_str(&buffer).unwrap();
            Ok(config)
        }
        Err(_why) => panic!("file not found"),
    }
}
