//! This module handles communication with the controller firmware via poststation.
use std::sync::{atomic::{AtomicU32, Ordering}, mpsc};

use icd::{LightState, SetLightEndpoint};
use poststation_sdk::PoststationClient;

pub enum ControllerCommands {
    Light(LightState),
}

pub async fn controller_task(cntrl: Controller, rx: mpsc::Receiver<ControllerCommands>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            ControllerCommands::Light(state) => {
                cntrl.light(state).await;
            }
        }
    }
}

pub struct Controller {
    serial: u64,
    client: PoststationClient,
    ctr: AtomicU32,
}

impl Controller {
    pub fn new(client: PoststationClient, serial: u64) -> Self {
        Self { client, serial, ctr: AtomicU32::new(0) }
    }

    #[inline(always)]
    fn ctr(&self) -> u32 {
        self.ctr.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn light(&self, light_state: LightState) {
        let _ = self.client.proxy_endpoint::<SetLightEndpoint>(
            self.serial,
            self.ctr(),
            &light_state,
        ).await;  // FIXME: need error checking
        println!("Set light: {:?}", light_state);
    }
}
