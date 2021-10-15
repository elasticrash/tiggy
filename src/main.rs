extern crate md5;
extern crate phf;
extern crate rand;
mod composer;
mod config;
mod models;
use std::{convert::TryFrom, thread, time::Duration};

use config::JSONConfiguration;
use rsip::{header_opt, message::HasHeaders, Header, Request, Response, SipMessage, StatusCode};
use std::net::UdpSocket;

use crate::{
    composer::{
        request::{authorized_register_request, unauthorized_register_request},
        response::ok,
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
    println!("[{}] - {:?}", line!(), ip.to_string());
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

    send(&conf.clone(), register_cseq_1.to_string(), &mut socket);

    let mut count: i32 = 0;

    loop {
        let packet_size = peek(&mut socket, &mut buffer);
        thread::sleep(Duration::from_secs(3));
        if packet_size > 0 {
            let msg = skip_fail!(receive(&mut socket, &mut buffer));
            if msg.is_response() {
                let response = Response::try_from(msg.clone()).unwrap();
                match response.status_code {
                    StatusCode::Unauthorized => {
                        let www_auth =
                            header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                                .unwrap();
                        send(
                            &conf,
                            authorized_register_request(
                                &dialog.history.last().unwrap(),
                                &conf,
                                &rsip::typed::WwwAuthenticate::try_from(www_auth.clone()).unwrap(),
                            )
                            .to_string(),
                            &mut socket,
                        );
                    }
                    StatusCode::Trying => {}
                    StatusCode::OK => {}
                    _ => {}
                }
            } else {
                let request = Request::try_from(msg.clone()).unwrap();

                match request.method {
                    rsip::Method::Register => {}
                    rsip::Method::Ack => {}
                    rsip::Method::Bye => {}
                    rsip::Method::Cancel => {}
                    rsip::Method::Info => {}
                    rsip::Method::Invite => {
                        send(
                            &conf,
                            ok(&conf, &ip.clone().to_string()).to_string(),
                            &mut socket,
                        );
                    }
                    rsip::Method::Message => {}
                    rsip::Method::Notify => {}
                    rsip::Method::Options => {}
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
    }
    return;
}

fn send(conf: &JSONConfiguration, msg: String, socket: &mut UdpSocket) {
    println!("[{}] - {:?}", line!(), ">>>>>>>>>>>>>");
    print_msg(msg.clone());

    socket
        .send_to(
            msg.as_bytes(),
            format!("{}:{}", &conf.sip_server, &conf.sip_port),
        )
        .unwrap();
}

fn receive(socket: &mut UdpSocket, buffer: &mut [u8; 65535]) -> Result<SipMessage, rsip::Error> {
    println!("[{}] - {:?}", line!(), "<<<<<<<<<<<<<");

    let (amt, _src) = socket.recv_from(buffer).unwrap();
    let slice = &mut buffer[..amt];
    let r_message_a = String::from_utf8_lossy(&slice);
    print_msg(r_message_a.to_string());

    SipMessage::try_from(r_message_a.to_string())
}

fn peek(socket: &mut UdpSocket, buffer: &mut [u8]) -> usize {
    println!("[{}] - {:?}", line!(), "--O^O--");
    match socket.peek(buffer) {
        Ok(received) => received,
        Err(_e) => {
            println!("[{}] --stream-is-empty--", line!());
            0
        }
    }
}

fn print_msg(msg: String) {
    let print = msg.split("\r\n");
    for line in print {
        println!("[{}] - {:?}", line!(), line);
    }
}
