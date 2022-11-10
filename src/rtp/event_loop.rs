use std::{
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

use crate::{
    state::{dialogs::Dialogs, options::Verbosity},
    transmissions::sockets::{peek, receive_base, send},
};
use std::time::Duration;

pub fn rtp_event_loop(
    c_connection: &IpAddr,
    port: u16,
    dialog_state: Arc<Mutex<Dialogs>>,
) -> JoinHandle<()> {
    let connection = *c_connection;
    tokio::spawn(async move {
        let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", 40024)).unwrap();
        let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));
        socket
            .connect(format!("{}:{}", &connection, &port))
            .expect("connect function failed");

        let mut rtp_buffer = [0_u8; 65535];

        'thread: loop {
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

            let mut state = dialog_state.lock().unwrap();
            let channel = state.get_rtp_channel().unwrap();

            if let Ok(data) = channel.1.try_recv() {
                if data.exit {
                    break 'thread;
                }
                send(&mut socket, &data.event.unwrap(), &Verbosity::Quiet);
            }
        }
    })
}
