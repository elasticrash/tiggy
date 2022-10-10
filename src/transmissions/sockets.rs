use crate::{
    log::{print_msg, MTLogs},
    state::options::Verbosity,
};
use rsip::SipMessage;
use std::{convert::TryFrom, net::UdpSocket};

/// Bundle Ip and Port for Upd connections into a signle struct
pub struct SocketV4 {
    pub ip: String,
    pub port: u16,
}

/// Sends a udp message
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

/// Receives a message through upd
/// Returns a SIPMessage type
/// * TODO: Think about returning something more generic (i.e. String)
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

/// Take a look on socket whether a message is available without picking it up
/// Returns number of messages awaiting to be received
pub fn peek(socket: &mut UdpSocket, buffer: &mut [u8]) -> usize {
    match socket.peek(buffer) {
        Ok(received) => received,
        Err(_e) => 0,
    }
}
