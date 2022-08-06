use std::thread;

pub fn log_out() {
    println!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        ">>>>>>>>>>>>>"
    );
}

pub fn log_in() {
    println!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        "<<<<<<<<<<<<<"
    );
}

pub fn slog(log: &str) {
    println!("<{:?}> [{}] - {:?}", thread::current().id(), line!(), log);
}

pub fn print_menu() {
    println!("s. Toggle Silent mode");
    println!("c. Dial Number");
    println!("x. Exit");
}

pub fn print_msg(msg: String, s: bool) {
    let print = msg.split("\r\n");
    if !s {
        for line in print {
            println!("<{:?}> [{}] - {:?}", thread::current().id(), line!(), line);
        }
    }
}
