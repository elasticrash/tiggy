use crate::{
    commands::{auth::Auth, auth::AuthModel, ok::ok, trying::trying},
    config::JSONConfiguration,
    state::{
        dialogs::{Direction, State},
        options::SelfConfiguration,
    },
    transmissions::sockets::{MpscBase, SocketV4},
};
use rsip::{
    header_opt,
    headers::ToTypedHeader,
    message::HasHeaders,
    message::HeadersExt,
    typed::{ProxyAuthenticate, Via, WwwAuthenticate},
    Header, Request, Response, StatusCode,
};
use std::{
    convert::TryFrom,
    net::IpAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn process_request_inbound(
    request: &Request,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<State>>,
    settings: &mut SelfConfiguration,
) {
    let mut locked_state = state.lock().unwrap();
    let channel = locked_state.get_sip_channel().unwrap();

    let via: Via = request.via_header().unwrap().typed().unwrap();

    match request.method {
        rsip::Method::Register => {}
        rsip::Method::Ack => {}
        rsip::Method::Bye => {
            channel
                .0
                .send(MpscBase {
                    event: Some(SocketV4 {
                        ip: via.uri.host().to_string(),
                        port: 5060,
                        bytes: ok(
                            conf,
                            &settings.ip.clone().to_string(),
                            request,
                            rsip::Method::Bye,
                            false,
                        )
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                    }),
                    exit: false,
                })
                .unwrap();
        }
        rsip::Method::Cancel => {}
        rsip::Method::Info => {}
        rsip::Method::Invite => {
            // let connection: Option<IpAddr>;

            channel
                .0
                .send(MpscBase {
                    event: Some(SocketV4 {
                        ip: via.uri.host().to_string(),
                        port: 5060,
                        bytes: trying(conf, &settings.ip.clone().to_string(), request)
                            .to_string()
                            .as_bytes()
                            .to_vec(),
                    }),
                    exit: false,
                })
                .unwrap();
            thread::sleep(Duration::from_secs(1));
            channel
                .0
                .send(MpscBase {
                    event: Some(SocketV4 {
                        ip: via.uri.host().to_string(),
                        port: 5060,
                        bytes: ok(
                            conf,
                            &settings.ip.clone().to_string(),
                            request,
                            rsip::Method::Invite,
                            true,
                        )
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                    }),
                    exit: false,
                })
                .unwrap();

            info!("{}", String::from_utf8_lossy(&request.body).to_string());
            let sdp = sdp_rs::SessionDescription::try_from(
                String::from_utf8_lossy(&request.body).to_string(),
            );

            let connection: Option<IpAddr> = Some(
                sdp.clone()
                    .unwrap()
                    .connection
                    .unwrap()
                    .connection_address
                    .base,
            );
            let rtp_port: Option<u16> =
                Some(sdp.unwrap().media_descriptions.first().unwrap().media.port);

            match connection.is_some() && rtp_port.is_some() {
                true => {
                    // START NEW THREAD ON THE ABOVE TO RECEIVE PACKETS
                    // rtp::event_loop::rtp_event_loop(
                    //     &settings.ip,
                    //     49152,
                    //     state.clone(),
                    //     &connection.unwrap(),
                    //     rtp_port.unwrap(),
                    // );
                }
                false => {}
            }
        }
        rsip::Method::Message => {}
        rsip::Method::Notify => {
            channel
                .0
                .send(MpscBase {
                    event: Some(SocketV4 {
                        ip: via.uri.host().to_string(),
                        port: 5060,
                        bytes: ok(
                            conf,
                            &settings.ip.clone().to_string(),
                            request,
                            rsip::Method::Notify,
                            false,
                        )
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                    }),
                    exit: false,
                })
                .unwrap();
        }
        rsip::Method::Options => {
            channel
                .0
                .send(MpscBase {
                    event: Some(SocketV4 {
                        ip: via.uri.host().to_string(),
                        port: 5060,
                        bytes: ok(
                            conf,
                            &settings.ip.clone().to_string(),
                            request,
                            rsip::Method::Options,
                            false,
                        )
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                    }),
                    exit: false,
                })
                .unwrap();
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
    state: &Arc<Mutex<State>>,
) {
    match response.status_code {
        StatusCode::Unauthorized | StatusCode::ProxyAuthenticationRequired => {
            // TODO: this part needs to be a bit more generic
            // Now its too specific for registrations
            let www_auth = header_opt!(response.headers().iter(), Header::WwwAuthenticate);
            let proxy_auth = header_opt!(response.headers().iter(), Header::ProxyAuthenticate);

            if www_auth.is_some() || proxy_auth.is_some() {
                let auth_model: AuthModel = if let Some(..) = www_auth {
                    AuthModel {
                        nonce: WwwAuthenticate::try_from(www_auth.unwrap().clone())
                            .unwrap()
                            .nonce,
                        realm: WwwAuthenticate::try_from(www_auth.unwrap().clone())
                            .unwrap()
                            .realm,
                        qop: WwwAuthenticate::try_from(www_auth.unwrap().clone())
                            .unwrap()
                            .qop,
                        opaque: WwwAuthenticate::try_from(www_auth.unwrap().clone())
                            .unwrap()
                            .opaque,
                    }
                } else {
                    let www_auth_from_proxy =
                        ProxyAuthenticate::try_from(proxy_auth.unwrap().clone()).unwrap();

                    AuthModel {
                        nonce: (www_auth_from_proxy).0.nonce,
                        realm: (www_auth_from_proxy).0.realm,
                        qop: (www_auth_from_proxy).0.qop,
                        opaque: (www_auth_from_proxy).0.opaque,
                    }
                };

                let mut transaction: Option<String> = None;
                {
                    let state: Arc<Mutex<State>> = state.clone();
                    let mut locked_state = state.lock().unwrap();
                    let mut registrations = locked_state.get_registrations().unwrap();
                    for dg in registrations.iter_mut() {
                        if matches!(dg.diag_type, Direction::Inbound) {
                            let mut transactions = dg.transactions.get_transactions().unwrap();
                            let mut local_transaction = transactions.first_mut().unwrap();
                            local_transaction.object.nonce = Some(auth_model.nonce.clone());
                            local_transaction.object.set_auth(
                                conf,
                                "REGISTER",
                                &auth_model.clone(),
                            );
                            local_transaction.object.msg = local_transaction.local.clone();

                            transaction = Some(
                                local_transaction
                                    .object
                                    .push_auth_to_register(response.status_code.clone())
                                    .to_string(),
                            );
                            break;
                        }
                    }
                }

                if let Some(..) = transaction {
                    let locked_socket = state.clone();
                    let mut unlocked_socket = locked_socket.lock().unwrap();
                    let channel = unlocked_socket.get_sip_channel().unwrap();

                    channel
                        .0
                        .send(MpscBase {
                            event: Some(SocketV4 {
                                ip: conf.clone().sip_server,
                                port: conf.clone().sip_port,
                                bytes: transaction.unwrap().as_bytes().to_vec(),
                            }),
                            exit: false,
                        })
                        .unwrap();
                }
            }
        }
        StatusCode::Trying => {}
        StatusCode::OK => {}
        _ => {}
    }
}
