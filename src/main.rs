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
mod pcap;

use menu::menu_commands::send_menu_commands;
use network::get_ipv4;
use processor::message::{setup_processor, Message, MessageType};
use rocket::fairing::AdHoc;
use rocket::response::status;
use rocket::State;
use state::dialogs::{Dialogs, Direction, UdpCommand};
use state::options::{SelfConfiguration, Verbosity};
use std::sync::mpsc::{self, sync_channel, SyncSender};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use uuid::Uuid;

use crate::pcap::capture;
use crate::startup::registration::unregister_ua;
use crate::transmissions::sockets::MpscBase;

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
    let conf = config::read("./config.json").unwrap();

    let interface = match get_ipv4() {
        Ok(ipv4) => ipv4,
        Err(why) => panic!("{}", why),
    };

    let ip = interface.addr.ip();

    let (mtx, mrx) = sync_channel::<Message>(1);
    let (stx, srx) = setup_processor::<UdpCommand>();
    let (rtx, rrx) = setup_processor::<UdpCommand>();

    let dialog_state: Arc<Mutex<Dialogs>> =
        Arc::new(Mutex::new(Dialogs::new((stx, srx), (rtx, rrx))));

    // Needed to unregister the UA on shutdown
    let exit_config = conf.clone();
    let exit_state = dialog_state.clone();

    let pcap_conf = conf.clone();

    let local_conf = SelfConfiguration {
        flow: Direction::Inbound,
        verbosity: Verbosity::Minimal,
        ip,
    };

    let arc_settings = Arc::new(Mutex::new(local_conf));

    sip::event_loop::sip_event_loop(&conf, &dialog_state, &arc_settings);

    tokio::spawn(async move {
        'thread: loop {
            // send a command for processing
            if let Ok(processable_object) = mrx.try_recv() {
                info!("command received");
                let mut settings = arc_settings.lock().unwrap();

                if send_menu_commands(
                    &processable_object,
                    &dialog_state,
                    &conf,
                    &mut settings,
                    &ip,
                ) {
                    info!("preparing to exit");

                    let mut state = dialog_state.lock().unwrap();
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

    tokio::spawn(async move {
        capture(&interface, &Uuid::new_v4(), &pcap_conf.pcap);
    });

    rocket::build()
        .manage(mtx)
        .mount("/", routes![make_call, toggle_log])
        .attach(AdHoc::on_shutdown("Shutdown Printer", |_| {
            Box::pin(async move {
                info!("sending unregister command");
                unregister_ua(&exit_state, &exit_config);
                thread::sleep(Duration::from_secs(1));
            })
        }))
}
