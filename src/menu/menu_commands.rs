use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

use log::info;

use crate::{
    config::JSONConfiguration,
    flow::outbound::{outbound_configure, outbound_start},
    processor::message::Message,
    startup::registration::unregister_ua,
    state::{
        dialogs::{State, Direction},
        options::{SelfConfiguration, Verbosity},
    },
};

pub fn send_menu_commands(
    processable_object: &Message,
    dialog_state: Arc<Mutex<State>>,
    conf: &JSONConfiguration,
    settings: &mut SelfConfiguration,
    ip: &IpAddr,
) -> bool {
    info!("received command {}", processable_object.bind);
    info!("begging matching");
    match processable_object.bind {
        'u' => false,
        'x' => {
            unregister_ua(dialog_state, conf);
            true
        }
        's' => {
            settings.verbosity = if matches!(settings.verbosity, Verbosity::Diagnostic) {
                Verbosity::Minimal
            } else {
                Verbosity::Diagnostic
            };
            false
        }
        'q' => {
            settings.verbosity = Verbosity::Quiet;
            false
        }
        'd' => {
            match &processable_object.content {
                Some(o) => {
                    info!("checking dial command");

                    if is_string_numeric(o.clone()) {
                        settings.flow = Direction::Outbound;
                        outbound_configure(conf, ip, o, dialog_state.clone());
                        outbound_start(conf, dialog_state, &settings.verbosity);
                    }
                }
                None => todo!(),
            };
            false
        }
        'a' => todo!(),
        _ => {
            info!(
                "{:?}: Invalid Command/Not supported",
                processable_object.bind
            );
            false
        }
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
