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
/// Logging
mod log;
/// Gets IP information
mod network;
/// Actions that need to happen when the UA starts
mod startup;
/// SIP State
mod state;
/// Upd
mod transmissions;
/// Terminal UI
mod ui;

/// Processor
mod processor;

use network::get_ipv4;
use processor::message::{setup_processor, Message};
use rsip::SipMessage;
use startup::registration::register_ua;
use state::options::{SelfConfiguration, Verbosity};
use std::collections::VecDeque;
use std::io;
use std::net::UdpSocket;
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;
use transmissions::sockets::{send, SocketV4};
use ui::menu::builder::build_menu;
use ui::menu::draw::{menu_and_refresh, send_menu_commands};

use crate::flow::inbound::{process_request_inbound, process_response_inbound};
use crate::transmissions::sockets::{peek, receive};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use flow::outbound::{process_request_outbound, process_response_outbound};
use log::MTLogs;
use state::dialogs::{Dialogs, Direction};
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;
use ui::app::App;

fn main() -> Result<(), io::Error> {
    let logs: MTLogs = Arc::new(Mutex::from(VecDeque::new()));
    let thread_logs: MTLogs = Arc::clone(&logs);

    let conf = config::read("./config.json").unwrap();

    let ip = match get_ipv4() {
        Ok(ipv4) => ipv4,
        Err(why) => panic!("{}", why),
    };

    log::slog(&format!("IP found {} :", ip), &logs);

    let (tx, rx) = setup_processor::<Message>();
    let (stx, srx) = setup_processor::<SocketV4>();

    logs.lock().unwrap().push_back(format!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        ip
    ));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    let builder = thread::Builder::new();
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", 5060)).unwrap();

    let _handler = builder
        .spawn(move || {
            let mut settings = SelfConfiguration {
                flow: Direction::Inbound,
                verbosity: Verbosity::Minimal,
                ip: &ip,
            };

            let mut buffer = [0_u8; 65535];

            let dialog_state: Arc<Mutex<Dialogs>> = Arc::new(Mutex::new(Dialogs::new(stx)));
            {
                let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));

                socket
                    .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                    .expect("connect function failed");
            }

            register_ua(&dialog_state, &conf, &mut settings);
            let action_menu = Arc::new(build_menu());

            'thread: loop {
                // peek on the socket, for pending messages
                let mut maybe_msg: Option<SipMessage> = None;
                {
                    let packets_queued = peek(&mut socket, &mut buffer);

                    if packets_queued > 0 {
                        maybe_msg = match receive(
                            &mut socket,
                            &mut buffer,
                            &settings.verbosity,
                            &thread_logs,
                        ) {
                            Ok(buf) => Some(buf),
                            Err(_) => None,
                        };
                    }
                }

                // distribute message on the correct process
                if let Some(..) = maybe_msg {
                    let msg = maybe_msg.unwrap();
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

                // send a command for processing
                if let Ok(processable_object) = rx.try_recv() {
                    log::slog(
                        format!("received input, {:?}", processable_object.bind).as_str(),
                        &thread_logs,
                    );

                    if send_menu_commands(
                        &processable_object,
                        &dialog_state,
                        &action_menu,
                        &conf,
                        &mut settings,
                        &ip,
                        &thread_logs,
                    ) {
                        break 'thread;
                    }
                }

                if let Ok(data) = srx.try_recv() {
                    send(&mut socket, &data, &settings.verbosity, &thread_logs);
                }
            }
        })
        .unwrap();

    // create app and run it
    let app: App = App::default();
    let res = menu_and_refresh(&mut terminal, &tx, &logs, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
