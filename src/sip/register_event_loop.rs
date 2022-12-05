use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Utc;
use cron::Schedule;
use tokio::task::JoinHandle;

use crate::{
    config::JSONConfiguration, startup::registration::register_ua, state::dialogs::Dialogs,
};

pub fn reg_event_loop(
    c_conf: &JSONConfiguration,
    c_dialog_state: &Arc<Mutex<Dialogs>>,
    ip: std::net::IpAddr,
) -> JoinHandle<()> {
    let state: Arc<Mutex<Dialogs>> = c_dialog_state.clone();
    let conf = c_conf.clone();

    tokio::spawn(async move {
        let dialog_state = state;
        info!("inital registry");
        {
            register_ua(&dialog_state, &conf, &ip.clone());
        }
        info!("configuring scheduled registry");
        let expression = "*/2.5 * * * *";
        let schedule = Schedule::from_str(expression).unwrap();
        info!("starting scheduled registry");
        loop {
            for datetime in schedule.upcoming(Utc).take(1) {
                if datetime > Utc::now() {
                    register_ua(&dialog_state, &conf, &ip.clone());
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}
