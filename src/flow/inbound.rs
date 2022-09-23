use crate::{
    commands::{ok::ok, trying::trying},
    composer::communication::Auth,
    config::JSONConfiguration,
    log,
    state::dialogs::{Dialogs, Direction},
    transmissions::sockets::{send, SocketV4},
};
use rsip::{
    header_opt,
    headers::ToTypedHeader,
    message::HasHeaders,
    message::HeadersExt,
    typed::{Via, WwwAuthenticate},
    Header, Request, Response, StatusCode,
};
use std::{
    collections::VecDeque,
    convert::TryFrom,
    net::{IpAddr, UdpSocket},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn process_request_inbound(
    request: &Request,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    ip: &IpAddr,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let via: Via = request.via_header().unwrap().typed().unwrap();

    log::slog(
        format!("received inbound request, {}", request.method).as_str(),
        logs,
    );

    match request.method {
        rsip::Method::Register => {}
        rsip::Method::Ack => {}
        rsip::Method::Bye => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    conf,
                    &ip.clone().to_string(),
                    request,
                    rsip::Method::Bye,
                    false,
                )
                .to_string(),
                socket,
                silent,
                logs,
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
                trying(conf, &ip.clone().to_string(), request).to_string(),
                socket,
                silent,
                logs,
            );
            thread::sleep(Duration::from_secs(1));
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                ok(
                    conf,
                    &ip.clone().to_string(),
                    request,
                    rsip::Method::Invite,
                    true,
                )
                .to_string(),
                socket,
                silent,
                logs,
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
                    conf,
                    &ip.clone().to_string(),
                    request,
                    rsip::Method::Options,
                    false,
                )
                .to_string(),
                socket,
                silent,
                logs,
            );
        }
        rsip::Method::PRack => {}
        rsip::Method::Publish => {}
        rsip::Method::Refer => {}
        rsip::Method::Subscribe => {}
        rsip::Method::Update => {}
    }
}

pub fn process_response_inbound(
    response: &Response,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<Dialogs>>,
    silent: bool,
    logs: &Arc<Mutex<VecDeque<String>>>,
) {
    let mut locked_state = state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    log::slog(
        format!("received inbound response, {}", response.status_code).as_str(),
        logs,
    );

    match response.status_code {
        StatusCode::Unauthorized => {
            let auth = WwwAuthenticate::try_from(
                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            for dg in dialogs.iter_mut() {
                if matches!(dg.diag_type, Direction::Inbound) {
                    let mut tr = dg.transactions.get_transactions().unwrap();
                    let mut transaction = tr.last_mut().unwrap();
                    transaction.object.nonce = Some(auth.nonce.clone());
                    transaction.object.set_auth(conf, "REGISTER");
                    transaction.object.msg = transaction.local.clone();

                    send(
                        &SocketV4 {
                            ip: conf.clone().sip_server,
                            port: conf.clone().sip_port,
                        },
                        transaction.object.push_auth_to_register().to_string(),
                        socket,
                        silent,
                        logs,
                    );
                }
            }
        }
        StatusCode::Trying => {}
        StatusCode::OK => {}
        _ => {}
    }
}
