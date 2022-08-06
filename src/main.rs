extern crate md5;
extern crate phf;
extern crate rand;
mod commands;
mod composer;
mod config;
mod log;
mod menu;
mod models;
mod sockets;

use std::process;
use std::sync::mpsc::{self};
use std::sync::Arc;
use std::thread;
use std::{convert::TryFrom, time::Duration};

use models::SocketV4;
use rsip::typed::WwwAuthenticate;
use rsip::{
    header_opt,
    headers::ToTypedHeader,
    message::{HasHeaders, HeadersExt},
    typed::Via,
    Header, Request, Response, StatusCode,
};
use std::net::UdpSocket;

use crate::commands::invite::Invite;
use crate::composer::communication::{Call, Trying};
use crate::composer::registration::Register;
use crate::sockets::{peek, receive, send};
use crate::{
    composer::messages::{ok, trying},
    models::SIP,
};

use crate::menu::builder::build_menu;

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("[{}] - {}", line!(), e);
                continue;
            }
        }
    };
}

fn main() {
    let conf = config::read("./config.json").unwrap();
    let ip = get_if_addrs::get_if_addrs().unwrap()[0].addr.ip();
    let (tx, rx) = mpsc::channel();

    println!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        ip.to_string()
    );

    let handler = thread::spawn(move || {
        let mut silent = false;
        let mut buffer = [0 as u8; 65535];

        let mut socket = UdpSocket::bind("0.0.0.0:5060").unwrap();
        let _io_result = socket.set_read_timeout(Some(Duration::new(1, 0)));

        socket
            .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
            .expect("connect function failed");

        let mut dialog = SIP {
            history: Vec::new(),
        };

        let reg_conf = conf.clone();
        let inv_conf = conf.clone();

        let mut register: Register = Register {
            branch: "z9hG4bKnashds8".to_string(),
            extension: reg_conf.extension.to_string(),
            ip: ip.to_string(),
            md5: None,
            password: reg_conf.password.to_string(),
            sip_port: reg_conf.sip_port.to_string(),
            sip_server: reg_conf.sip_server.to_string(),
            username: reg_conf.username,
            realm: None,
            nonce: None,
            msg: None,
        };

        let mut invite: Invite = Invite {
            username: inv_conf.username,
            extension: inv_conf.extension.to_string(),
            sip_server: inv_conf.sip_server.to_string(),
            sip_port: inv_conf.sip_port.to_string(),
            ip: ip.to_string(),
            msg: None,
            cld: None,
        };

        let register_cseq_1 = register.ask();
        dialog.history.push(register_cseq_1.clone());

        send(
            &SocketV4 {
                ip: conf.clone().sip_server,
                port: conf.clone().sip_port,
            },
            register_cseq_1.to_string(),
            &mut socket,
            silent,
        );

        let mut count: i32 = 0;

        loop {
            let packet_size = peek(&mut socket, &mut buffer);
            if packet_size > 0 {
                let msg = skip_fail!(receive(&mut socket, &mut buffer, silent));
                if msg.is_response() {
                    let response = Response::try_from(msg.clone()).unwrap();

                    match response.status_code {
                        StatusCode::Unauthorized => {
                            let auth = WwwAuthenticate::try_from(
                                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                                    .unwrap()
                                    .clone(),
                            )
                            .unwrap();

                            register.nonce = Some(auth.nonce);
                            register.realm = Some(auth.realm);
                            register.calculate_md5();
                            register.msg = dialog.history.last().clone();

                            send(
                                &SocketV4 {
                                    ip: conf.clone().sip_server,
                                    port: conf.clone().sip_port,
                                },
                                register.attempt().to_string(),
                                &mut socket,
                                silent,
                            );
                        }
                        StatusCode::Trying => {}
                        StatusCode::OK => {}
                        _ => {}
                    }
                } else {
                    let request = Request::try_from(msg.clone()).unwrap();
                    let via: Via = request.via_header().unwrap().typed().unwrap();

                    match request.clone().method {
                        rsip::Method::Register => {}
                        rsip::Method::Ack => {}
                        rsip::Method::Bye => {
                            send(
                                &SocketV4 {
                                    ip: via.uri.host().to_string(),
                                    port: 5060,
                                },
                                ok(
                                    &conf,
                                    &ip.clone().to_string(),
                                    &request,
                                    rsip::Method::Bye,
                                    false,
                                )
                                .to_string(),
                                &mut socket,
                                silent,
                            );
                        }
                        rsip::Method::Cancel => {}
                        rsip::Method::Info => {}
                        rsip::Method::Invite => {
                            send(
                                &SocketV4 {
                                    ip: via.uri.host().to_string(),
                                    port: 5060,
                                },
                                trying(&conf, &ip.clone().to_string(), &request).to_string(),
                                &mut socket,
                                silent,
                            );
                            thread::sleep(Duration::from_secs(1));
                            send(
                                &SocketV4 {
                                    ip: via.uri.host().to_string(),
                                    port: 5060,
                                },
                                ok(
                                    &conf,
                                    &ip.clone().to_string(),
                                    &request,
                                    rsip::Method::Invite,
                                    true,
                                )
                                .to_string(),
                                &mut socket,
                                silent,
                            );
                        }
                        rsip::Method::Message => {}
                        rsip::Method::Notify => {}
                        rsip::Method::Options => {
                            send(
                                &SocketV4 {
                                    ip: via.uri.host().to_string(),
                                    port: 5060,
                                },
                                ok(
                                    &conf,
                                    &ip.clone().to_string(),
                                    &request,
                                    rsip::Method::Options,
                                    false,
                                )
                                .to_string(),
                                &mut socket,
                                silent,
                            );
                        }
                        rsip::Method::PRack => {}
                        rsip::Method::Publish => {}
                        rsip::Method::Refer => {}
                        rsip::Method::Subscribe => {}
                        rsip::Method::Update => {}
                    }
                }
            }
            count += 1;
            if count == 1800 {
                break;
            }

            let action_menu = Arc::new(build_menu());

            match rx.try_recv() {
                Ok(code) => {
                    let mut command = String::from(code);
                    let mut argument: String = "".to_string();
                    log::slog(format!("received command, {}", command.to_string()).as_str());

                    if command.len() > 1 {
                        let split_command = command.split("|").collect::<Vec<&str>>();
                        argument = split_command[1].to_string();
                        command = split_command[0].to_string();
                    }

                    if !is_string_numeric(argument.clone()) {
                        command = "invalid_argument".to_string();
                    }

                    match action_menu.iter().find(|&x| x.value == command) {
                        Some(item) => match item.category {
                            menu::builder::MenuType::Exit => {
                                break;
                            }
                            menu::builder::MenuType::Silent => {
                                silent = !silent;
                            }
                            menu::builder::MenuType::Dial => {
                                invite.cld = Some(argument);

                                send(
                                    &SocketV4 {
                                        ip: conf.clone().sip_server,
                                        port: conf.clone().sip_port,
                                    },
                                    invite.ask().to_string(),
                                    &mut socket,
                                    silent,
                                );
                            }
                            menu::builder::MenuType::Answer => todo!(),
                            _ => log::slog(format!("{} Not supported", command).as_str()),
                        },
                        None => todo!(),
                    }
                }
                Err(_) => {}
            }
        }
    });

    let cmd_menu = Arc::new(build_menu());

    loop {
        let mut buffer = String::new();
        match std::io::stdin().read_line(&mut buffer) {
            Err(why) => panic!("couldn't read {:?}", why.raw_os_error()),
            _ => (),
        };

        match cmd_menu.iter().find(|&x| x.value == buffer.trim()) {
            Some(item) => match item.category {
                menu::builder::MenuType::DisplayMenu => {
                    log::print_menu();
                }
                menu::builder::MenuType::Exit => {
                    log::slog("Terminating");
                    tx.send(item.value.to_string()).unwrap();
                    handler.join().unwrap();
                    process::exit(0);
                }
                menu::builder::MenuType::Silent => {
                    tx.send(item.value.to_string()).unwrap();
                }
                menu::builder::MenuType::Dial => {
                    log::slog("Enter Phone Number");
                    let mut phone_buffer = String::new();
                    match std::io::stdin().read_line(&mut phone_buffer) {
                        Err(why) => panic!("couldn't read {:?}", why.raw_os_error()),
                        _ => (),
                    };

                    let _ = tx
                        .send(format!("d|{}", phone_buffer.trim().to_owned()))
                        .unwrap();
                }
                menu::builder::MenuType::Answer => {
                    tx.send(item.value.to_string()).unwrap();
                }
            },
            None => log::slog("Invalid Command"),
        }
    }
}

fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    return true;
}
