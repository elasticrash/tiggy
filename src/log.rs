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
        ">>>>>>>>>>>>>"
    );
}

pub fn print_menu(){
    println!("s. Toggle Silent mode");
    println!("c. Dial Number");
    println!("x. Exit");
}