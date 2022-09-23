extern crate md5;
extern crate rand;
mod commands;
mod composer;
mod config;
mod flow;
mod log;
mod startup;
mod state;
mod transmissions;
mod ui;

use startup::registration::register_ua;
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
use log::print_msg;
use state::dialogs::{Dialogs, Direction};
use std::net::UdpSocket;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;
use ui::app::App;

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("[{}] - {}", line!(), e);
                continue;
            }
        }
    };
}

fn main() -> Result<(), io::Error> {
    let logs = Arc::new(Mutex::from(VecDeque::new()));
    let thread_logs = Arc::clone(&logs);

    let conf = config::read("./config.json").unwrap();
    let is_there_an_ipv4 = if_addrs::get_if_addrs().unwrap().into_iter().find(|ip| {
        print_msg(
            format!("available interface:, {}", ip.addr.ip()),
            false,
            &logs,
        );
        ip.ip().is_ipv4()
    });

    let ip = match is_there_an_ipv4 {
        Some(ipv4) => ipv4,
        None => panic!("could not find an ipv4 interface"),
    }
    .addr
    .ip();

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
            let mut silent = true;
            let mut flow = Direction::Inbound;

            let mut buffer = [0_u8; 65535];

            let mut socket = UdpSocket::bind("0.0.0.0:5060").unwrap();
            let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));

            socket
                .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                .expect("connect function failed");

            let dialog_state: Arc<Mutex<Dialogs>> = Arc::new(Mutex::new(Dialogs::new()));

            register_ua(&dialog_state, &conf, &ip, &mut socket, silent, &thread_logs);

            'thread: loop {
                let packets_queued = peek(&mut socket, &mut buffer);

                if packets_queued > 0 {
                    let msg = skip_fail!(receive(&mut socket, &mut buffer, silent, &thread_logs));

                    match flow {
                        Direction::Inbound => match msg {
                            rsip::SipMessage::Request(request) => process_request_inbound(
                                &request,
                                &mut socket,
                                &conf,
                                &ip,
                                silent,
                                &thread_logs,
                            ),
                            rsip::SipMessage::Response(response) => process_response_inbound(
                                &response,
                                &mut socket,
                                &conf,
                                &dialog_state,
                                silent,
                                &thread_logs,
                            ),
                        },
                        Direction::Outbound => match msg {
                            rsip::SipMessage::Request(request) => process_request_outbound(
                                &request,
                                &mut socket,
                                &conf,
                                &ip,
                                &dialog_state,
                                silent,
                                &mut flow,
                                &thread_logs,
                            ),
                            rsip::SipMessage::Response(response) => process_response_outbound(
                                &response,
                                &mut socket,
                                &conf,
                                &ip,
                                &dialog_state,
                                silent,
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
                            menu::builder::MenuType::Exit => {
                                break 'thread;
                            }
                            menu::builder::MenuType::Silent => {
                                silent = !silent;
                            }
                            menu::builder::MenuType::Dial => {
                                print_msg("outbound_configure".to_string(), true, &thread_logs);
                                flow = Direction::Outbound;
                                outbound_configure(&conf, &ip, &argument.clone(), &dialog_state);
                                outbound_start(
                                    &mut socket,
                                    &conf,
                                    &dialog_state,
                                    silent,
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
