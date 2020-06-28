extern crate md5;
extern crate phf;
extern crate rand;
extern crate tokio;
mod config;
mod message;
mod register;
use crate::message::SipMessageAttributes;
use crate::message::SIP;
use crate::register::SipMessageRegister;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let conf = config::read("./config.json").unwrap();
    let ip = get_if_addrs::get_if_addrs().unwrap()[0].addr.ip();
    println!("[{}] - {:?}", line!(), ip.to_string());

    let mut socket = UdpSocket::bind("0.0.0.0:5060").await.unwrap();

    let mut blank = SIP::default();
    let mut r_message_init = blank.create_register_message(&conf.clone(), &ip.clone().to_string());

    print_send(r_message_init.generate_sip());
    socket
        .send_to(r_message_init.generate_sip().as_bytes(), &conf.sip_server)
        .await
        .unwrap();

    let mut buf = [0; 65535];
    let (amt, src) = socket.recv_from(&mut buf).await.unwrap();

    let r_message_a = String::from_utf8_lossy(&buf);
    let p_message_a = parser(r_message_a.split_at(amt).0);
    print_received(p_message_a.generate_sip());

    let www_auth_body = p_message_a.www_authenticate.split(':').last();
    let www_auth_parts: Vec<&str> = www_auth_body.unwrap().split(',').collect();
    let nonce: &str = www_auth_parts[1]
        .split('=')
        .last()
        .unwrap()
        .split('\"')
        .collect::<Vec<&str>>()[1];

    println!("[{}] - {:?}", line!(), nonce);

    let mut r_message_v = r_message_init.add_auth(
        &conf.username,
        &conf.password,
        "sip:192.168.137.1",
        "192.168.137.1",
        nonce,
        "3",
    );
    r_message_v.cseq = "CSeq:3 REGISTER";
    print_send(r_message_v.generate_sip());
    socket
        .send_to(r_message_v.generate_sip().as_bytes(), &conf.sip_server)
        .await
        .unwrap();

    let (amt, src) = socket.recv_from(&mut buf).await.unwrap();

    let r_message_b = String::from_utf8_lossy(&buf);
    let p_message_b = parser(r_message_b.split_at(amt).0);
    print_received(p_message_b.generate_sip());

    Ok(())
}

fn parser(msg: &str) -> SIP {
    let carrier = "\r\n";
    let v: Vec<&str> = msg.split(carrier).collect();
    let mut new_sip = SIP::default();
    new_sip.set_by_key("Command", &v[0]);

    for i in 1..v.len() {
        let split: Vec<&str> = v[i].split(':').collect();
        let key = split.first().unwrap();
        let value = split.last().unwrap();
        new_sip.set_by_key(key, value);
    }

    return new_sip;
}

fn print_send(msg: String) {
    println!("[{}] - {:?}", line!(), ">>>>>>>>>>>>>");
    println!("[{}] - {:?}", line!(), msg);
}

fn print_received(msg: String) {
    println!("[{}] - {:?}", line!(), "<<<<<<<<<<<<<");
    println!("[{}] - {:?}", line!(), msg);
}
