extern crate md5;
extern crate phf;
extern crate rand;
mod commands;
mod composer;
mod config;
mod flow;
mod log;
mod menu;
mod sockets;
mod ui;

use std::collections::VecDeque;
use std::io;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::time::Instant;
use std::{convert::TryFrom, time::Duration};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use flow::outbound::{
    outbound_configure, outbound_request_flow, outbound_response_flow, outbound_start,
};
use flow::Flow;
use rsip::{Method, Response};
use std::net::UdpSocket;
use tui::backend::{Backend, CrosstermBackend};
use tui::widgets::{Block, Borders};
use tui::Terminal;
use ui::app::{ui, App, InputMode};

use crate::composer::communication::Call;
use crate::flow::inbound::{inbound_request_flow, inbound_response_flow, inbound_start};
use crate::sockets::{peek, receive, send, SocketV4};

use crate::menu::builder::build_menu;

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
    let conf = config::read("./config.json").unwrap();
    let ip = get_if_addrs::get_if_addrs().unwrap()[0].addr.ip();
    let (tx, rx) = mpsc::channel();
    let logs = Arc::new(Mutex::from(VecDeque::new()));

    logs.lock().unwrap().push_back(format!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        ip.to_string()
    ));

    let thread_logs = Arc::clone(&logs);

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
            let mut silent = false;
            let mut flow = Flow::Inbound;

            let mut buffer = [0 as u8; 65535];

            let mut socket = UdpSocket::bind("0.0.0.0:5060").unwrap();
            let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));

            socket
                .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                .expect("connect function failed");

            let mut count: i32 = 0;
            let shared_in = inbound_start(&conf, &ip);
            let shared_out = outbound_configure(&conf, &ip);

            'thread: loop {
                if count == 0 {
                    send(
                        &SocketV4 {
                            ip: conf.clone().sip_server,
                            port: conf.clone().sip_port,
                        },
                        shared_in.borrow().reg.ask().to_string(),
                        &mut socket,
                        silent,
                        &thread_logs,
                    );
                }

                let packet_size = peek(&mut socket, &mut buffer);

                if packet_size > 0 {
                    let msg = skip_fail!(receive(&mut socket, &mut buffer, silent, &thread_logs));

                    match flow {
                        Flow::Inbound => {
                            if msg.is_response() {
                                let response = Response::try_from(msg.clone()).unwrap();

                                inbound_response_flow(
                                    &response,
                                    &mut socket,
                                    &conf,
                                    &shared_in,
                                    silent,
                                    &thread_logs,
                                );
                            } else {
                                inbound_request_flow(
                                    &msg,
                                    &mut socket,
                                    &conf,
                                    &ip,
                                    silent,
                                    &thread_logs,
                                );
                            }
                        }
                        Flow::Outbound => {
                            if msg.is_response() {
                                let response = Response::try_from(msg.clone()).unwrap();
                                outbound_response_flow(&response, &shared_out);
                            } else {
                                let inb_msg = outbound_request_flow(&msg);
                                if inb_msg == Method::Bye {
                                    flow = Flow::Inbound;
                                }
                            }
                        }
                    }
                }
                count += 1;

                let action_menu = Arc::new(build_menu());

                match rx.try_recv() {
                    Ok(code) => {
                        let mut command = String::from(code);
                        let mut argument: String = "".to_string();
                        log::slog(
                            format!("received command, {}", command.to_string()).as_str(),
                            &thread_logs,
                        );

                        if command.len() > 1 {
                            let split_command = command.split("|").collect::<Vec<&str>>();
                            argument = split_command[1].to_string();
                            command = split_command[0].to_string();
                        }

                        if !is_string_numeric(argument.clone()) {
                            command = "invalid_argument".to_string();
                        }

                        let key_code_command = KeyCode::Char(command.chars().nth(0).unwrap());

                        match action_menu.iter().find(|&x| x.value == key_code_command) {
                            Some(item) => match item.category {
                                menu::builder::MenuType::Exit => {
                                    break 'thread;
                                }
                                menu::builder::MenuType::Silent => {
                                    silent = !silent;
                                }
                                menu::builder::MenuType::Dial => {
                                    flow = Flow::Outbound;
                                    {
                                        let mut shared = shared_out.borrow_mut();
                                        shared.inv.cld = Some(argument);
                                    }
                                    outbound_start(
                                        &mut socket,
                                        &conf,
                                        &shared_out,
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
                    Err(_) => {}
                }
            }
        })
        .unwrap();

    // create app and run it
    let app: App = App::default();
    let res = run_app(&mut terminal, &tx, &logs, app);

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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    tx: &Sender<String>,
    logs: &Arc<Mutex<VecDeque<String>>>,
    mut app: App,
) -> io::Result<()> {
    let cmd_menu = Arc::new(build_menu());
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app, &logs))?;
        let timeout = app
            .tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match cmd_menu.iter().find(|&x| x.value == key.code) {
                        Some(item) => {
                            let key_value = match item.value {
                                KeyCode::Char(c) => c,
                                _ => 'u',
                            };

                            match item.category {
                                menu::builder::MenuType::DisplayMenu => {
                                    log::print_menu();
                                }
                                menu::builder::MenuType::Exit => {
                                    log::slog("Terminating", &logs);
                                    thread::sleep(Duration::from_millis(300));
                                    return Ok(());
                                }
                                menu::builder::MenuType::Silent => {
                                    tx.send(key_value.to_string()).unwrap();
                                }
                                menu::builder::MenuType::Dial => {
                                    app.input_mode = InputMode::Editing;
                                }
                                menu::builder::MenuType::Answer => {
                                    todo!();
                                }
                            }
                        }
                        None => log::slog("Invalid Command", &logs),
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                            let _ = tx
                                .send(format!("d|{}", app.input.trim().to_owned()))
                                .unwrap();
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
            if last_tick.elapsed() >= app.tick_rate {
                last_tick = Instant::now();
            }
        }
    }
}

fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    return true;
}
