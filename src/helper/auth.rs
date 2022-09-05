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
    let ha2 = format!(
        "{}:sip:{}@{}:{}",
        method,
        extension,
        sip_server,
        sip_port
    );

    let cmd5 = format!(
        "{:x}:{}:{:x}",
        md5::compute(ha1),
        nonce,
        md5::compute(ha2)
    );
    let md5 = format!("{:x}", md5::compute(cmd5));
    md5
}
