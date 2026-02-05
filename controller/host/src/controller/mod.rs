//! This module handles communication with the controller firmware via poststation.
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use icd::{BakingState, BcInstStatus, LightState, ValveState, VctHandshake};
use poststation_sdk::{PoststationClient, connect};
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};

use crate::{
    logger::{LogMessage, send_log_message, send_log_message_now},
    status::InstrumentStatus,
};

mod client;

use client::ControllerClient;

pub enum ControllerCommands {
    Light(LightState),
    Baking(BakingState),
    TransferValve(ValveState),
    PumpValve(ValveState),
    VctHandshake(VctHandshake),
}

/// Start the controller tasks
///
/// Starts the two controller tasks:
/// - Listen to commands.
/// - Listen to broadcasts and react on them.
///
/// # Arguments
/// - `config`: Configuration for the controller.
/// - `inst_status`: Shared instrument status to update from broadcasts.
/// - `rx_ctrl`: Receiver for controller commands.
///
/// # Returns
/// Tuple of joint handles for the two async tasks, which we will await later in main.
pub async fn start_controller_tasks(
    config: ControllerConfig,
    inst_status: Arc<Mutex<InstrumentStatus>>,
    rx_ctrl: mpsc::Receiver<ControllerCommands>,
) -> (JoinHandle<()>, JoinHandle<()>) {
    let ps_client = connect(config.address)
        .await
        .expect("Poststation connection must work. Is poststation running?");

    let serial = if let Some(device) = ps_client
        .get_devices()
        .await
        .expect("Poststation must return list of devices")
        .iter()
        .find(|d| d.product == Some(config.product_name.clone()) && d.is_connected)
    {
        device.serial
    } else {
        panic!("No '{}' device found in poststation.", config.product_name);
    };

    // Controller command task
    let cntrl_client = ControllerClient::new(ps_client.clone(), serial);
    let cntrl_task = tokio::spawn(controller_task(
        cntrl_client,
        rx_ctrl,
        Arc::clone(&inst_status),
    ));

    // Broadcast listener task.
    let cntrl_bc_task = tokio::spawn(controller_broadcast_listener(
        ps_client,
        serial,
        inst_status,
    ));

    (cntrl_task, cntrl_bc_task)
}

/// Task that communicates with the controller firmware via poststation.
pub async fn controller_task(
    cntrl: ControllerClient,
    mut rx: mpsc::Receiver<ControllerCommands>,
    inst_status: Arc<Mutex<InstrumentStatus>>,
) {
    let mut rx_shutdown = crate::HALT_SENDER.get().unwrap().subscribe();

    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                match cmd {
                    ControllerCommands::Light(state) => {
                        cntrl.light(state).await;
                    }
                    ControllerCommands::Baking(state) => {
                        cntrl.baking(state).await;
                    }
                    ControllerCommands::PumpValve(state) => {
                        inst_status.lock().expect("Poisoned").set_valve_pump_call(state.clone());
                        cntrl.pump_valve(state).await;
                    }
                    ControllerCommands::TransferValve(state) => {
                        inst_status.lock().expect("Poisoned").set_valve_transfer_call(state.clone());
                        cntrl.transfer_valve(state).await;
                    }
                    ControllerCommands::VctHandshake(handshake) => {
                        cntrl.vct_handshake(handshake).await;
                    }
                }
            }
            _ = sleep(Duration::from_secs(10)) => {
                // keep alive task, query unique ID every minute to keep stuff alive
                cntrl.keep_alive().await;
            }
            _ = rx_shutdown.recv() => {
                println!("Controller command handling task shutting down");
                break;
            }
        }
    }
}

/// Task that listens to broadcast messages from the controller firmware via poststation.
///
/// This task also updates the UI accordingly.
pub async fn controller_broadcast_listener(
    client: PoststationClient,
    serial: u64,
    inst_status: Arc<Mutex<InstrumentStatus>>,
) {
    let mut rx_shutdown = crate::HALT_SENDER.get().unwrap().subscribe();

    let listener_wait_time = Duration::from_millis(icd::BROADCAST_INTERVAL_MS * 2);

    let mut run_initialize = true; // If true, runs initialize after broadcast

    loop {
        tokio::select! {
            stream_result = client.stream_topic::<BcInstStatus>(serial) => {

                match stream_result {
                    Ok(mut listener) => {
                        tokio::select! {
                            msg = listener.recv() => {
                                if let Some(status) = msg {
                                    inst_status.lock().expect("Poisoned").update_from_controller_broadcast(status);

                                    // initialization on first start, only if UI is set
                                    if run_initialize && inst_status.lock().expect("Poisoned").initialize_call_states_from_bc().is_ok() {
                                            run_initialize = false;
                                        };
                                } else {
                                    send_log_message(LogMessage::new_error("Poststation broadcast stream closed unexpectedly.")).await;
                                }
                            }
                            _ = sleep(listener_wait_time) => {
                                    send_log_message(LogMessage::new_warning("Poststation listener timed out while waiting for broadcast message.")).await;
                            }
                        }
                    }
                    Err(e) => {
                        if e.to_string().contains("Device Disconnected") {
                            send_log_message(LogMessage::new_error(&format!(
                                "Controller is disconnected: Please check the connection. Retry in {} ms.", icd::BROADCAST_INTERVAL_MS
                            )))
                            .await;
                        } else {
                            send_log_message(LogMessage::new_error(
                                "Connection to poststation lost. Ensure poststation is running and restart this program."
                            )).await;
                            break;
                        }
                        sleep(Duration::from_millis(icd::BROADCAST_INTERVAL_MS)).await;
                    }
                }
            }
            _ = rx_shutdown.recv() => {
                println!("Controller broadcast listener shutting down");
                break;
            }
        }
    }
}

/// Get a clone of the controller command sender.
fn get_cntrl_cmd_sender() -> mpsc::Sender<ControllerCommands> {
    crate::CONTROLLER_COMMAND_SENDER
        .get()
        .expect("Uninitialized")
        .clone()
}

/// Convenience function to await sending a controller command.
///
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub async fn send_cntrl_cmd(cmd: ControllerCommands) {
    let sender = get_cntrl_cmd_sender();
    if let Err(e) = sender.send(cmd).await {
        send_log_message_now(LogMessage::new_error(
            &format!("Failed to send controller command: {}", e),
        ));
    }
}

/// Convenience function to send a controller command without awaiting.
///
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub fn send_cntrl_cmd_now(cmd: ControllerCommands) {
    let sender = get_cntrl_cmd_sender();
    if let Err(e) = sender.try_send(cmd) {
        send_log_message_now(LogMessage::new_error(
            &format!("Failed to send controller command now: {}", e),
        ));
    }
}

/// A structure that holds the configuration for the controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    /// The product name that is displayed in poststation.
    pub product_name: String,
    /// Address and port of the poststation serve.
    pub address: String,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            product_name: String::from("Cryostorage Controller"),
            address: String::from("127.0.0.1:51837"),
        }
    }
}
