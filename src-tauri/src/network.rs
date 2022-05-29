use std::sync::Arc;

use tauri::Window;

use crate::{
    message::{
        create_message_text,
        error_text,
        parse_message,
        MessageErrorKind,
        MessageType
    },
    partners::BoopPartner,
    send_boop_to_frontend,
    send_connection_status,
    send_error_to_frontend,
    send_partners_update_event,
    FrontendErrorMessage,
    PartnerOnlineStatus,
    ServerConnectionStatus
};

use {
    crate::{
        ConfigState,
        ConnectionInterface,
        ConnectionState,
        ControlMessage,
        ControlRx,
        ControlTx,
        PartnersState,
        SinkRx,
        SinkTx,
        TrustAnchors
    },
    std::{
        collections::HashMap,
        io::{
            self,
            Error
        },
        net::{
            SocketAddr,
            ToSocketAddrs
        },
        time::Duration
    },
    tauri::State,
    tokio::{
        io::{
            split,
            AsyncBufReadExt,
            AsyncWriteExt,
            BufReader,
            ReadHalf,
            WriteHalf
        },
        net::TcpStream,
        sync::{
            mpsc::unbounded_channel,
            Mutex
        }
    },
    tokio_rustls::{
        client::TlsStream,
        rustls::ClientConfig,
        TlsConnector
    }
};

const PING_INTERVAL: u64 = 5;
const ALLOWED_PING_MISSED: u32 = 3;
const PARTNER_CHECK_INTERVAL: u64 = 15;

type Writer = WriteHalf<TlsStream<TcpStream>>;

pub async fn connect_to_server<'a>(
    conn_state: State<'_, ConnectionState>,
    config_state: State<'_, ConfigState>,
    partners_state: State<'_, PartnersState>,
    trust_anchors: State<'_, TrustAnchors>,
    window: Arc<Window>
) -> Result<bool, Box<dyn std::error::Error + 'a>> {
    send_connection_status(&window, ServerConnectionStatus::AttemptingConnection);

    // lock config and get necessary data
    let (addr, domain);
    let (user, password);
    {
        let app_settings = config_state.0.lock().await;

        // parse address and domain + resolve dns for next steps
        (addr, domain) = parse_server_address(&app_settings.server_address())?;
        user = app_settings.user_name();
        password = app_settings.password();
    }

    // lock current connection interface, close the connection, clear the handle and
    // keep the lock to make sure no other process tries to access the
    // connection during this connect call send close message to the current
    // stream handler -> will close the existing connection
    let mut interface_option = conn_state.0.lock().await;
    if let Some(conn_interface) = &*interface_option {
        // check if channel is still open (the connection might have been terminated
        // unexpectedly before with the control channel going out of scope)
        if !conn_interface.control_channel.is_closed() {
            conn_interface
                .control_channel
                .send(ControlMessage::CloseConnection)?;
        }
    }
    // drop the interface -> its locked so existing connections should be
    // interrupted until the new interface is built
    *interface_option = None;

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(trust_anchors.0.clone())
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));

    // connect to socket
    let stream = TcpStream::connect(&addr).await?;

    debug!("{}", &domain);
    let domain = rustls::ServerName::try_from(domain.as_str())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?;

    // handshake tls etc
    let stream = connector.connect(domain, stream).await?;

    // create connection interface
    let (sink_tx, sink_rx): (SinkTx, SinkRx) = unbounded_channel();
    let (control_tx, control_rx): (ControlTx, ControlRx) = unbounded_channel();

    // put connection interface in state
    *interface_option = Some(ConnectionInterface {
        control_channel: control_tx,
        sink:            sink_tx
    });

    // split stream and create reader for connection
    let (readhalf, mut writehalf) = split(stream);
    let reader = BufReader::new(readhalf);

    // handshake with server / login
    let (login_correct, mut reader) = handshake(reader, &mut writehalf, user, password).await?;
    if !login_correct {
        return Ok(false);
    }

    // clone partners handle to use in the connection thread
    let partners_handle = Arc::clone(&partners_state.0);

    // change connection status
    send_connection_status(&window, ServerConnectionStatus::Connected);

    // start background activity
    tokio::spawn(async move {
        if let Err(err) = rw_loop(
            &mut reader,
            writehalf,
            partners_handle,
            sink_rx,
            control_rx,
            &window
        )
        .await
        {
            // log error
            error!("disconnected after connection I/O error: {}", err);
            // signal frontend that connection was closed
            send_error_to_frontend(&window, FrontendErrorMessage {
                message: "Server connection was closed unexpectedly, see log for details".into()
            });

            // change connection status in frontend
            send_connection_status(&window, ServerConnectionStatus::Disconnected);
        } else {
            info!("closed connection as expected");
        }
    });

    Ok(true)
}

fn parse_server_address(server_address: &String) -> Result<(SocketAddr, String), io::Error> {
    let addr = server_address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

    // split off port
    let colon_search_result = server_address.rfind(":");
    if let Some(idx) = colon_search_result {
        let domain: String = server_address.chars().take(idx).collect();
        debug!("extracted domain: {}", &domain);

        Ok((addr, domain))
    } else {
        error!("server address doesn't include port");
        Err(io::Error::from(io::ErrorKind::NotFound))
    }
}

async fn handshake<'a>(
    mut reader: BufReader<ReadHalf<TlsStream<TcpStream>>>,
    writehalf: &mut Writer,
    user: String,
    password: String
) -> Result<(bool, BufReader<ReadHalf<TlsStream<TcpStream>>>), io::Error> {
    // set up receiver so we don't miss the message
    let read_thread_handle = tokio::spawn(async move {
        loop {
            let mut buf = String::new();
            let read = reader.read_line(&mut buf).await;
            if let Ok(n) = read {
                if n == 0 {
                    // EOF / stream closed
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }

                let parse_res = parse_message(&buf);
                if let Err(err) = parse_res {
                    error!("couldn't parse the received server message: {}", err);
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
                }
                let received_message = parse_res.unwrap();
                return if received_message == MessageType::HEY {
                    Ok((true, reader))
                } else if received_message == MessageType::NO {
                    Ok((false, reader))
                } else {
                    error!(
                        "protocol mismatch by server, expected HEY/NO, got: {}",
                        create_message_text(received_message)
                    );
                    Err(io::Error::from(io::ErrorKind::InvalidData))
                };
            } else {
                // other read error
                read?;
            }
        }
    });

    // send login message
    send_message(writehalf, MessageType::CONNECT(user, password)).await?;

    // wait for server response and return
    let res = read_thread_handle.await;
    if let Err(err) = res {
        panic!(
            "something weird happening while doing the protocol handshake: {}",
            err
        );
    }

    res.unwrap()
}

async fn rw_loop(
    reader: &mut BufReader<ReadHalf<TlsStream<TcpStream>>>,
    mut writehalf: Writer,
    partners_handle: Arc<Mutex<HashMap<String, (BoopPartner, PartnerOnlineStatus)>>>,
    mut sink_rx: SinkRx,
    mut control_rx: ControlRx,
    window: &Window
) -> io::Result<()> {
    // create watchdog for pings
    let mut ping_watchdog = tokio::time::interval(Duration::from_secs(PING_INTERVAL));
    ping_watchdog.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay); // if tick is missed, fire next tick asap and then wait the full afk timeout
                                                                                    // again

    let mut partner_watchdog = tokio::time::interval(Duration::from_secs(PARTNER_CHECK_INTERVAL));
    partner_watchdog.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay); // if tick is missed, fire next tick asap and then wait the full afk timeout
                                                                                       // again

    let mut missed_pongs: u32 = 0;

    loop {
        let mut buf = String::new();
        tokio::select! {
          _ = ping_watchdog.tick() => {
            send_pings_and_check_misses(&mut missed_pongs, &mut writehalf).await?;
          },
          _ = partner_watchdog.tick() => {
            check_partner_availability(&partners_handle, &mut writehalf).await?
          },
          Some(msg) = sink_rx.recv() => {
            send_message(&mut writehalf, msg).await?;
          },
          Some(control_msg) = control_rx.recv() => {
            handle_control_msg(control_msg, &mut writehalf).await?;
          },
          res = reader.read_line(&mut buf) => {
            handle_message_input(res, &buf, &partners_handle, &mut missed_pongs, &mut writehalf, window).await?;
          }
        }
    }
}

async fn send_pings_and_check_misses(
    missed_pongs: &mut u32,
    writehalf: &mut Writer
) -> io::Result<()> {
    if *missed_pongs > ALLOWED_PING_MISSED {
        // too many misses -> disconnect
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "closed connection, server missed too many pings"
        ))
    } else {
        // none or acceptable number of misses -> send new ping
        debug!(
            "sent ping to server, current pong miss counter: {}",
            missed_pongs
        );
        *missed_pongs += 1;
        send_message(writehalf, MessageType::PING).await
    }
}

async fn check_partner_availability(
    partners_handle: &Arc<Mutex<HashMap<String, (BoopPartner, PartnerOnlineStatus)>>>,
    writehalf: &mut Writer
) -> io::Result<()> {
    // locks partners map
    let partners = partners_handle.lock().await;

    // sends "are you there?" msg for every known partner
    for partner_key in partners.keys() {
        debug!("asked for online status of partner: {}", partner_key);
        send_message(writehalf, MessageType::AYT(partner_key.clone())).await?;
    }

    // none of the previous requests failed -> seems okay
    Ok(())
}

async fn handle_message_input(
    res: io::Result<usize>,
    buf: &String,
    partners_handle: &Arc<Mutex<HashMap<String, (BoopPartner, PartnerOnlineStatus)>>>,
    missed_pongs: &mut u32,
    writehalf: &mut Writer,
    window: &Window
) -> io::Result<()> {
    match res {
        Ok(n) => {
            if false && n == 0 {
                // EOF while reading
                debug!("received 0 buffer length");
                return Err(Error::from(io::ErrorKind::UnexpectedEof));
            }

            let parse_result = parse_message(buf);
            if let Ok(msg) = parse_result {
                match msg {
                    MessageType::BOOP(partner_key) => {
                        // log boop to logger
                        info!("got booped by {}", &partner_key);
                        // transmit boop to frontend
                        send_boop_to_frontend(window, partner_key);
                    }
                    MessageType::BYE => {
                        // server says goodbye after disconnect
                        return Ok(());
                    }
                    MessageType::PONG => {
                        *missed_pongs = 0;
                    }
                    MessageType::ERROR(err) => {
                        error!("server reported error: {}", error_text(err));
                    }
                    MessageType::ONLINE(partner_key) => {
                        let mut partners = partners_handle.lock().await;
                        if let Some(entry) = partners.get_mut(&partner_key) {
                            entry.1 = PartnerOnlineStatus::Online;
                        }

                        // update frontend
                        send_partners_update_event(window, &*partners);
                    }
                    MessageType::AFK(partner_key) => {
                        let mut partners = partners_handle.lock().await;
                        if let Some(entry) = partners.get_mut(&partner_key) {
                            entry.1 = PartnerOnlineStatus::Afk;
                        }

                        // update frontend
                        send_partners_update_event(window, &*partners);
                    }
                    _ => {
                        // against protocol -> disconnect
                        return send_error_and_close(writehalf, MessageErrorKind::ProtocolMismatch)
                            .await;
                    }
                }
            } else {
                // not standard-compliant
                send_error_and_close(writehalf, parse_result.unwrap_err().into()).await?;
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "couldn't parse the received message"
                ));
            }
        }
        Err(err) => return Err(err)
    }

    // none of the other logic failed -> should be okay
    Ok(())
}

async fn handle_control_msg(control_msg: ControlMessage, writehalf: &mut Writer) -> io::Result<()> {
    // handles all logic involved with the control channel from the mainthread to
    // the connection loop
    match control_msg {
        ControlMessage::CloseConnection => send_message(writehalf, MessageType::DISCONNECT).await
    }
}

async fn send_error_and_close(writehalf: &mut Writer, err: MessageErrorKind) -> io::Result<()> {
    send_message_and_close(writehalf, MessageType::ERROR(err)).await
}

async fn send_message_and_close(writehalf: &mut Writer, message: MessageType) -> io::Result<()> {
    send_message(writehalf, message).await?;
    writehalf.shutdown().await
}

async fn send_message(writehalf: &mut Writer, message: MessageType) -> io::Result<()> {
    let msg_text = create_message_text(message);
    writehalf.write_all(msg_text.as_bytes()).await
}
