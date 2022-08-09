use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
};

use tui::{
    style::Style,
    text::{Span, Spans},
};

pub fn log_out(logs: &Arc<Mutex<VecDeque<String>>>) {
    let mut arr = logs.lock().unwrap();
    arr.push_back(format!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        ">>>>>>>>>>>>>"
    ));
}

pub fn log_in(logs: &Arc<Mutex<VecDeque<String>>>) {
    let mut arr = logs.lock().unwrap();
    arr.push_back(format!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        "<<<<<<<<<<<<<"
    ));
}

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
        { Spans::from(Span::styled("d. Dial Number & (enter to sumbit)", Style::default())) },
        { Spans::from(Span::styled("   or (esc to cancel)", Style::default())) },
        { Spans::from(Span::styled("x. Exit", Style::default())) },
    ]
}

pub fn print_msg(msg: String, s: bool, logs: &Arc<Mutex<VecDeque<String>>>) {
    let print: Vec<&str> = msg.split("\r\n").collect();
    let mut arr = logs.lock().unwrap();
    if !s {
        for line in print {
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
}
