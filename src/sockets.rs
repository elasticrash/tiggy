use crate::log::{log_in, log_out, print_msg};
use rsip::SipMessage;
use std::{
    collections::VecDeque,
    convert::TryFrom,
    net::UdpSocket,
    sync::{Arc, Mutex},
};

pub struct SocketV4 {
    pub ip: String,
    pub port: u16,
}

pub fn send(
    s_conf: &SocketV4,
    msg: String,
    socket: &mut UdpSocket,
    s: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    log_out(&logs);
    print_msg(msg.clone(), s, logs);

    socket
        .send_to(msg.as_bytes(), format!("{}:{}", s_conf.ip, s_conf.port))
        .unwrap();
}

pub fn receive(
    socket: &mut UdpSocket,
    buffer: &mut [u8; 65535],
    s: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> Result<SipMessage, rsip::Error> {
    log_in(&logs);

    let (amt, _src) = socket.recv_from(buffer).unwrap();
    let slice = &mut buffer[..amt];
    let r_message_a = String::from_utf8_lossy(&slice);
    print_msg(r_message_a.to_string(), s, &logs);

    SipMessage::try_from(r_message_a.to_string())
}

pub fn peek(socket: &mut UdpSocket, buffer: &mut [u8]) -> usize {
    match socket.peek(buffer) {
        Ok(received) => received,
        Err(_e) => 0,
    }
}
