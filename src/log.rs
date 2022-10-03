use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::prelude::*,
    path::Path,
    sync::{Arc, Mutex},
};

use tui::{
    style::Style,
    text::{Span, Spans},
};

use crate::state::options::Verbosity;

pub type MTLogs = Arc<Mutex<VecDeque<String>>>;

pub fn slog(log: &str, logs: &MTLogs) {
    let mut arr = logs.lock().unwrap();
    arr.push_back(format!("{:?}", log));
}

pub fn print_menu() -> Vec<Spans<'static>> {
    vec![
        { Spans::from(Span::styled("s. Toggle Silent mode", Style::default())) },
        {
            Spans::from(Span::styled(
                "d. Dial Number & (enter to sumbit)",
                Style::default(),
            ))
        },
        { Spans::from(Span::styled("   or (esc to cancel)", Style::default())) },
        { Spans::from(Span::styled("x. Exit", Style::default())) },
    ]
}

pub fn print_msg(msg: String, vrb: &Verbosity, logs: &MTLogs) {
    let print: Vec<&str> = msg.split("\r\n").collect();
    let mut arr = logs.lock().unwrap();

    match vrb {
        Verbosity::Diagnostic => {
            for line in print.clone() {
                arr.push_back(format!("{:?}", line));
            }
        }
        Verbosity::Minimal => arr.push_back(format!("{:?}", print[0])),
        Verbosity::Normal => {}
        Verbosity::Quiet => {}
    }
    // logs to file
    flog(&print);
}

pub fn flog(print: &Vec<&str>) {
    if !Path::new("log.txt").exists() {
        File::create("log.txt").unwrap();
    }
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("log.txt")
        .unwrap();
    for line in print {
        if let Err(e) = writeln!(file, "{:?}", line) {
            println!("Error writing to file: {}", e);
        }
    }
}
