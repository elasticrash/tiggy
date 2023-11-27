use pnet_macros_support::packet::Packet;
use rand::Rng;
use std::{
    f64::consts::PI,
    net::IpAddr,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

use crate::{
    rtp::MutableRtpPacket, rtp::RtpType, state::dialogs::State, transmissions::sockets::MpscBase,
};
use udp_polygon::{config::Address, config::Config, config::FromArguments, Polygon};

#[allow(dead_code)]
const SAMPLE_RATE: f64 = 44_100.0;
#[allow(dead_code)]
const FREQUENCY: f64 = 440.0;
#[allow(dead_code)]
const AMPLITUDE: f32 = 0.25;
#[allow(dead_code)]
const ALAW_MAX: i16 = 0x0FFF;

#[allow(dead_code)]
pub fn rtp_event_loop(
    c_connection: &IpAddr,
    _port: u16,
    dialog_state: Arc<Mutex<State>>,
    r_connection: &IpAddr,
    rtp_port: u16,
) -> JoinHandle<()> {
    let connection = *c_connection;
    let rtp_connection = *r_connection;
    let udp_config = Config::from_arguments(
        vec![Address {
            ip: connection,
            port: rtp_port,
        }],
        Some(Address {
            ip: rtp_connection,
            port: 5060,
        }),
    );

    let mut polygon = Polygon::configure(udp_config);
    let rx = polygon.receive();

    tokio::spawn(async move {
        let mut _rtp_buffer = [0_u8; 65535];
        let mut phase = 0.0;

        let mut rng = rand::thread_rng();
        let n1: u32 = rng.gen();
        let mut n2: u16 = rng.gen();
        let n3: u32 = rng.gen();
        let proper_loop = 0;

        info!("starting rtp event loop");

        'thread: loop {
            let mut send_buffer = [0_u8; 1500];

            let mut state = dialog_state.lock().unwrap();
            let channel = state.get_rtp_channel().unwrap();

            let mut packet = MutableRtpPacket::new(&mut send_buffer).unwrap();
            packet.set_version(2);
            packet.set_payload_type(RtpType::Pcma);
            packet.set_sequence(n2);
            packet.set_timestamp(n1);
            packet.set_ssrc(n3);

            let mut body: [u8; 1405] = [0; 1405];
            for item in &mut body {
                // Generating a sine wave sample
                let mut sample = ((phase * FREQUENCY * 2.0 * PI).sin() as f32 * AMPLITUDE) as i16;

                // Incrementing the phase
                phase += 1.0 / SAMPLE_RATE;
                if phase >= 1.0 {
                    phase -= 1.0;
                }

                let mut mask: u16 = 0x0800;
                let mut sign: u8 = 0;
                let mut position: u8 = 11;

                if sample < 0 {
                    sample = sample.overflowing_neg().0;
                    sign = 0x80;
                }
                if sample > ALAW_MAX {
                    sample = ALAW_MAX;
                }
                while (sample as u16 & mask) != mask && position >= 5 {
                    mask >>= 1;
                    position -= 1;
                }
                let lsb: u8 = if position == 4 {
                    ((sample >> 1) & 0x0f) as u8
                } else {
                    ((sample >> (position - 4)) & 0x0f) as u8
                };
                let output = ((sign | ((position - 4) << 4) | lsb) ^ 0x55) as i8;
                *item = output as u8;
            }

            packet.set_payload(&body);

            channel
                .0
                .send(MpscBase {
                    event: Some(packet.consume_to_immutable().packet().to_vec()),
                    exit: false,
                    ..Default::default()
                })
                .unwrap();

            n2 = proper_loop + 1;

            let maybe_msg = rx.try_recv();

            // distribute message on the correct process
            if let Ok(..) = maybe_msg {
                let msg = maybe_msg.unwrap();
                info!("{}", String::from_utf8_lossy(&msg));
            }

            if let Ok(data) = channel.1.try_recv() {
                if data.exit {
                    break 'thread;
                }
                polygon.send(data.event.unwrap().into());
            }
        }
    })
}
