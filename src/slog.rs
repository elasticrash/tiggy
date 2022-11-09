use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::prelude::*,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::state::options::Verbosity;

pub type MTLogs = Arc<Mutex<VecDeque<String>>>;

/// Pushes log into a vector
pub fn slog(log: &str, logs: &MTLogs) {
    let mut arr = logs.lock().unwrap();
    arr.push_back(format!("{:?}", log));
}

/// Logs a Message on the console UI based on verbosity Level
pub fn print_msg(msg: String, vrb: &Verbosity, logs: &MTLogs) {
    let print: Vec<&str> = msg.split("\r\n").collect();

    match vrb {
        Verbosity::Diagnostic => {
            for line in print.clone() {
                info!("{}", line);
            }
        }
        Verbosity::Minimal => info!("{}", print[0]),
        Verbosity::Quiet => {}
    }
    // logs to file
    flog(&print);
}

/// Logs to a file in detail, easier to see what's going on, the logs on the UI
/// are basically a gimmick, give or take, this should be opt in, though in the
/// future
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
