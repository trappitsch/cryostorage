//! This module handles communication with the controller firmware via poststation.
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use icd::{
    BakingState, BcInstStatus, LightState, SetLightEndpoint, SetPumpValveEndpoint,
    SetTransferValveEndpoint, SetVctHandshakeEndpoint, ValveState, VctHandshake,
};
use poststation_sdk::PoststationClient;
use serde::{Deserialize, Serialize};
use tokio::{time::sleep, sync::mpsc};

use crate::{get_log_sender, logger::LogMessage, status::InstrumentStatus};

pub enum ControllerCommands {
    Light(LightState),
    Baking(BakingState),
    TransferValve(ValveState),
    PumpValve(ValveState),
    VctHandshake(VctHandshake),
}

/// Task that communicates with the controller firmware via poststation.
pub async fn controller_task(cntrl: Controller, mut rx: mpsc::Receiver<ControllerCommands>) {
    let mut rx_shutdown = crate::HALT_SENDER.get().unwrap().subscribe();

    let log_sender = crate::get_log_sender();
    log_sender
        .try_send(LogMessage::new_info("Controller task started"))
        .unwrap();

    loop {
        tokio::select! {
            command_result = rx.recv() => {
                if let Some(cmd) = command_result {
                    match cmd {
                        ControllerCommands::Light(state) => {
                            cntrl.light(state).await;
                        }
                        ControllerCommands::Baking(state) => {
                            cntrl.baking(state).await;
                        }
                        ControllerCommands::PumpValve(state) => {
                            cntrl.pump_valve(state).await;
                        }
                        ControllerCommands::TransferValve(state) => {
                            cntrl.transfer_valve(state).await;
                        }
                        ControllerCommands::VctHandshake(handshake) => {
                            cntrl.vct_handshake(handshake).await;
                        }
                    }
                }
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
    let ls = get_log_sender();

    loop {
        tokio::select! {
            stream_result = client.stream_topic::<BcInstStatus>(serial) => {
                if let Ok(mut listener) = stream_result {

                    tokio::select! {
                        msg = listener.recv() => {
                            if let Some(status) = msg {
                                inst_status.lock().expect("Poisoned").update_from_controller_broadcast(status);
                            } else {
                                ls.send(LogMessage::new_error("Poststation broadcast stream closed unexpectedly.")).await.expect("Log send must work");
                            }
                        }
                        _ = sleep(listener_wait_time) => {
                                ls.send(LogMessage::new_warning("Poststation listener timed out while waiting for broadcast message.")).await.expect("Log send must work");
                        }
                    }
                } else {
                    ls.send(LogMessage::new_error(format!("Poststation failed to connect to broadcast stream. Retry in {} ms. ", icd::BROADCAST_INTERVAL_MS).as_str())).await.expect("Log send must work");
                    sleep(Duration::from_millis(icd::BROADCAST_INTERVAL_MS)).await;
                }
            }
            _ = rx_shutdown.recv() => {
                println!("Controller broadcast listener shutting down");
                break;
            }
        }
    }
}

/// Holds the controller communication functions.
pub struct Controller {
    serial: u64,
    client: PoststationClient,
    ctr: AtomicU32,
}

impl Controller {
    pub fn new(client: PoststationClient, serial: u64) -> Self {
        Self {
            client,
            serial,
            ctr: AtomicU32::new(0),
        }
    }

    #[inline(always)]
    fn ctr(&self) -> u32 {
        self.ctr.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn baking(&self, baking_state: BakingState) {
        if self
            .client
            .proxy_endpoint::<icd::SetBakingEndpoint>(self.serial, self.ctr(), &baking_state)
            .await
            .is_err()
        {
            let ls = get_log_sender();
            ls.send(LogMessage::new_error(
                "Failed to send new baking state to controller",
            ))
            .await
            .expect("Log send must work");
        }
    }

    pub async fn light(&self, light_state: LightState) {
        if self
            .client
            .proxy_endpoint::<SetLightEndpoint>(self.serial, self.ctr(), &light_state)
            .await
            .is_err()
        {
            let ls = get_log_sender();
            ls.send(LogMessage::new_error(
                "Failed to send new light state to controller",
            ))
            .await
            .expect("Log send must work");
        }
    }

    pub async fn pump_valve(&self, valve_state: ValveState) {
        if self
            .client
            .proxy_endpoint::<SetPumpValveEndpoint>(self.serial, self.ctr(), &valve_state)
            .await
            .is_err()
        {
            let ls = get_log_sender();
            ls.send(LogMessage::new_error(
                "Failed to send new pump valve state to controller",
            ))
            .await
            .expect("Log send must work");
        }
    }

    pub async fn transfer_valve(&self, valve_state: ValveState) {
        if self
            .client
            .proxy_endpoint::<SetTransferValveEndpoint>(self.serial, self.ctr(), &valve_state)
            .await
            .is_err()
        {
            let ls = get_log_sender();
            ls.send(LogMessage::new_error(
                "Failed to send new transfer valve state to controller",
            ))
            .await
            .expect("Log send must work");
        }
    }

    pub async fn vct_handshake(&self, handshake: VctHandshake) {
        if self
            .client
            .proxy_endpoint::<SetVctHandshakeEndpoint>(self.serial, self.ctr(), &handshake)
            .await
            .is_err()
        {
            let ls = get_log_sender();
            ls.send(LogMessage::new_error(
                "Failed to send new VCT handshake to controller",
            ))
            .await
            .expect("Log send must work");
        }
    }
}

/// A structure that holds the configuration for the controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    /// The serial number of the controller -> get form poststation.
    pub serial: u64,
    /// Address and port of the poststation serve.
    pub address: String,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            serial: 123456789,
            address: String::from("127.0.0.1:51837"),
        }
    }
}
