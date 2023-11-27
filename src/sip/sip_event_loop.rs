use crate::{slog::udp_logger, state::options::Verbosity};
use dns_lookup::lookup_host;
use rsip::SipMessage;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;
use yansi::Paint;

use crate::{
    config::JSONConfiguration,
    flow::{
        inbound::{process_request_inbound, process_response_inbound},
        outbound::{process_request_outbound, process_response_outbound},
    },
    state::{
        dialogs::{Direction, State},
        options::SelfConfiguration,
    },
};
use udp_polygon::{config::Address, config::Config, config::FromArguments, Polygon};

const VERBOSITY: Verbosity = Verbosity::Minimal;

pub fn sip_event_loop(
    c_conf: &JSONConfiguration,
    c_dialog_state: Arc<Mutex<State>>,
    c_settings: &Arc<Mutex<SelfConfiguration>>,
) -> JoinHandle<()> {
    let state: Arc<Mutex<State>> = c_dialog_state;
    let arc_settings: Arc<Mutex<SelfConfiguration>> = Arc::clone(c_settings);
    let conf = c_conf.clone();

    let ips: Vec<std::net::IpAddr> = lookup_host(&conf.sip_server).unwrap();

    let udp_config = Config::from_arguments(
        vec![Address {
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: conf.sip_port,
        }],
        Some(Address {
            ip: ips[0],
            port: conf.sip_port,
        }),
    );

    println!("Listening on: {:?}", udp_config);

    let mut polygon = Polygon::configure(udp_config);
    let rx = polygon.receive();

    tokio::spawn(async move {
        let dialog_state = state;

        let mut _sip_buffer = [0_u8; 65535];

        'thread: loop {
            // peek on the socket, for pending messages
            let maybe_msg = rx.try_recv();

            // distribute message on the correct process
            if let Ok(..) = maybe_msg {
                udp_logger(
                    Paint::yellow(String::from_utf8_lossy(&maybe_msg.clone().unwrap()).to_string())
                        .to_string(),
                    &VERBOSITY,
                );

                let msg = SipMessage::try_from(maybe_msg.unwrap()).unwrap();
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
                if data.override_default_destination.is_some() {
                    polygon.change_destination(SocketAddr::new(
                        data.override_default_destination.clone().unwrap().ip,
                        data.override_default_destination.clone().unwrap().port,
                    ));
                }
                if data.exit {
                    break 'thread;
                }
                udp_logger(
                    Paint::yellow(
                        String::from_utf8_lossy(&data.event.clone().unwrap()).to_string(),
                    )
                    .to_string(),
                    &VERBOSITY,
                );
                polygon.send(data.event.unwrap());
                polygon.change_destination(SocketAddr::new(ips[0], conf.sip_port))
            }
        }
    })
}
