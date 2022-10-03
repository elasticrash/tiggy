use crate::{
    log::{print_msg, MTLogs},
    state::options::Verbosity,
};
use rsip::SipMessage;
use std::{convert::TryFrom, net::UdpSocket};

pub struct SocketV4 {
    pub ip: String,
    pub port: u16,
}

pub fn send(
    s_conf: &SocketV4,
    socket: &mut UdpSocket,
    msg: String,
    vrb: &Verbosity,
    logs: &MTLogs,
) {
    print_msg("===>".to_string(), vrb, logs);
    print_msg(msg.clone(), vrb, logs);

    socket
        .send_to(msg.as_bytes(), format!("{}:{}", s_conf.ip, s_conf.port))
        .unwrap();
}

pub fn receive(
    socket: &mut UdpSocket,
    buffer: &mut [u8; 65535],
    vrb: &Verbosity,
    logs: &MTLogs,
) -> Result<SipMessage, rsip::Error> {
    let (amt, _src) = socket.recv_from(buffer).unwrap();
    let slice = &mut buffer[..amt];
    let r_message_a = String::from_utf8_lossy(slice);
    print_msg("<===".to_string(), vrb, logs);
    print_msg(r_message_a.to_string(), vrb, logs);

    SipMessage::try_from(r_message_a.to_string())
}

pub fn peek(socket: &mut UdpSocket, buffer: &mut [u8]) -> usize {
    match socket.peek(buffer) {
        Ok(received) => received,
        Err(_e) => 0,
    }
}
