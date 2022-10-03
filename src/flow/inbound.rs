use crate::{
    commands::{ok::ok, trying::trying},
    composer::communication::Auth,
    config::JSONConfiguration,
    state::{
        dialogs::{Dialogs, Direction},
        options::SelfConfiguration,
    },
    transmissions::sockets::{send, SocketV4},
    MTLogs,
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
    convert::TryFrom,
    net::UdpSocket,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn process_request_inbound(
    request: &Request,
    socket: &mut UdpSocket,
    conf: &JSONConfiguration,
    settings: &mut SelfConfiguration,
    logs: &MTLogs,
) {
    let via: Via = request.via_header().unwrap().typed().unwrap();

    match request.method {
        rsip::Method::Register => {}
        rsip::Method::Ack => {}
        rsip::Method::Bye => {
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                socket,
                ok(
                    conf,
                    &settings.ip.clone().to_string(),
                    request,
                    rsip::Method::Bye,
                    false,
                )
                .to_string(),
                &settings.verbosity,
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
                socket,
                trying(conf, &settings.ip.clone().to_string(), request).to_string(),
                &settings.verbosity,
                logs,
            );
            thread::sleep(Duration::from_secs(1));
            send(
                &SocketV4 {
                    ip: via.uri.host().to_string(),
                    port: 5060,
                },
                socket,
                ok(
                    conf,
                    &settings.ip.clone().to_string(),
                    request,
                    rsip::Method::Invite,
                    true,
                )
                .to_string(),
                &settings.verbosity,
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
                socket,
                ok(
                    conf,
                    &settings.ip.clone().to_string(),
                    request,
                    rsip::Method::Options,
                    false,
                )
                .to_string(),
                &settings.verbosity,
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
    settings: &mut SelfConfiguration,
    logs: &MTLogs,
) {
    let mut locked_state = state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

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
                    let mut transactions = dg.transactions.get_transactions().unwrap();
                    let mut transaction = transactions.last_mut().unwrap();
                    transaction.object.nonce = Some(auth.nonce);
                    transaction.object.set_auth(conf, "REGISTER");
                    transaction.object.msg = transaction.local.clone();

                    send(
                        &SocketV4 {
                            ip: conf.clone().sip_server,
                            port: conf.clone().sip_port,
                        },
                        socket,
                        transaction.object.push_auth_to_register().to_string(),
                        &settings.verbosity,
                        logs,
                    );
                    break;
                }
            }
        }
        StatusCode::Trying => {}
        StatusCode::OK => {}
        _ => {}
    }
}
