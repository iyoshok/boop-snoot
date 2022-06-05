#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

const LOG_DIR: &str = "logs";

use {
    flexi_logger::{
        Duplicate,
        FileSpec,
        Logger,
        WriteMode
    },
    rustls_native_certs::load_native_certs,
    std::{
        collections::HashMap,
        fs,
        io,
        process::exit,
        sync::Arc
    },
    tauri::State,
    tokio::sync::{
        mpsc,
        Mutex
    },
    tokio_rustls::rustls::RootCertStore
};

#[macro_use]
extern crate log;

mod config;
mod files;
mod message;
mod network;
mod partners;
mod window_titles;

use {
    serde::{
        Deserialize,
        Serialize
    },
    tauri::Window
};

use std::fmt::Display;

use tauri::Manager;
use window_titles::get_random_window_title;

use crate::{
    config::{
        BoopConfig,
        CONFIG_FILE
    },
    files::{
        get_object_or_default,
        save_file
    },
    message::MessageType,
    network::connect_to_server,
    partners::{
        BoopPartner,
        PARTNERS_FILE
    }
};

#[derive(Debug)]
pub enum ControlMessage {
    CloseConnection
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FrontendPartnerObject {
    nickname: String,
    user_key: String,
    online:   i8
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PartnerOnlineStatus {
    Afk = -1,
    Unknown = 0,
    Online = 1
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ServerConnectionStatus {
    Disconnected = -1,
    AttemptingConnection = 0,
    Connected = 1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendErrorMessage {
    message: String
}

impl Display for FrontendErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Shorthand for the transmit half of the message channel.
pub type SinkTx = mpsc::UnboundedSender<MessageType>;
pub type ControlTx = mpsc::UnboundedSender<ControlMessage>;

/// Shorthand for the receive half of the message channel.
pub type SinkRx = mpsc::UnboundedReceiver<MessageType>;
pub type ControlRx = mpsc::UnboundedReceiver<ControlMessage>;

// state definitions
struct ConnectionInterface {
    sink:            SinkTx,
    control_channel: ControlTx
}

pub struct ConfigState(Arc<Mutex<BoopConfig>>);
pub struct PartnersState(Arc<Mutex<HashMap<String, (BoopPartner, PartnerOnlineStatus)>>>);
pub struct ConnectionState(Arc<Mutex<Option<ConnectionInterface>>>);
pub struct TrustAnchors(RootCertStore);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct BoopPayload {
    partner_key: String
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PartnerUpdatePayload {
    user_key: String,
    online:   i8
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct ConnectionStatusPayload {
    status: i8
}

fn main() {
    // initialize logger
    init_logging();

    // get config
    let config: BoopConfig = get_object_or_default(&CONFIG_FILE.to_owned());

    // get saved partners and build hashmap
    let partners: Vec<BoopPartner> = get_object_or_default(&PARTNERS_FILE.to_owned());
    let mut partners_hashmap = HashMap::new();
    for partner in partners {
        partners_hashmap.insert(partner.user_key(), (partner, PartnerOnlineStatus::Unknown));
    }

    // initialize cert store
    let cert_store = match init_trust_anchors() {
        Ok(cert_store) => cert_store,
        Err(err) => {
            error!("failed to initialize trust anchors: {}", err);
            exit(-1);
        }
    };

    tauri::Builder::default()
        .manage(ConnectionState(Arc::new(Mutex::new(None))))
        .manage(ConfigState(Arc::new(Mutex::new(config))))
        .manage(PartnersState(Arc::new(Mutex::new(partners_hashmap))))
        .manage(TrustAnchors(cert_store))
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            let _ = main_window.set_title(&get_random_window_title())?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            get_settings,
            save_settings,
            get_partners,
            add_or_update_partner,
            del_partner,
            show_main_window,
            boop
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_logging() {
    fs::create_dir_all(LOG_DIR).expect("failed to create logging directory");
    Logger::try_with_str("debug")
        .expect("failed to set logging configuration")
        .log_to_file(
            FileSpec::default()
                .directory(LOG_DIR)
                .basename("boop_client")
        )
        .write_mode(WriteMode::BufferAndFlush)
        .duplicate_to_stdout(Duplicate::Info)
        .start()
        .expect("failed to initialize logging");
    debug!("log initialized");
}

fn init_trust_anchors() -> Result<RootCertStore, io::Error> {
    let certs = load_native_certs()?;
    debug!("found {} OS certificates", certs.len());

    let mut roots = RootCertStore::empty();
    for cert in certs {
        roots
            .add(&tokio_rustls::rustls::Certificate(cert.0))
            .unwrap();
    }

    Ok(roots)
}

#[tauri::command]
async fn get_settings<'a>(state: State<'a, ConfigState>) -> Result<BoopConfig, ()> {
    Ok(state.0.lock().await.clone())
}

#[tauri::command]
async fn save_settings<'a>(
    new_settings: BoopConfig,
    state: State<'a, ConfigState>
) -> Result<(), ()> {
    let mut config = state.0.lock().await;
    debug!("saving settings");

    // save changes to disk
    if let Err(err) = save_file(&CONFIG_FILE.to_owned(), &new_settings).await {
        error!("failed to save new settings to disk: {}", err);
        return Err(());
    }

    // save changes to state
    *config = new_settings;

    Ok(())
}

#[tauri::command]
async fn add_or_update_partner<'a>(
    partner: BoopPartner,
    state: State<'a, PartnersState>
) -> Result<(), ()> {
    let mut partners = state.0.lock().await;

    // update state
    let old_val_option = partners.insert(
        partner.user_key(),
        (partner.clone(), PartnerOnlineStatus::Unknown)
    );

    // save changes to disk and roll state changes back if the disk write failed
    let disk_write_result = save_partners_changes(&partners).await;
    if let Err(_) = disk_write_result {
        // uh oh something went wrong while saving -> restore previous state so disk and
        // memory state match
        if let Some(old_val) = old_val_option {
            // previous value was overwritten -> restore previous value
            let _ = partners.insert(partner.user_key(), old_val);
        } else {
            // the value was newly created -> delete
            let _ = partners.remove(&partner.user_key());
        }
    }

    // the success of this operation is bound to the success of the disk write, so
    // just return that
    disk_write_result
}

#[tauri::command]
async fn del_partner<'a>(
    partner_key: String,
    state: State<'a, PartnersState>
) -> Result<(), ()> {
    let mut partners = state.0.lock().await;

    // update state
    let old_val_option = partners.remove(&partner_key);

    // save changes to disk and roll state changes back if the disk write failed
    let disk_write_result = save_partners_changes(&partners).await;
    if let Err(_) = disk_write_result {
        // uh oh something went wrong while saving -> restore previous state so disk and
        // memory state match
        if let Some(old_val) = old_val_option {
            // previous value was overwritten -> restore previous value
            let _ = partners.insert(partner_key, old_val);
        }

        // else: if the value didn't exist beforehand (old_val_option == None) ,
        // we didn't delete it and therefore didn't alter the state, so do
        // nothing :)
    }

    // the success of this operation is bound to the success of the disk write, so
    // just return that
    disk_write_result
}

#[tauri::command]
async fn get_partners<'a>(state: State<'a, PartnersState>) -> Result<Vec<FrontendPartnerObject>, ()> {
    let partners = state.0.lock().await;
    Ok(get_partners_payload(&*partners))
}

#[tauri::command]
async fn connect(
    conn_state: State<'_, ConnectionState>,
    config_state: State<'_, ConfigState>,
    partners_state: State<'_, PartnersState>,
    trust_anchors: State<'_, TrustAnchors>,
    window: Window
) -> Result<bool, ()> {
    let boxed_window = Arc::new(window);
    match connect_to_server(
        conn_state,
        config_state,
        partners_state,
        trust_anchors,
        Arc::clone(&boxed_window)
    )
    .await
    {
        Ok(logged_in) => {
            info!("logged in? {}", logged_in);
            if !logged_in {
                send_connection_status(&boxed_window, ServerConnectionStatus::Disconnected);
            }
            Ok(logged_in) // tells the frontend whether the login was accepted
                          // or not
        }
        Err(e) => {
            send_connection_status(&boxed_window, ServerConnectionStatus::Disconnected);

            // error logging
            error!("opening the new connection has failed, details follow");
            if let Some(err) = e.downcast_ref::<tokio::io::Error>() {
                error!("I/O error: {}", err);
            } else if let Some(err) = e.downcast_ref::<std::io::Error>() {
                error!("I/O error: {}", err);
            } else {
                error!("something weird happened: {}", e);
            }

            Err(())
        }
    }
}

#[tauri::command]
async fn disconnect(conn_state: State<'_, ConnectionState>) -> Result<(), ()> {
    // lock current connection interface, close the connection, clear the handle and
    // keep the lock to make sure no other process tries to access the
    // connection during this connect call send close message to the current
    // stream handler -> will close the existing connection
    let mut interface_option = conn_state.0.lock().await;
    if let Some(conn_interface) = &*interface_option {
        // check if channel is still open (the connection might have been terminated
        // unexpectedly before with the control channel going out of scope)
        if !conn_interface.control_channel.is_closed() {
            let close_res = conn_interface
                .control_channel
                .send(ControlMessage::CloseConnection);
            if let Err(err) = close_res {
                error!("failed to close connection as requested: {}", err);
            }
        }
    }
    // drop the interface -> its locked so existing connections should be
    // interrupted until the new interface is built
    *interface_option = None;

    Ok(())
}

#[tauri::command]
async fn boop(partner_key: String, connection_state: State<'_, ConnectionState>) -> Result<(), ()> {
    let connection_interface = connection_state.0.lock().await;

    if let Some(connections) = &*connection_interface {
        if !connections.sink.is_closed() {
            if let Err(err) = connections.sink.send(MessageType::BOOP(partner_key)) {
                error!("failed to send boop to sink: {}", err);
                return Err(());
            }
        } else {
            warn!("client tried to boop, but the sink channel was closed");
        }
    } else {
        warn!("client tried to boop without an active server connection");
    }

    Ok(())
}

#[tauri::command]
async fn show_main_window(window: tauri::Window) {
    // Show main window
    window.get_window("main").unwrap().show().unwrap();
}

pub fn send_partners_update_event(window: &Window, user_key: &String, status: PartnerOnlineStatus) {
    debug!("sending partners-update event to frontend");
    if let Err(err) = window.emit_all("partner-status-changed", PartnerUpdatePayload {
        user_key: user_key.clone(),
        online:   status as i8
    }) {
        error!("failed to send partners update to frontend: {}", err);
    }
}

/// Transforms the partners state to an event payload intended for frontend
/// partner updates
fn get_partners_payload(
    partners: &HashMap<String, (BoopPartner, PartnerOnlineStatus)>
) -> Vec<FrontendPartnerObject> {
    let mut vec = Vec::new();

    for (_, (partner, status)) in partners {
        vec.push(FrontendPartnerObject {
            nickname: partner.nickname(),
            user_key: partner.user_key(),
            online:   *status as i8
        })
    }

    vec
}

async fn save_partners_changes(
    partners: &HashMap<String, (BoopPartner, PartnerOnlineStatus)>
) -> Result<(), ()> {
    let partner_config: Vec<BoopPartner> = partners
        .iter()
        .map(|(_, (partner_object, _))| partner_object.clone())
        .collect();

    if let Err(err) = save_file(&PARTNERS_FILE.to_owned(), &partner_config).await {
        error!("failed to save changed partners config to disk: {}", err);
        return Err(());
    }

    Ok(())
}

pub fn send_error_to_frontend(window: &Window, error_message: FrontendErrorMessage) {
    debug!("sending frontend error message: {}", &error_message);
    let emit_res = window.emit_all("backend-error", error_message);
    if let Err(send_err) = emit_res {
        error!("failed to send frontend error: {}\n", send_err);
    }
}

pub fn send_boop_to_frontend(window: &Window, partner_key: String) {
    debug!("transmitting boop by {} to frontend", partner_key);
    let emit_res = window.emit_all("booped", BoopPayload { partner_key });
    if let Err(send_err) = emit_res {
        error!("failed to send boop to frontend: {}\n", send_err);
    }
}

pub fn send_connection_status(window: &Window, conn_status: ServerConnectionStatus) {
    let status = conn_status as i8;
    debug!("sending connection status change {} to frontend", status);
    let emit_res = window.emit_all("connection-state-changed", ConnectionStatusPayload {
        status
    });
    if let Err(send_err) = emit_res {
        error!(
            "failed to send connection status change to frontend: {}\n",
            send_err
        );
    }
}
