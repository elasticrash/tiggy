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

use network::get_ipv4;
use rsip::SipMessage;
use startup::registration::{register_ua, unregister_ua};
use state::options::{SelfConfiguration, Verbosity};
use std::collections::VecDeque;
use std::io;
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Duration;
use ui::menu;
use ui::menu::builder::build_menu;
use ui::menu::draw::menu_and_refresh;

use crate::flow::inbound::{process_request_inbound, process_response_inbound};
use crate::transmissions::sockets::{peek, receive};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use flow::outbound::{
    outbound_configure, outbound_start, process_request_outbound, process_response_outbound,
};
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

    let (tx, rx) = mpsc::channel();

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

    let _handler = builder
        .spawn(move || {
            let mut settings = SelfConfiguration {
                flow: Direction::Inbound,
                verbosity: Verbosity::Minimal,
                ip: &ip,
            };

            let mut buffer = [0_u8; 65535];
            let dialog_state: Arc<Mutex<Dialogs>> = Arc::new(Mutex::new(Dialogs::new(5060)));
            {
                let mut locked_state = dialog_state.lock().unwrap();
                let socket = locked_state.get_socket().unwrap();
                let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));

                socket
                    .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                    .expect("connect function failed");
            }
            register_ua(&dialog_state, &conf, &mut settings, &thread_logs);

            'thread: loop {
                let mut maybe_msg: Option<SipMessage> = None;
                {
                    let mut locked_state = dialog_state.lock().unwrap();
                    let mut socket = locked_state.get_socket().unwrap();

                    let packets_queued = peek(&mut socket, &mut buffer);

                    //flog(&vec![&format!("packets queued {}", packets_queued)]);
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

                if let Some(..) = maybe_msg {
                    let msg = maybe_msg.unwrap();
                    match settings.flow {
                        Direction::Inbound => match msg {
                            rsip::SipMessage::Request(request) => process_request_inbound(
                                &request,
                                &conf,
                                &dialog_state,
                                &mut settings,
                                &thread_logs,
                            ),
                            rsip::SipMessage::Response(response) => process_response_inbound(
                                &response,
                                &conf,
                                &dialog_state,
                                &mut settings,
                                &thread_logs,
                            ),
                        },
                        Direction::Outbound => match msg {
                            rsip::SipMessage::Request(request) => process_request_outbound(
                                &request,
                                &conf,
                                &dialog_state,
                                &mut settings,
                                &thread_logs,
                            ),
                            rsip::SipMessage::Response(response) => process_response_outbound(
                                &response,
                                &conf,
                                &dialog_state,
                                &mut settings,
                                &thread_logs,
                            ),
                        },
                    }
                }

                let action_menu = Arc::new(build_menu());

                if let Ok(code) = rx.try_recv() {
                    let mut command = String::from(code);
                    let mut argument: String = "".to_string();
                    log::slog(
                        format!("received command, {}", command).as_str(),
                        &thread_logs,
                    );

                    if command.len() > 1 {
                        let split_command = command.split('|').collect::<Vec<&str>>();
                        argument = split_command[1].to_string();
                        command = split_command[0].to_string();
                    }

                    if !is_string_numeric(argument.clone()) {
                        command = "invalid_argument".to_string();
                    }

                    let key_code_command = KeyCode::Char(command.chars().next().unwrap());

                    match action_menu.iter().find(|&x| x.value == key_code_command) {
                        Some(item) => match item.category {
                            menu::builder::MenuType::Unregister => {
                                unregister_ua(
                                    &dialog_state,
                                    &conf,
                                    &settings.verbosity,
                                    &thread_logs,
                                );
                                break 'thread;
                            }
                            menu::builder::MenuType::Exit => {}
                            menu::builder::MenuType::Silent => {
                                settings.verbosity =
                                    if matches!(settings.verbosity, Verbosity::Diagnostic) {
                                        Verbosity::Minimal
                                    } else {
                                        Verbosity::Diagnostic
                                    }
                            }
                            menu::builder::MenuType::Quiet => settings.verbosity = Verbosity::Quiet,
                            menu::builder::MenuType::Dial => {
                                settings.flow = Direction::Outbound;
                                outbound_configure(&conf, &ip, &argument.clone(), &dialog_state);
                                outbound_start(
                                    &conf,
                                    &dialog_state,
                                    &settings.verbosity,
                                    &thread_logs,
                                );
                            }
                            menu::builder::MenuType::Answer => todo!(),
                            _ => log::slog(
                                format!("{} Not supported", command).as_str(),
                                &thread_logs,
                            ),
                        },
                        None => todo!(),
                    }
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

fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    true
}
