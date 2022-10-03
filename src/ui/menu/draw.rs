use std::{
    io,
    sync::{mpsc::Sender, Arc},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use tui::{backend::Backend, Terminal};

use crate::{
    log::{self, MTLogs},
    ui::app::{ui, App, InputMode},
};

use super::builder::{build_menu, MenuType};

pub fn menu_and_refresh<B: Backend>(
    terminal: &mut Terminal<B>,
    tx: &Sender<String>,
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
                                    log::print_menu();
                                }
                                MenuType::Exit => {
                                    log::slog("Terminating", logs);
                                    tx.send("u".to_string()).unwrap();
                                    thread::sleep(Duration::from_millis(500));
                                    return Ok(());
                                }
                                MenuType::Silent => {
                                    tx.send(key_value.to_string()).unwrap();
                                }
                                MenuType::Dial => {
                                    app.input_mode = InputMode::Editing;
                                }
                                MenuType::Answer => {
                                    todo!();
                                }
                                MenuType::Unregister => {}
                                MenuType::Quiet => tx.send(key_value.to_string()).unwrap(),
                            }
                        }
                        None => log::slog("Invalid Command", logs),
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                            tx.send(format!("d|{}", app.input.trim().to_owned()))
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
