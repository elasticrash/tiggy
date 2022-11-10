use if_addrs::Interface;

/// Iterates through all the available interfaces and pick the first IPV4
pub fn get_ipv4() -> Result<Interface, String> {
    let is_there_an_ipv4 = if_addrs::get_if_addrs()
        .unwrap()
        .into_iter()
        .find(|ip| ip.ip().is_ipv4());

    let interface = match is_there_an_ipv4 {
        Some(ipv4) => ipv4,
        None => return Err("No IP V4 found".to_string()),
    };

    Ok(interface)
}
