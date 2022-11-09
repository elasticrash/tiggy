use crate::{
    slog::{print_msg, MTLogs},
    state::options::Verbosity,
};
use rsip::SipMessage;
use std::{convert::TryFrom, net::UdpSocket};
use yansi::Paint;

pub struct MpscBase<T> {
    pub event: Option<T>,
    pub exit: bool,
}

/// Bundle Ip, Port and payload for Upd connections into a single struct
pub struct SocketV4 {
    pub ip: String,
    pub port: u16,
    pub bytes: Vec<u8>,
}

/// Sends a udp message
pub fn send(socket: &mut UdpSocket, data: &SocketV4, vrb: &Verbosity, logs: &MTLogs) {
    print_msg(
        Paint::yellow(String::from_utf8_lossy(&data.bytes).to_string()).to_string(),
        vrb,
        logs,
    );

    socket
        .send_to(&data.bytes, format!("{}:{}", &data.ip, &data.port))
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
    print_msg(Paint::green(r_message_a.to_string()).to_string(), vrb, logs);

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
