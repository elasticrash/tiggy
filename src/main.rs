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
use log::print_menu;
use rsip::Response;
use std::net::UdpSocket;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Corner, Direction, Layout};
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::{Frame, Terminal};

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
            let mut buffer = [0 as u8; 65535];

            let mut socket = UdpSocket::bind("0.0.0.0:5060").unwrap();
            let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));

            socket
                .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                .expect("connect function failed");

            let mut count: i32 = 0;
            let shared = inbound_start(&conf, &ip);

            'thread: loop {
                if count == 0 {
                    send(
                        &SocketV4 {
                            ip: conf.clone().sip_server,
                            port: conf.clone().sip_port,
                        },
                        shared.borrow().reg.ask().to_string(),
                        &mut socket,
                        silent,
                        &thread_logs,
                    );
                }

                let packet_size = peek(&mut socket, &mut buffer);
                if packet_size > 0 {
                    let msg = skip_fail!(receive(&mut socket, &mut buffer, silent, &thread_logs));
                    if msg.is_response() {
                        let response = Response::try_from(msg.clone()).unwrap();

                        inbound_response_flow(
                            &response,
                            &mut socket,
                            &conf,
                            &shared,
                            silent,
                            &thread_logs,
                        );
                    } else {
                        inbound_request_flow(&msg, &mut socket, &conf, &ip, silent, &thread_logs);
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
                                    // invite.cld = Some(argument);

                                    // send(
                                    //     &SocketV4 {
                                    //         ip: conf.clone().sip_server,
                                    //         port: conf.clone().sip_port,
                                    //     },
                                    //     invite.ask().to_string(),
                                    //     &mut socket,
                                    //     silent,
                                    // );
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
    let res = run_app(&mut terminal, &tx, &logs, Duration::from_millis(250));

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
    tick_rate: Duration,
) -> io::Result<()> {
    let cmd_menu = Arc::new(build_menu());
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &logs))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match cmd_menu.iter().find(|&x| x.value == key.code) {
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
                                thread::sleep(Duration::from_secs(1));
                                return Ok(());
                            }
                            menu::builder::MenuType::Silent => {
                                tx.send(key_value.to_string()).unwrap();
                            }
                            menu::builder::MenuType::Dial => {
                                todo!();
                                log::slog("Enter Phone Number", &logs);
                                let mut phone_buffer = String::new();
                                match std::io::stdin().read_line(&mut phone_buffer) {
                                    Err(why) => panic!("couldn't read {:?}", why.raw_os_error()),
                                    _ => (),
                                };

                                let _ = tx
                                    .send(format!("d|{}", phone_buffer.trim().to_owned()))
                                    .unwrap();
                            }
                            menu::builder::MenuType::Answer => {
                                todo!();
                            }
                        }
                    }
                    None => log::slog("Invalid Command", &logs),
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, logs: &Arc<Mutex<VecDeque<String>>>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());

    let command_block = Block::default().title("Commands").borders(Borders::ALL);
    let raw_block = Block::default().title("Raw Logs").borders(Borders::ALL);

    let lines = print_menu();

    let command_list = List::new(vec![ListItem::new(lines)])
        .block(command_block)
        .start_corner(Corner::TopRight);
    f.render_widget(command_list, chunks[0]);

    let mut lg: Vec<Spans> = vec![];
    let mut lgs = logs.lock().unwrap();
    if lgs.len() > 500 {
        lgs.drain(0..300);
    }
    let offset = lgs.len() as i32 - (f.size().height - 3) as i32;
    let offset_ = if offset < 0 { 0 } else { offset };

    for x in 0..f.size().height - 3 {
        if x < lgs.len() as u16 {
            lg.push(Spans::from(Span::styled(
                &*lgs[(offset_ + x as i32) as usize],
                Style::default(),
            )));
        }
    }

    let events_list = List::new(vec![ListItem::new(lg)])
        .block(raw_block)
        .start_corner(Corner::TopRight);
    f.render_widget(events_list, chunks[1]);
}

fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    return true;
}
