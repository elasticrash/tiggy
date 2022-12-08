use rsip::SipMessage;
use std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

use crate::{
    config::JSONConfiguration,
    flow::{
        inbound::{process_request_inbound, process_response_inbound},
        outbound::{process_request_outbound, process_response_outbound},
    },
    state::{
        dialogs::{State, Direction},
        options::{SelfConfiguration, Verbosity},
    },
    transmissions::sockets::{peek, receive, send},
};
use std::time::Duration;

pub fn sip_event_loop(
    c_conf: &JSONConfiguration,
    c_dialog_state: Arc<Mutex<State>>,
    c_settings: &Arc<Mutex<SelfConfiguration>>,
) -> JoinHandle<()> {
    let state: Arc<Mutex<State>> = c_dialog_state;
    let arc_settings: Arc<Mutex<SelfConfiguration>> = Arc::clone(c_settings);
    let conf = c_conf.clone();

    tokio::spawn(async move {
        let dialog_state = state;

        let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", 5060)).unwrap();
        let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));
        socket
            .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
            .expect("connect function failed");

        let verbosity: Verbosity;
        let mut sip_buffer = [0_u8; 65535];
        {
            let settings = arc_settings.lock().unwrap();
            verbosity = settings.verbosity.clone();
        }

        'thread: loop {
            // peek on the socket, for pending messages
            let mut maybe_msg: Option<SipMessage> = None;
            {
                let packets_queued = peek(&mut socket, &mut sip_buffer);

                if packets_queued > 0 {
                    maybe_msg = match receive(&mut socket, &mut sip_buffer, &verbosity) {
                        Ok(buf) => Some(buf),
                        Err(_) => None,
                    };
                }
            }

            // distribute message on the correct process
            if let Some(..) = maybe_msg {
                let msg = maybe_msg.unwrap();
                let mut settings = arc_settings.lock().unwrap();
                {
                    info!("match flow, {}", settings.flow);
                    match settings.flow {
                        Direction::Inbound => match msg {
                            rsip::SipMessage::Request(request) => process_request_inbound(
                                &request,
                                &conf,
                                &dialog_state,
                                &mut settings,
                            ),
                            rsip::SipMessage::Response(response) => {
                                process_response_inbound(&response, &conf, &dialog_state)
                            }
                        },
                        Direction::Outbound => match msg {
                            rsip::SipMessage::Request(request) => process_request_outbound(
                                &request,
                                &conf,
                                &dialog_state,
                                &mut settings,
                            ),
                            rsip::SipMessage::Response(response) => process_response_outbound(
                                &response,
                                &conf,
                                &dialog_state,
                                &mut settings,
                            ),
                        },
                    }
                }
            }

            let mut state = dialog_state.lock().unwrap();
            let channel = state.get_sip_channel().unwrap();

            if let Ok(data) = channel.1.try_recv() {
                if data.exit {
                    break 'thread;
                }
                send(&mut socket, &data.event.unwrap(), &verbosity);
            }
        }
    })
}
