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
//mod rtp;

/// SIP
mod sip;

use menu::menu_commands::send_menu_commands;
use network::get_ipv4;
use processor::message::{setup_processor, Message, MessageType};
use rocket::response::status;
use rocket::State;
use slog::{flog, MTLogs};
use state::dialogs::{Dialogs, Direction};
use state::options::{SelfConfiguration, Verbosity};
use std::collections::VecDeque;
use std::sync::mpsc::{self, sync_channel, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self};
use transmissions::sockets::SocketV4;

use crate::transmissions::sockets::MpscBase;

#[macro_use]
extern crate rocket;

#[post("/call/<number>")]
fn make_call(tr: &State<SyncSender<Message>>, number: &str) -> status::Accepted<String> {
    info!("sending dial command with {}", )
    tr.try_send(Message::new(
        MessageType::MenuCommand,
        'd',
        Some(number.to_string()),
    ))
    .unwrap();
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
    let logs: MTLogs = Arc::new(Mutex::from(VecDeque::new()));
    let sip_logs: MTLogs = Arc::clone(&logs);
    let rtp_logs: MTLogs = Arc::clone(&logs);

    let conf = config::read("./config.json").unwrap();

    let ip = match get_ipv4() {
        Ok(ipv4) => ipv4,
        Err(why) => panic!("{}", why),
    };

    flog(&vec![{ &format!("IP found {}", ip) }]);

    let (mtx, mrx) = sync_channel::<Message>(1);
    let (stx, srx) = setup_processor::<MpscBase<SocketV4>>();
    let (rtx, rrx) = setup_processor::<MpscBase<SocketV4>>();

    info!("Using: {}", ip);

    let dialog_state: Arc<Mutex<Dialogs>> =
        Arc::new(Mutex::new(Dialogs::new((stx, srx), (rtx, rrx))));

    let arc_settings = Arc::new(Mutex::new(SelfConfiguration {
        flow: Direction::Inbound,
        verbosity: Verbosity::Minimal,
        ip: ip,
    }));

    sip::event_loop::sip_event_loop(&conf, &dialog_state, &arc_settings, &sip_logs);

    tokio::spawn(async move {
        'thread: loop {
            // send a command for processing
            if let Ok(processable_object) = mrx.try_recv() {
               info!("received input, {:?}", processable_object.bind);
                let mut settings = arc_settings.lock().unwrap();

                if send_menu_commands(
                    &processable_object,
                    &dialog_state,
                    &conf,
                    &mut settings,
                    &ip,
                    &logs,
                ) {
                    let mut state = dialog_state.lock().unwrap();
                    let channel = state.get_channel().unwrap();
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

    rocket::build()
        .manage(mtx)
        .mount("/", routes![make_call, toggle_log])
}
