extern crate md5;
extern crate rand;
/// Composes SIP Messages
mod commands;
/// Extentions on rsip
mod composer;
/// Loads configuration
mod config;
/// Call Flows
mod flow;
/// Logging
mod log;
/// Gets IP information
mod network;
/// Actions that need to happen when the UA starts
mod startup;
/// SIP State
mod state;
/// Upd
mod transmissions;
/// Terminal UI
mod ui;

/// Processor
mod processor;

/// RTP
//mod rtp;

/// SIP
mod sip;

use network::get_ipv4;
use processor::message::{setup_processor, Message};
use state::options::{SelfConfiguration, Verbosity};
use std::collections::VecDeque;
use std::io;
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};
use std::thread::{self};
use transmissions::sockets::SocketV4;
use ui::menu::builder::build_menu;
use ui::menu::draw::{menu_and_refresh, send_menu_commands};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{flog, MTLogs};
use state::dialogs::{Dialogs, Direction};
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;
use ui::app::App;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let logs: MTLogs = Arc::new(Mutex::from(VecDeque::new()));
    let sip_logs: MTLogs = Arc::clone(&logs);
    let rtp_logs: MTLogs = Arc::clone(&logs);
    let just_logs: MTLogs = Arc::clone(&logs);

    let conf = config::read("./config.json").unwrap();

    let ip = match get_ipv4() {
        Ok(ipv4) => ipv4,
        Err(why) => panic!("{}", why),
    };

    flog(&vec![{ &format!("IP found {}", ip) }]);

    let (stx, _srx) = setup_processor::<SocketV4>();
    let (rtx, _rrx) = setup_processor::<SocketV4>();

    logs.lock().unwrap().push_back(format!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        ip
    ));

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

    // let mut rtp_socket = UdpSocket::bind(format!("0.0.0.0:{}", 49152)).unwrap();

    let dialog_state: Arc<Mutex<Dialogs>> = Arc::new(Mutex::new(Dialogs::new(stx, rtx)));
    let (tx, rx) = setup_processor::<Message>();
    let action_menu = Arc::new(build_menu());

    let arc_settings = Arc::new(Mutex::new(SelfConfiguration {
        flow: Direction::Inbound,
        verbosity: Verbosity::Minimal,
        ip: ip,
    }));

    sip::event_loop::sip_event_loop(&conf, &dialog_state, &arc_settings, &sip_logs);

    tokio::spawn(async move {
        'thread: loop {
            // send a command for processing
            if let Ok(processable_object) = rx.try_recv() {
                log::slog(
                    format!("received input, {:?}", processable_object.bind).as_str(),
                    &logs,
                );
                let mut settings = arc_settings.lock().unwrap();

                if send_menu_commands(
                    &processable_object,
                    &dialog_state,
                    &action_menu,
                    &conf,
                    &mut settings,
                    &ip,
                    &logs,
                ) {
                    flog(&vec![{ "got exit ui" }]);

                    let sender = dialog_state.lock().unwrap();
                    sender
                        .sip
                        .lock()
                        .unwrap()
                        .send(SocketV4 {
                            ip: "".to_string(),
                            port: 111,
                            bytes: vec![],
                            exit: true,
                        })
                        .unwrap();
                    break 'thread;
                }
            }
        }
    });

    // create app and run it
    let app: App = App::default();
    let res = menu_and_refresh(&mut terminal, &tx, &just_logs, app);

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
