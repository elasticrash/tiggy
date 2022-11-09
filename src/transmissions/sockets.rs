use crate::{slog::udp_logger, state::options::Verbosity};
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
pub fn send(socket: &mut UdpSocket, data: &SocketV4, vrb: &Verbosity) {
    udp_logger(
        Paint::yellow(String::from_utf8_lossy(&data.bytes).to_string()).to_string(),
        vrb,
    );

    socket
        .send_to(&data.bytes, format!("{}:{}", &data.ip, &data.port))
        .unwrap();
}

/// Receives a message through upd
pub fn receive(
    socket: &mut UdpSocket,
    buffer: &mut [u8; 65535],
    vrb: &Verbosity,
) -> Result<SipMessage, rsip::Error> {
    let slice = receive_base(socket, buffer);
    let r_message_a = String::from_utf8_lossy(&slice);
    udp_logger(Paint::green(r_message_a.to_string()).to_string(), vrb);

    SipMessage::try_from(r_message_a.to_string())
}

pub fn receive_base(socket: &mut UdpSocket, buffer: &mut [u8; 65535]) -> Vec<u8> {
    let (amt, _src) = socket.recv_from(buffer).unwrap();
    let slice = &mut buffer[..amt];
    slice.to_vec()
}

/// Take a look on socket whether a message is available without picking it up
/// Returns number of messages awaiting to be received
pub fn peek(socket: &mut UdpSocket, buffer: &mut [u8]) -> usize {
    match socket.peek(buffer) {
        Ok(received) => received,
        Err(_e) => 0,
    }
}
