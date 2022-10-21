use std::{
    io,
    net::IpAddr,
    sync::{mpsc::Sender, Arc, Mutex},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use tui::{backend::Backend, Terminal};

use crate::{
    config::JSONConfiguration,
    flow::outbound::{outbound_configure, outbound_start},
    log::{self, MTLogs},
    processor::message::{Message, MessageType},
    startup::registration::unregister_ua,
    state::{
        dialogs::{Dialogs, Direction},
        options::{SelfConfiguration, Verbosity},
    },
    ui::app::{ui, App, InputMode},
};

use super::builder::{build_menu, print_menu, MenuItem, MenuType};

pub fn menu_and_refresh<B: Backend>(
    terminal: &mut Terminal<B>,
    tx: &Sender<Message>,
    logs: &MTLogs,
    mut app: App,
) -> io::Result<()> {
    let cmd_menu = Arc::new(build_menu());
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app, logs))?;
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
                                MenuType::DisplayMenu => {
                                    print_menu();
                                }
                                MenuType::Exit => {
                                    tx.send(Message::new(
                                        MessageType::MenuCommand,
                                        key_value,
                                        None,
                                    ))
                                    .unwrap();
                                    return Ok(());
                                }
                                MenuType::Silent | MenuType::Quiet => {
                                    tx.send(Message::new(
                                        MessageType::MenuCommand,
                                        key_value,
                                        None,
                                    ))
                                    .unwrap();
                                }
                                MenuType::Dial => {
                                    app.input_mode = InputMode::Editing;
                                }
                                MenuType::Answer => {
                                    todo!();
                                }
                                MenuType::Unregister => {}
                            }
                        }
                        None => log::slog("Invalid Command", logs),
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                            tx.send(Message::new(
                                MessageType::MenuCommand,
                                'd',
                                Some(app.input.trim().to_owned()),
                            ))
                            .unwrap()
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

pub fn send_menu_commands(
    processable_object: &Message,
    dialog_state: &Arc<Mutex<Dialogs>>,
    action_menu: &Arc<Vec<MenuItem>>,
    conf: &JSONConfiguration,
    settings: &mut SelfConfiguration,
    ip: &IpAddr,
    logs: &MTLogs,
) -> bool {
    let key_code_command = KeyCode::Char(processable_object.bind);

    match action_menu.iter().find(|&x| x.value == key_code_command) {
        Some(item) => match item.category {
            super::builder::MenuType::Unregister => false,
            super::builder::MenuType::Exit => {
                unregister_ua(dialog_state, conf);
                true
            }
            super::builder::MenuType::Silent => {
                settings.verbosity = if matches!(settings.verbosity, Verbosity::Diagnostic) {
                    Verbosity::Minimal
                } else {
                    Verbosity::Diagnostic
                };
                false
            }
            super::builder::MenuType::Quiet => {
                settings.verbosity = Verbosity::Quiet;
                false
            }
            super::builder::MenuType::Dial => {
                match &processable_object.content {
                    Some(o) => {
                        if is_string_numeric(o.clone()) {
                            settings.flow = Direction::Outbound;
                            outbound_configure(conf, ip, o, dialog_state);
                            outbound_start(conf, dialog_state, &settings.verbosity, logs);
                        }
                    }
                    None => todo!(),
                };
                false
            }
            super::builder::MenuType::Answer => todo!(),
            _ => {
                log::slog(
                    format!(
                        "{:?}: Invalid Command/Not supported",
                        processable_object.bind
                    )
                    .as_str(),
                    logs,
                );
                false
            }
        },
        None => todo!(),
    }
}

fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    true
}
