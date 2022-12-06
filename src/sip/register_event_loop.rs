use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Utc;
use cron::Schedule;
use tokio::task::JoinHandle;

use crate::{
    config::JSONConfiguration,
    startup::registration::{keep_alive, register_ua},
    state::dialogs::State,
};

pub fn reg_event_loop(
    c_conf: &JSONConfiguration,
    c_dialog_state: Arc<Mutex<State>>,
    ip: std::net::IpAddr,
) -> JoinHandle<()> {
    let state: Arc<Mutex<State>> = c_dialog_state.clone();
    let conf = c_conf.clone();

    tokio::spawn(async move {
        let init_reg_state = state.clone();
        info!("inital registry");
        {
            register_ua(&init_reg_state, &conf, &ip.clone());
        }
        info!("configuring scheduled registry");
        let expression = "0 */2 * * * *";
        let schedule = Schedule::from_str(expression).unwrap();
        info!("starting scheduled registry");
        loop {
            let upcoming = schedule.upcoming(Utc).take(1).next();
            info!("upcoming registration event {}", upcoming.unwrap());
            'inner: loop {
                let rep_reg_state = state.clone();

                match upcoming {
                    Some(dt) => {
                        if Utc::now() > dt {
                            keep_alive(rep_reg_state, &conf);
                            break 'inner;
                        }
                    }
                    None => info!("no upcoming schedule"),
                }

                std::thread::sleep(Duration::from_millis(500));
            }
        }
    })
}
