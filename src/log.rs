use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::prelude::*,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

use tui::{
    style::Style,
    text::{Span, Spans},
};

pub fn slog(log: &str, logs: &Arc<Mutex<VecDeque<String>>>) {
    let mut arr = logs.lock().unwrap();
    arr.push_back(format!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        log
    ));
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

pub fn print_msg(msg: String, s: bool, logs: &Arc<Mutex<VecDeque<String>>>) {
    let print: Vec<&str> = msg.split("\r\n").collect();
    let mut arr = logs.lock().unwrap();
    if !s {
        for line in print.clone() {
            arr.push_back(format!(
                "<{:?}> [{}] - {:?}",
                thread::current().id(),
                line!(),
                line
            ));
        }
    } else {
        arr.push_back(format!(
            "<{:?}> [{}] - {:?}",
            thread::current().id(),
            line!(),
            print[0]
        ));
    }

    // logs to file
    if !Path::new("log.txt").exists() {
        File::create("log.txt").unwrap();
    }
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("log.txt")
        .unwrap();
    for line in print {
        if let Err(e) = writeln!(
            file,
            "<{:?}> [{}] - {:?}",
            thread::current().id(),
            line!(),
            line
        ) {
            println!("Error writing to file: {}", e);
        }
    }
}
