use pnet_macros_support::packet::Packet;
use rand::Rng;
use std::{
    f64::consts::PI,
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

use crate::{
    rtp::MutableRtpPacket,
    rtp::RtpType,
    state::{dialogs::State, options::Verbosity},
    transmissions::sockets::{peek, receive_base, send, MpscBase, SocketV4},
};
use std::time::Duration;

const SAMPLE_RATE: f64 = 8_000.0;
const SAMPLES_PER_PACKET: usize = 160; // 20ms at 8kHz
#[allow(dead_code)]
const FREQUENCY: f64 = 440.0;
#[allow(dead_code)]
const AMPLITUDE: f32 = 1024.0; // ~25% of 12-bit ALAW_MAX (0x0FFF)
#[allow(dead_code)]
const ALAW_MAX: i16 = 0x0FFF;

pub fn rtp_event_loop(
    c_connection: &IpAddr,
    port: u16,
    dialog_state: Arc<Mutex<State>>,
    r_connection: &IpAddr,
    rtp_port: u16,
) -> JoinHandle<()> {
    let connection = *c_connection;
    let rtp_connection = *r_connection;

    tokio::spawn(async move {
        let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();
        let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));
        socket
            .connect(format!("{}:{}", &rtp_connection, &rtp_port))
            .expect("connect function failed");

        let mut rtp_buffer = [0_u8; 65535];
        let mut phase = 0.0;

        let (mut timestamp, mut n2, n3): (u32, u16, u32) = {
            let mut rng = rand::thread_rng();
            (rng.gen(), rng.gen(), rng.gen())
        };

        info!("target rtp located : {:?}:{:?}", rtp_connection, rtp_port);
        info!("source rtp located : {:?}:{}", connection, 49152);
        info!("starting rtp event loop");

        'thread: loop {
            let mut send_buffer = [0_u8; 1500];

            let mut body: [u8; SAMPLES_PER_PACKET] = [0; SAMPLES_PER_PACKET];
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

            let should_exit = {
                let mut state = dialog_state.lock().unwrap();
                let channel = state.get_rtp_channel().unwrap();

                let mut packet = MutableRtpPacket::new(&mut send_buffer).unwrap();
                packet.set_version(2);
                packet.set_payload_type(RtpType::Pcma);
                packet.set_sequence(n2);
                packet.set_timestamp(timestamp);
                packet.set_ssrc(n3);
                packet.set_payload(&body);

                channel
                    .0
                    .send(MpscBase {
                        event: Some(SocketV4 {
                            ip: rtp_connection.to_string(),
                            port: rtp_port,
                            bytes: packet.consume_to_immutable().packet().to_vec(),
                        }),
                        exit: false,
                    })
                    .unwrap();

                let exit = if let Ok(data) = channel.1.try_recv() {
                    if !data.exit {
                        send(&mut socket, &data.event.unwrap(), &Verbosity::Quiet);
                    }
                    data.exit
                } else {
                    false
                };
                exit
            }; // MutexGuard dropped here

            n2 = n2.wrapping_add(1);
            timestamp = timestamp.wrapping_add(SAMPLES_PER_PACKET as u32);

            if should_exit {
                break 'thread;
            }

            tokio::time::sleep(Duration::from_millis(20)).await;

            // peek on the socket, for pending messages
            let mut maybe_msg: Option<Vec<u8>> = None;
            {
                let packets_queued = peek(&mut socket, &mut rtp_buffer);

                if packets_queued > 0 {
                    maybe_msg = Some(receive_base(&mut socket, &mut rtp_buffer));
                    info!("rtp package received");
                }
            }

            // distribute message on the correct process
            if let Some(..) = maybe_msg {
                let msg = maybe_msg.unwrap();
                info!("{}", String::from_utf8_lossy(&msg));
            }
        }
    })
}
