use pnet_macros::packet;
use pnet_macros_support::packet::PrimitiveValues;
use pnet_macros_support::types::{u1, u16be, u2, u32be, u4, u7};

pub mod event_loop;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RtpType {
    Pcmu, // 0
    Pcma, // 8
    G722, // 9
    Unknown,
}

impl RtpType {
    pub fn new(val: u7) -> Self {
        match val {
            0 => Self::Pcmu,
            8 => Self::Pcma,
            9 => Self::G722,
            _ => Self::Unknown,
        }
    }
}
#[packet]
pub struct Rtp {
    pub version: u2,
    pub padding: u1,
    pub extension: u1,
    pub csrc_count: u4,
    pub marker: u1,
    #[construct_with(u7)]
    pub payload_type: RtpType,
    pub sequence: u16be,
    pub timestamp: u32be,
    pub ssrc: u32be,
    #[length = "csrc_count"]
    pub csrc_list: Vec<u32be>,
    #[payload]
    pub payload: Vec<u8>,
}

impl PrimitiveValues for RtpType {
    type T = (u7,);

    fn to_primitive_values(&self) -> Self::T {
        match self {
            Self::Pcmu => (0,),
            Self::Pcma => (8,),
            Self::G722 => (9,),
            _ => panic!("unsuported value"),
        }
    }
}
