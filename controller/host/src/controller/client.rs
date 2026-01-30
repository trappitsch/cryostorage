//! Module to handle the controller client.
//!
//! Takes a poststation client and provides the method to interact with our controller.
//!
use std::sync::atomic::{AtomicU32, Ordering};

use icd::{
    BakingState, GetUniqueIdEndpoint, LightState, SetLightEndpoint, SetPumpValveEndpoint,
    SetTransferValveEndpoint, SetVctHandshakeEndpoint, ValveState, VctHandshake,
};
use poststation_sdk::PoststationClient;

use crate::logger::{LogMessage, send_log_message};

/// Holds the controller client for communication functions.
pub struct ControllerClient {
    serial: u64,
    client: PoststationClient,
    ctr: AtomicU32,
}

impl ControllerClient {
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

    pub async fn keep_alive(&self) {
        if self
            .client
            .proxy_endpoint::<GetUniqueIdEndpoint>(self.serial, self.ctr(), &())
            .await
            .is_err()
        {
            send_log_message(LogMessage::new_error(
                "Failed to send keep-alive to controller",
            ))
            .await;
        }
    }

    pub async fn baking(&self, baking_state: BakingState) {
        if self
            .client
            .proxy_endpoint::<icd::SetBakingEndpoint>(self.serial, self.ctr(), &baking_state)
            .await
            .is_err()
        {
            send_log_message(LogMessage::new_error(
                "Failed to send new baking state to controller",
            ))
            .await;
        }
    }

    pub async fn light(&self, light_state: LightState) {
        if self
            .client
            .proxy_endpoint::<SetLightEndpoint>(self.serial, self.ctr(), &light_state)
            .await
            .is_err()
        {
            send_log_message(LogMessage::new_error(
                "Failed to send new light state to controller",
            ))
            .await;
        }
    }

    pub async fn pump_valve(&self, valve_state: ValveState) {
        if self
            .client
            .proxy_endpoint::<SetPumpValveEndpoint>(self.serial, self.ctr(), &valve_state)
            .await
            .is_err()
        {
            send_log_message(LogMessage::new_error(
                "Failed to send new pump valve state to controller",
            ))
            .await;
        }
    }

    pub async fn transfer_valve(&self, valve_state: ValveState) {
        if self
            .client
            .proxy_endpoint::<SetTransferValveEndpoint>(self.serial, self.ctr(), &valve_state)
            .await
            .is_err()
        {
            send_log_message(LogMessage::new_error(
                "Failed to send new transfer valve state to controller",
            ))
            .await;
        }
    }

    pub async fn vct_handshake(&self, handshake: VctHandshake) {
        if self
            .client
            .proxy_endpoint::<SetVctHandshakeEndpoint>(self.serial, self.ctr(), &handshake)
            .await
            .is_err()
        {
            send_log_message(LogMessage::new_error(
                "Failed to send new VCT handshake to controller",
            ))
            .await;
        }
    }
}
