use std::{net::UdpSocket, convert::TryFrom};
use rsip::SipMessage;
use crate::{
    log::{log_out, print_msg, log_in},
    models::SocketV4,
};

pub fn send(s_conf: &SocketV4, msg: String, socket: &mut UdpSocket, s: bool) {
    if !s {
        log_out();
    }
    print_msg(msg.clone(), s);

    socket
        .send_to(msg.as_bytes(), format!("{}:{}", s_conf.ip, s_conf.port))
        .unwrap();
}

pub fn receive(
    socket: &mut UdpSocket,
    buffer: &mut [u8; 65535],
    s: bool,
) -> Result<SipMessage, rsip::Error> {
    if !s {
        log_in();
    }

    let (amt, _src) = socket.recv_from(buffer).unwrap();
    let slice = &mut buffer[..amt];
    let r_message_a = String::from_utf8_lossy(&slice);
    print_msg(r_message_a.to_string(), s);

    SipMessage::try_from(r_message_a.to_string())
}

pub fn peek(socket: &mut UdpSocket, buffer: &mut [u8]) -> usize {
    match socket.peek(buffer) {
        Ok(received) => received,
        Err(_e) => 0,
    }
}
