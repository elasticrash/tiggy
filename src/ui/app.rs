use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::log::print_menu;

pub struct App {
    pub input: String,
    pub input_mode: InputMode,
    pub tick_rate: Duration,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            tick_rate: Duration::from_millis(200),
        }
    }
}

pub enum InputMode {
    Normal,
    Editing,
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &App, logs: &Arc<Mutex<VecDeque<String>>>) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(main[1]);

    let command_block = Block::default().title("Commands").borders(Borders::ALL);
    let raw_block = Block::default().title("Raw Logs").borders(Borders::ALL);
    let input_block = Block::default().borders(Borders::ALL).title("Enter Number to Dial");

    let mut text = Text::from(Spans::from("press d"));
    text.patch_style(Style::default());
    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(input_block);
    f.render_widget(input, main[0]);

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
    let offset = lgs.len() as i32 - (f.size().height) as i32;
    let offset_ = if offset < 0 { 0 } else { offset };

    for x in 3..chunks[0].height {
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
