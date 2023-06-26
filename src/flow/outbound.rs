use crate::{
    commands::{
        auth::Auth,
        auth::AuthModel,
        helper::{get_address_from_contact, get_address_from_record_route, get_remote_tag},
        ok::ok,
    },
    composer::header_extension::CustomHeaderExtension,
    config::JSONConfiguration,
    slog::udp_logger,
    state::{
        dialogs::{Dialog, Direction, State, Transactions},
        options::{SelfConfiguration, SipOptions, Verbosity},
        transactions::{Transaction, TransactionType},
    },
    transmissions::sockets::{MpscBase, SocketV4},
};

use chrono::prelude::*;
use rsip::{
    header_opt,
    message::HasHeaders,
    prelude::{HeadersExt, ToTypedHeader},
    typed::{Via, WwwAuthenticate},
    Header, Method, Request, Response, SipMessage, StatusCode,
};
use std::{
    convert::TryFrom,
    net::IpAddr,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

pub fn outbound_configure(
    conf: &JSONConfiguration,
    ip: &IpAddr,
    destination: &str,
    dialog_state: Arc<Mutex<State>>,
) {
    let mut locked_state = dialog_state.lock().unwrap();
    let mut dialogs = locked_state.get_dialogs().unwrap();

    let call_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let invite = SipOptions {
        branch: format!(
            "z9hG4bK{}{}{}{}{}{}",
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_millis()
        ),
        extension: conf.extension.to_string(),
        username: conf.username.clone(),
        sip_server: conf.sip_server.to_string(),
        sip_port: conf.sip_port.to_string(),
        ip: ip.to_string(),
        msg: None,
        cld: Some(destination.to_string()),
        md5: None,
        nonce: None,
        call_id: call_id.clone(),
        tag_local: Uuid::new_v4().to_string(),
        tag_remote: None,
        nc: None,
        cnonce: None,
        qop: false,
    };

    let dialog = Dialog {
        call_id: call_id.clone(),
        diag_type: Direction::Outbound,
        local_tag: Uuid::new_v4().to_string(),
        remote_tag: None,
        transactions: Transactions::new(),
        time: Local::now(),
    };

    dialogs.push(dialog);

    for dg in dialogs.iter_mut() {
        if dg.call_id == call_id {
            let mut transactions = dg.transactions.get_transactions().unwrap();
            let mut transaction = Transaction {
                object: invite.clone(),
                local: Some(invite.set_initial_invite()),
                remote: None,
                tr_type: TransactionType::Invite,
            };
            transaction.object.msg = Some(transaction.object.clone().set_initial_invite());
            transactions.push(transaction);
        }
    }
}

/// Sends the Intial invite for an outbound call
// TODO pass identifier for the call
pub fn outbound_start(conf: &JSONConfiguration, state: Arc<Mutex<State>>, vrb: &Verbosity) {
    let mut transaction: Option<String> = None;
    {
        let state: Arc<Mutex<State>> = state.clone();
        let mut locked_state = state.lock().unwrap();
        let mut dialogs = locked_state.get_dialogs().unwrap();
        info!("number of dialogs {}: ", dialogs.len());

        for dg in dialogs.iter_mut().rev() {
            if matches!(dg.diag_type, Direction::Outbound) {
                let mut transactions = dg.transactions.get_transactions().unwrap();

                udp_logger(
                    format!("number of transactions {}: ", transactions.len()),
                    vrb,
                );

                let mut loop_transaction = transactions.last_mut().unwrap();
                loop_transaction.local = loop_transaction.object.set_initial_invite().into();

                transaction = Some(loop_transaction.local.clone().unwrap().to_string());
                break;
            }
        }
    }
    if let Some(..) = transaction {
        let t_state = state;
        let mut locked_state = t_state.lock().unwrap();
        let channel = locked_state.get_sip_channel().unwrap();

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

pub fn process_request_outbound(
    request: &Request,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<State>>,
    settings: &mut SelfConfiguration,
) {
    let mut locked_state = state.lock().unwrap();
    let channel = locked_state.get_sip_channel().unwrap();

    let via: Via = request.via_header().unwrap().typed().unwrap();

    match request.method {
        Method::Ack => todo!(),
        Method::Bye => {
            settings.flow = Direction::Inbound;
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
        Method::Cancel => todo!(),
        Method::Info => todo!(),
        Method::Invite => todo!(),
        Method::Message => todo!(),
        Method::Notify => {}
        Method::Options => {
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
        Method::PRack => todo!(),
        Method::Publish => todo!(),
        Method::Refer => todo!(),
        Method::Register => todo!(),
        Method::Subscribe => todo!(),
        Method::Update => todo!(),
    }
}
pub fn process_response_outbound(
    response: &Response,
    conf: &JSONConfiguration,
    state: &Arc<Mutex<State>>,
    settings: &mut SelfConfiguration,
) {
    match response.status_code {
        StatusCode::Trying => {}
        StatusCode::Unauthorized | StatusCode::ProxyAuthenticationRequired => {
            info!("o/composing register response");

            let auth = WwwAuthenticate::try_from(
                header_opt!(response.headers().iter(), Header::WwwAuthenticate)
                    .unwrap()
                    .clone(),
            )
            .unwrap();

            let mut transaction: Option<String> = None;
            {
                let state: Arc<Mutex<State>> = state.clone();
                let mut locked_state = state.lock().unwrap();
                let mut dialogs = locked_state.get_dialogs().unwrap();

                for dg in dialogs.iter_mut().rev() {
                    if matches!(dg.diag_type, Direction::Outbound) {
                        let mut transactions = dg.transactions.get_transactions().unwrap();
                        let mut loop_transaction = transactions.last_mut().unwrap();
                        loop_transaction.object.nonce = Some(auth.nonce.clone());
                        loop_transaction.object.set_auth(
                            conf,
                            "INVITE",
                            &AuthModel {
                                nonce: auth.nonce.clone(),
                                realm: auth.realm.clone(),
                                qop: auth.qop.clone(),
                            },
                        );
                        loop_transaction.object.msg = Some(loop_transaction.local.clone().unwrap());
                        transaction = Some(
                            loop_transaction
                                .object
                                .push_auth_to_invite(response.status_code.clone())
                                .to_string(),
                        );
                        break;
                    }
                }
            }
            if let Some(..) = transaction {
                let state = state.clone();
                let mut locked_state = state.lock().unwrap();
                let channel = locked_state.get_sip_channel().unwrap();

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
        StatusCode::Ringing => {
            let state: Arc<Mutex<State>> = state.clone();
            let mut locked_state = state.lock().unwrap();
            let mut dialogs = locked_state.get_dialogs().unwrap();

            for dg in dialogs.iter_mut().rev() {
                if matches!(dg.diag_type, Direction::Outbound) {
                    let mut transactions = dg.transactions.get_transactions().unwrap();
                    let mut transaction = transactions.last_mut().unwrap();
                    transaction.remote = Some(SipMessage::Response(response.clone()));
                    break;
                }
            }
        }
        StatusCode::BusyHere => {}
        StatusCode::SessionProgress => {}
        StatusCode::OK => {
            let mut transaction: Option<String> = None;
            let connection: Option<IpAddr>;
            let rtp_port: Option<u16>;
            {
                let state: Arc<Mutex<State>> = state.clone();

                let mut locked_state = state.lock().unwrap();
                let mut dialogs = locked_state.get_dialogs().unwrap();

                for dg in dialogs.iter_mut().rev() {
                    if matches!(dg.diag_type, Direction::Outbound) {
                        let mut transactions = dg.transactions.get_transactions().unwrap();
                        let loop_transaction = transactions.last().unwrap();
                        if loop_transaction.local.is_some() && loop_transaction.remote.is_some() {
                            let hstr = response.clone().to_header().unwrap().to_string();

                            info!("{}", String::from_utf8_lossy(&response.body).to_string());
                            let sdp = sdp_rs::SessionDescription::try_from(
                                String::from_utf8_lossy(&response.body).to_string(),
                            );

                            connection = Some(
                                sdp.clone()
                                    .unwrap()
                                    .connection
                                    .unwrap()
                                    .connection_address
                                    .base,
                            );
                            rtp_port =
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

                            let remote_tag = get_remote_tag(&hstr);
                            let now = Utc::now();

                            let ack = SipOptions {
                                branch: format!(
                                    "z9hG4bK{}{}{}{}{}{}",
                                    now.month(),
                                    now.day(),
                                    now.hour(),
                                    now.minute(),
                                    now.second(),
                                    now.timestamp_millis()
                                ),
                                extension: conf.extension.to_string(),
                                username: conf.username.clone(),
                                sip_server: conf.sip_server.to_string(),
                                sip_port: conf.sip_port.to_string(),
                                ip: settings.ip.to_string(),
                                msg: None,
                                cld: loop_transaction.object.cld.clone(),
                                call_id: loop_transaction.object.call_id.clone(),
                                tag_local: loop_transaction.object.tag_local.clone(),
                                tag_remote: Some(remote_tag.to_string()),
                                md5: None,
                                nonce: None,
                                nc: None,
                                cnonce: None,
                                qop: false
                            };

                            let via_from_invite = loop_transaction
                                .local
                                .as_ref()
                                .unwrap()
                                .via_header()
                                .unwrap();
                            let cseq_count = loop_transaction
                                .local
                                .as_ref()
                                .unwrap()
                                .cseq_header()
                                .unwrap();
                            let contact = response.contact_header().unwrap();

                            let mut ack_transaction = Transaction {
                                object: ack.clone(),
                                local: Some(ack.create_ack(
                                    via_from_invite,
                                    response.headers.get_record_route_header_array().clone(),
                                    contact,
                                    cseq_count,
                                )),
                                remote: None,
                                tr_type: TransactionType::Ack,
                            };

                            ack_transaction.object.msg = Some(ack_transaction.object.create_ack(
                                via_from_invite,
                                response.headers.get_record_route_header_array().clone(),
                                contact,
                                cseq_count,
                            ));

                            transactions.push(ack_transaction.clone());
                            transaction = Some(ack_transaction.local.as_ref().unwrap().to_string());
                        }
                        break;
                    }
                }
            }
            if let Some(..) = transaction {
                let state = state.clone();
                let mut locked_state = state.lock().unwrap();
                let channel = locked_state.get_sip_channel().unwrap();

                let c_header = response.headers.get_contact().unwrap().to_string();
                let rr_header = response.headers.get_record_route_header_array();
                let rr_header_last = rr_header.last();

                let curi = match rr_header_last {
                    Some(h_t_s) => (
                        get_address_from_record_route(h_t_s.to_string()),
                        conf.clone().sip_port,
                    ),
                    None => get_address_from_contact(c_header),
                };

                info!("sending ACK @{:?}", curi);

                channel
                    .0
                    .send(MpscBase {
                        event: Some(SocketV4 {
                            ip: curi.0.to_string(),
                            port: curi.1,
                            bytes: transaction.unwrap().as_bytes().to_vec(),
                        }),
                        exit: false,
                    })
                    .unwrap();
            }
        }
        StatusCode::ServerTimeOut => {
            info!("something is a bit slow, getting a timeout");
        }
        StatusCode::RequestTimeout => {
            info!("something is a bit slow, getting a timeout");
        }
        _ => todo!(),
    }
}
