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

    let blank = SIP::blank();
    let command = blank.create_register_message(&conf.clone(), &ip.clone().to_string());

    println!("[{}] - {:?}", line!(), command.generate_sip());

    socket
        .send_to(command.generate_sip().as_bytes(), &conf.sip_server)
        .await
        .unwrap();

    let mut buf = [0; 65535];
    let (amt, src) = socket.recv_from(&mut buf).await.unwrap();

    socket
        .send_to(command.generate_sip().as_bytes(), &conf.sip_server)
        .await
        .unwrap();

    let full_message = String::from_utf8_lossy(&buf);
    parser(full_message.split_at(amt).0);
    println!("[{}] - {:?}", line!(), full_message.split_at(amt).0);

    Ok(())
}

fn parser(msg: &str) {
    let carrier = "\r\n";
    let v: Vec<&str> = msg.split(carrier).collect();
    let mut empty_sip = SIP::empty();
    empty_sip.set_by_key("command", &v[0]);

    for i in 1..v.len() {
        let split: Vec<&str> = v[i].split(':').collect();
        let key = split.first().unwrap();
        let value = split.last().unwrap();

        empty_sip.set_by_key(key, value);
    }

    println!("[{}] - {:?}", line!(), empty_sip);
}
