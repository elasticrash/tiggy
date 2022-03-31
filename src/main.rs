extern crate md5;
extern crate phf;
extern crate rand;
mod composer;
mod config;
mod models;
use std::process;
use std::sync::mpsc::{self};
use std::thread;
use std::{convert::TryFrom, time::Duration};

use models::SocketV4;
use rsip::{
    header_opt,
    headers::ToTypedHeader,
    message::{HasHeaders, HeadersExt},
    typed::Via,
    Header, Request, Response, SipMessage, StatusCode,
};
use std::net::UdpSocket;

use crate::{
    composer::{
        request::{authorized_register_request, unauthorized_register_request},
        response::{ok, simple_ok, trying},
    },
    models::SIP,
};

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

        let register_cseq_1 = unauthorized_register_request(&conf.clone(), &ip.clone().to_string());
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
                            let www_auth =
                                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                                    .unwrap();
                            send(
                                &SocketV4 {
                                    ip: conf.clone().sip_server,
                                    port: conf.clone().sip_port,
                                },
                                authorized_register_request(
                                    &dialog.history.last().unwrap(),
                                    &conf,
                                    &rsip::typed::WwwAuthenticate::try_from(www_auth.clone())
                                        .unwrap(),
                                )
                                .to_string(),
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
                                simple_ok(
                                    &conf,
                                    &ip.clone().to_string(),
                                    &request,
                                    rsip::Method::Bye,
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
                            thread::sleep(Duration::from_secs(3));
                            send(
                                &SocketV4 {
                                    ip: via.uri.host().to_string(),
                                    port: 5060,
                                },
                                ok(&conf, &ip.clone().to_string(), &request).to_string(),
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
                                simple_ok(
                                    &conf,
                                    &ip.clone().to_string(),
                                    &request,
                                    rsip::Method::Options,
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

            match rx.try_recv() {
                Ok(code) => {
                    println!(
                        "<{:?}> [{}] - Received {} command",
                        thread::current().id(),
                        line!(),
                        code
                    );
                    if code == "x" {
                        break;
                    }
                    if code == "s" {
                        println!(
                            "<{:?}> [{}] - Executing {} command",
                            thread::current().id(),
                            line!(),
                            code
                        );
                        silent = !silent;
                    }
                }
                Err(_) => {}
            }
        }
    });

    loop {
        let mut buffer = String::new();
        match std::io::stdin().read_line(&mut buffer) {
            Err(why) => panic!("couldn't read {:?}", why.raw_os_error()),
            _ => (),
        };
        
        if buffer.trim() == "x" {
            println!(
                "<{:?}> [{}] - {:?}",
                thread::current().id(),
                line!(),
                "Terminating."
            );
            let _ = tx.send("x").unwrap();
            handler.join().unwrap();
            process::exit(0);
        }
        if buffer.trim() == "s" {
            let _ = tx.send("s").unwrap();
        }
    }
}

fn send(s_conf: &SocketV4, msg: String, socket: &mut UdpSocket, s: bool) {
    println!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        ">>>>>>>>>>>>>"
    );
    print_msg(msg.clone(), s);

    socket
        .send_to(msg.as_bytes(), format!("{}:{}", s_conf.ip, s_conf.port))
        .unwrap();
}

fn receive(
    socket: &mut UdpSocket,
    buffer: &mut [u8; 65535],
    s: bool,
) -> Result<SipMessage, rsip::Error> {
    println!(
        "<{:?}> [{}] - {:?}",
        thread::current().id(),
        line!(),
        "<<<<<<<<<<<<<"
    );

    let (amt, _src) = socket.recv_from(buffer).unwrap();
    let slice = &mut buffer[..amt];
    let r_message_a = String::from_utf8_lossy(&slice);
    print_msg(r_message_a.to_string(), s);

    SipMessage::try_from(r_message_a.to_string())
}

fn peek(socket: &mut UdpSocket, buffer: &mut [u8]) -> usize {
    match socket.peek(buffer) {
        Ok(received) => received,
        Err(_e) => 0,
    }
}

fn print_msg(msg: String, s: bool) {
    let print = msg.split("\r\n");
    if !s {
        for line in print {
            println!("<{:?}> [{}] - {:?}", thread::current().id(), line!(), line);
        }
    }
}
