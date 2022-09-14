pub fn calculate_md5(
    username: &String,
    password: &String,
    realm: &String,
    extension: &String,
    sip_server: &String,
    sip_port: &String,
    nonce: &String,
    method: &String,
) -> String {
    let ha1 = format!("{}:{}:{}", username, realm, password);
    let ha2 = format!("{}:sip:{}@{}:{}", method, extension, sip_server, sip_port);

    let cmd5 = format!("{:x}:{}:{:x}", md5::compute(ha1), nonce, md5::compute(ha2));
    let md5 = format!("{:x}", md5::compute(cmd5));
    md5
}

#[cfg(test)]
mod tests {
    use crate::helper::auth::calculate_md5;

    #[test]
    fn md5_from_config() {
        let md5 = calculate_md5(
            &"1123341004".to_string(),
            &"123".to_string(),
            &"sip.server.com".to_string(),
            &"1004".to_string(),
            &"sip.server.com".to_string(),
            &"5060".to_string(),
            &"YxXVVmMV1CqOO5KBA9b9D4Yi7JNy513z".to_string(),
            &"REGISTER".to_string(),
        );
        assert_eq!(md5, "dab6dae59c1e00a003c4d28748e66894");
    }
}
