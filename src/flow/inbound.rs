use crate::{
    commands::{auth::Auth, ok::ok, trying::trying},
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
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn process_request_inbound(
    request: &Request,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<Dialogs>>,
    settings: &mut SelfConfiguration,
    logs: &MTLogs,
) {
    let mut locked_state = state.lock().unwrap();
    let mut socket = locked_state.get_socket().unwrap();

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
                &mut socket,
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
                &mut socket,
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
                &mut socket,
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
                &mut socket,
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
    conf: &JSONConfiguration,
    state: &Arc<Mutex<Dialogs>>,
    settings: &mut SelfConfiguration,
    logs: &MTLogs,
) {
    match response.status_code {
        StatusCode::Unauthorized => {
            let auth = WwwAuthenticate::try_from(
                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            let mut transaction: Option<String> = None;
            {
                let state: Arc<Mutex<Dialogs>> = state.clone();
                let mut locked_state = state.lock().unwrap();
                let mut dialogs = locked_state.get_dialogs().unwrap();

                for dg in dialogs.iter_mut() {
                    if matches!(dg.diag_type, Direction::Inbound) {
                        let mut transactions = dg.transactions.get_transactions().unwrap();
                        let mut local_transaction = transactions.last_mut().unwrap();
                        local_transaction.object.nonce = Some(auth.nonce);
                        local_transaction.object.set_auth(conf, "REGISTER");
                        local_transaction.object.msg = local_transaction.local.clone();

                        transaction =
                            Some(local_transaction.object.push_auth_to_register().to_string());
                        break;
                    }
                }
            }

            if let Some(..) = transaction {
                let locked_socket = state.clone();
                let mut unlocked_socket = locked_socket.lock().unwrap();
                let mut socket = unlocked_socket.get_socket().unwrap();
                send(
                    &SocketV4 {
                        ip: conf.clone().sip_server,
                        port: conf.clone().sip_port,
                    },
                    &mut socket,
                    transaction.unwrap(),
                    &settings.verbosity,
                    logs,
                );
            }
        }
        StatusCode::Trying => {}
        StatusCode::OK => {}
        _ => {}
    }
}
