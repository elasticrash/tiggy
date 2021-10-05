extern crate md5;
extern crate phf;
extern crate rand;
extern crate tokio;
mod config;
mod models;
mod register;
use std::{convert::TryInto, thread, time::Duration};

use config::JSONConfiguration;
use tokio::net::UdpSocket;

use crate::{models::SIP, register::SipMessageRegister};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let conf = config::read("./config.json").unwrap();
    let ip = get_if_addrs::get_if_addrs().unwrap()[0].addr.ip();
    println!("[{}] - {:?}", line!(), ip.to_string());
    let mut buffer = [0 as u8; 65535];

    let mut socket = UdpSocket::bind("0.0.0.0:5060").await.unwrap();

    let mut blank = SIP {
        history: Vec::new(),
    };
    let r1_message_init = blank.generate_register_request(&conf.clone(), &ip.clone().to_string());

    send_and_receive(&conf.clone(), r1_message_init.to_string(), &mut socket, &mut buffer).await;
    let r2_message_init = blank.add_authentication(
        r1_message_init.try_into().unwrap(),
        &conf.clone(),
        &ip.clone().to_string(),
    );
    send_and_receive(&conf.clone(), r2_message_init.to_string(), &mut socket, &mut buffer).await;
    thread::sleep(Duration::from_secs(1));
    receive(&mut socket, &mut buffer).await;
    Ok(())
}

async fn send_and_receive(conf: &JSONConfiguration, msg: String, socket: &mut UdpSocket, buffer: &mut [u8; 65535]) {
    print_send();
    print_msg(msg.clone());

    socket
        .send_to(msg.as_bytes(), format!("{}:5060", &conf.sip_server))
        .await
        .unwrap();
    print_received();
    let (amt, src) = socket.recv_from(buffer).await.unwrap();
    let slice = &mut buffer[..amt];
    let r_message_a = String::from_utf8_lossy(&slice);
    print_msg(r_message_a.to_string());
    //TODO: parse and return response
}

async fn receive(socket: &mut UdpSocket, buffer: &mut [u8; 65535]){
    let (amt, src) = socket.recv_from(buffer).await.unwrap();
    let slice = &mut buffer[..amt];
    let r_message_a = String::from_utf8_lossy(&slice);
    print_msg(r_message_a.to_string());
}

fn print_send() {
    println!("[{}] - {:?}", line!(), ">>>>>>>>>>>>>");
}

fn print_received() {
    println!("[{}] - {:?}", line!(), "<<<<<<<<<<<<<");
}

fn print_msg(msg: String) {
    let print = msg.split("\r\n");
    for line in print {
        println!("[{}] - {:?}", line!(), line);
    }
}
