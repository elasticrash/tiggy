fn rtp_event_loop() {
    let builder = thread::Builder::new();
    builder.spawn(move || {
        let mut settings = SelfConfiguration {
            flow: Direction::Inbound,
            verbosity: Verbosity::Minimal,
            ip: &ip,
        };

        let mut sip_buffer = [0_u8; 65535];
        let mut rtp_buffer = [0_u8; 65535];

        let dialog_state: Arc<Mutex<Dialogs>> = Arc::new(Mutex::new(Dialogs::new(stx, rtx)));
        {
            let _io_result = sip_socket.set_read_timeout(Some(Duration::new(1, 0)));

            rtp_socket
                .connect(format!("{}:{}", &conf.sip_server, &conf.sip_port))
                .expect("connect function failed");
        }

        register_ua(&dialog_state, &conf, &mut settings);
        let action_menu = Arc::new(build_menu());

        'thread: loop {
            // peek on the socket, for pending messages
            let mut maybe_msg: Option<SipMessage> = None;
            {
                let packets_queued = peek(&mut sip_socket, &mut sip_buffer);

                if packets_queued > 0 {
                    maybe_msg = match receive(
                        &mut sip_socket,
                        &mut sip_buffer,
                        &settings.verbosity,
                        &thread_logs,
                    ) {
                        Ok(buf) => Some(buf),
                        Err(_) => None,
                    };
                }
            }

            let mut rtp_packet: Option<&str> = None;
            {
                let packets_queued = peek(&mut rtp_socket, &mut rtp_buffer);

                if packets_queued > 0 {
                    let (amt, _src) = rtp_socket.recv_from(&mut rtp_buffer).unwrap();
                    let slice = &mut rtp_buffer[..amt];
                    let rtp_packet = String::from_utf8_lossy(slice);
                }
            }

            // distribute message on the correct process
            if let Some(..) = maybe_msg {
                let msg = maybe_msg.unwrap();
                match settings.flow {
                    Direction::Inbound => match msg {
                        rsip::SipMessage::Request(request) => {
                            process_request_inbound(&request, &conf, &dialog_state, &mut settings)
                        }
                        rsip::SipMessage::Response(response) => {
                            process_response_inbound(&response, &conf, &dialog_state)
                        }
                    },
                    Direction::Outbound => match msg {
                        rsip::SipMessage::Request(request) => {
                            process_request_outbound(&request, &conf, &dialog_state, &mut settings)
                        }
                        rsip::SipMessage::Response(response) => process_response_outbound(
                            &response,
                            &conf,
                            &dialog_state,
                            &mut settings,
                        ),
                    },
                }
            }

            // send a command for processing
            if let Ok(processable_object) = rx.try_recv() {
                log::slog(
                    format!("received input, {:?}", processable_object.bind).as_str(),
                    &thread_logs,
                );

                if send_menu_commands(
                    &processable_object,
                    &dialog_state,
                    &action_menu,
                    &conf,
                    &mut settings,
                    &ip,
                    &thread_logs,
                ) {
                    break 'thread;
                }
            }

            if let Ok(data) = srx.try_recv() {
                send(&mut sip_socket, &data, &settings.verbosity, &thread_logs);
            }

            if let Ok(data) = rrx.try_recv() {
                send(&mut rtp_socket, &data, &settings.verbosity, &thread_logs);
            }
        }
    })
}
