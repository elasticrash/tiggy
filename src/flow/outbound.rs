use crate::{
    commands::{auth::Auth, helper::get_remote_tag, ok::ok},
    composer::header_extension::CustomHeaderExtension,
    config::JSONConfiguration,
    rtp::{MutableRtpPacket, RtpType},
    slog::udp_logger,
    state::{
        dialogs::{Dialog, State, Direction, Transactions},
        options::{SelfConfiguration, SipOptions, Verbosity},
        transactions::{Transaction, TransactionType},
    },
    transmissions::sockets::{MpscBase, SocketV4},
};

use chrono::prelude::*;
use pnet_macros_support::packet::Packet;
use rand::Rng;
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
        Method::Notify => todo!(),
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
                        loop_transaction.object.nonce = Some(auth.nonce);
                        loop_transaction.object.set_auth(conf, "INVITE");
                        loop_transaction.object.msg = Some(loop_transaction.local.clone().unwrap());
                        transaction =
                            Some(loop_transaction.object.push_auth_to_invite().to_string());
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
            let mut connection: Option<IpAddr> = None;
            let mut rtp_port: Option<u16> = None;
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

            // START NEW THREAD ON THE ABOVE TO RECEIVE PACKETS
            // rtp::event_loop::rtp_event_loop(&settings.ip, 49152, state.clone());

            if connection.is_some() && rtp_port.is_some() {
                let state = state.clone();
                info!("target rtp located : {:?}:{:?}", connection, rtp_port);
                info!("source rtp located : {:?}:{}", settings.ip, 49152);
                info!("starting rtp event loop");

                let mut rng = rand::thread_rng();
                let n1: u32 = rng.gen();
                let n2: u16 = rng.gen();
                let n3: u32 = rng.gen();

                info!("constructing first rtp packet");
                let mut state_for_rtp = state.lock().unwrap();

                let channel = state_for_rtp.get_rtp_channel().unwrap();
                let mut rtp_buffer = [0_u8; 24];
                let mut packet = MutableRtpPacket::new(&mut rtp_buffer).unwrap();
                packet.set_version(2);
                packet.set_payload_type(RtpType::Pcma);
                packet.set_sequence(n2);
                packet.set_timestamp(n1);
                packet.set_ssrc(n3);

                info!("sending rtp packet");
                channel
                    .0
                    .send(MpscBase {
                        event: Some(SocketV4 {
                            ip: connection.unwrap().to_string(),
                            port: rtp_port.unwrap(),
                            bytes: packet.consume_to_immutable().packet().to_vec(),
                        }),
                        exit: false,
                    })
                    .unwrap();
            }
        }
        _ => todo!(),
    }
}
