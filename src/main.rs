extern crate md5;
extern crate phf;
extern crate rand;
extern crate tokio;
mod config;
mod message;
use crate::message::SipMessageAttributes;
use crate::message::SIP;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let conf = config::read("./config.json").unwrap();
    let ip = get_if_addrs::get_if_addrs().unwrap()[0].addr.ip();
    println!("[{}] - {:?}", line!(), ip.to_string());

    let mut socket = UdpSocket::bind("0.0.0.0:5060").await.unwrap();

    let command = SIP {
        command: &format!("REGISTER sip:{} SIP/2.0", &ip),
        content_length: "Content-Length: 0",
        to: &format!("To: sip:{}@{}", &conf.username, &ip),
        from: &format!("From: sip:{}@{}", &conf.username, &ip),
        contact: &format!("Contact: sip:{}@{};transport=UDP", &conf.username, &ip),
        cseq: "CSeq: 445 REGISTER",
        call_id: &format!("Call-ID:{}@{}", &SIP::generate_call_id(), &ip),
        via: "Via: SIP/2.0/UDP 185.28.212.48;transport=UDP;branch=57ffd673319367006160043a8bad5ab5",
        user_agent: "User-Agent: sippy 0.2.5",
        allow: "Allow: INVITE,CANCEL,BYE,MESSAGE",
    };

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
