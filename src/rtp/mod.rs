pub mod event_loop;

#[allow(dead_code)]
pub enum RtpType {
    Pcmu, // 0
    Pcma, // 8
    G722, // 9
    Unknown,
}

#[allow(dead_code)]
impl RtpType {
    pub fn new(val: u8) -> Self {
        match val {
            0 => Self::Pcmu,
            8 => Self::Pcma,
            9 => Self::G722,
            _ => Self::Unknown,
        }
    }
}
#[allow(dead_code)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: u8,
    pub extension: u8,
    pub csrc_count: u8,
    pub marker: u8,
    pub payload_type: RtpType,
    pub sequence: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub csrc_list: Vec<u32>,
    pub payload: Vec<u8>,
}
