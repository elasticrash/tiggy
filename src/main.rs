extern crate md5;
extern crate rand;
/// Composes SIP Messages
mod commands;
/// Extentions on rsip
mod composer;
/// Loads configuration
mod config;
/// Call Flows
mod flow;
/// Gets IP information
mod network;
/// Logging
mod slog;
/// Actions that need to happen when the UA starts
mod startup;
/// SIP State
mod state;
/// Upd
mod transmissions;

/// Processor
mod processor;

/// Available Commands
mod menu;

/// RTP
mod rtp;

/// SIP
mod sip;

///PCAP
// mod pcap;
use menu::menu_commands::send_menu_commands;
use network::get_ipv4;
use processor::message::{setup_processor, Message, MessageType};
use rocket::fairing::AdHoc;
use rocket::response::status;
use rocket::State;
use state::dialogs::{Direction, State as SipState, UdpCommand};
use state::options::{SelfConfiguration, Verbosity};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
// use uuid::Uuid;

// use crate::pcap::capture;
use crate::startup::registration::unregister_ua;
use crate::transmissions::sockets::MpscBase;
use std::net::IpAddr;
use std::net::Ipv4Addr;

#[macro_use]
extern crate rocket;

#[post("/call/<number>")]
fn make_call(tr: &State<SyncSender<Message>>, number: &str) -> status::Accepted<String> {
    info!("sending dial command with {}", number);
    let receipt = tr.try_send(Message::new(
        MessageType::MenuCommand,
        'd',
        Some(number.to_string()),
    ));
    match receipt {
        Ok(_) => info!("command send"),
        Err(err) => error!("{:?}", err),
    };
    status::Accepted(Some(format!("number: '{}'", number)))
}

#[post("/log")]
fn toggle_log(tr: &State<SyncSender<Message>>) -> status::Accepted<String> {
    tr.try_send(Message::new(MessageType::MenuCommand, 's', None))
        .unwrap();
    status::Accepted(Some("Log toggled".to_string()))
}

#[launch]
fn rocket() -> _ {
    let conf = config::read("./edify.json").unwrap();

    let interface = match get_ipv4() {
        Ok(ipv4) => ipv4,
        Err(why) => panic!("{}", why),
    };

    let ip = interface.addr.ip();

    // PCAP
    // let pcap_conf = conf.clone();
    // tokio::spawn(async move {
    //     capture(&interface, &Uuid::new_v4(), &pcap_conf.pcap);
    // });

    // wait until pcap starts, this needs improvement, needs a feedback from pcap
    thread::sleep(Duration::from_secs(2));

    let (mtx, mrx) = sync_channel::<Message>(1);
    let (stx, srx) = setup_processor::<UdpCommand>();
    let (rtx, rrx) = setup_processor::<UdpCommand>();

    let dialog_state: Arc<Mutex<SipState>> =
        Arc::new(Mutex::new(SipState::new((stx, srx), (rtx, rrx))));

    let reg_state = dialog_state.clone();
    let sip_state = dialog_state.clone();
    let http_state = dialog_state.clone();
    let publisher_state = dialog_state.clone();

    // Needed to unregister the UA on shutdown
    let exit_config = conf.clone();

    let rocket = rocket::build()
        .manage(mtx)
        .mount("/", routes![make_call, toggle_log])
        .attach(AdHoc::on_shutdown("Shutdown Printer", |_| {
            Box::pin(async move {
                info!("sending unregister command");
                unregister_ua(http_state, &exit_config);
                // this needs improvements, needs feedback from pcap, after SIGINT, to stop capturing packets
                thread::sleep(Duration::from_secs(3));
            })
        }));

    let local_conf = SelfConfiguration {
        flow: Direction::Inbound,
        verbosity: Verbosity::Extreme,
        ip,
    };

    let arc_settings = Arc::new(Mutex::new(local_conf));

    sip::register_event_loop::reg_event_loop(&conf, reg_state, ip);
    sip::sip_event_loop::sip_event_loop(&conf, sip_state, &arc_settings);

    tokio::spawn(async move {
        'thread: loop {
            let command_state = dialog_state.clone();

            // send a command for processing
            if let Ok(processable_object) = mrx.try_recv() {
                info!("command received");
                let mut settings = arc_settings.lock().unwrap();

                if send_menu_commands(
                    &processable_object,
                    command_state,
                    &conf,
                    &mut settings,
                    &ip,
                ) {
                    info!("preparing to exit");

                    let mut state = publisher_state.lock().unwrap();
                    let channel = state.get_sip_channel().unwrap();
                    channel
                        .0
                        .send(MpscBase {
                            event: None,
                            exit: true,
                        })
                        .unwrap();
                    break 'thread;
                }
            }
        }
    });

    rocket
}
