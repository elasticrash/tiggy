use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

use crate::{
    config::JSONConfiguration,
    flow::outbound::{outbound_configure, outbound_start},
    processor::message::Message,
    slog::{MTLogs, self},
    startup::registration::unregister_ua,
    state::{
        dialogs::{Dialogs, Direction},
        options::{SelfConfiguration, Verbosity},
    },
};

pub fn send_menu_commands(
    processable_object: &Message,
    dialog_state: &Arc<Mutex<Dialogs>>,
    conf: &JSONConfiguration,
    settings: &mut SelfConfiguration,
    ip: &IpAddr,
    logs: &MTLogs,
) -> bool {
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
                    if is_string_numeric(o.clone()) {
                        settings.flow = Direction::Outbound;
                        outbound_configure(conf, ip, o, dialog_state);
                        outbound_start(conf, dialog_state, &settings.verbosity, logs);
                    }
                }
                None => todo!(),
            };
            false
        }
        'a' => todo!(),
        _ => {
            slog::slog(
                format!(
                    "{:?}: Invalid Command/Not supported",
                    processable_object.bind
                )
                .as_str(),
                logs,
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
