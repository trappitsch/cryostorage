use std::{error::Error, sync::{atomic::{AtomicU32, Ordering}, mpsc}};

use poststation_sdk::{connect, PoststationClient};

use icd::{LightState, SetLightEndpoint};

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // comms
    let (tx, rx) = mpsc::channel();
    let serial = 0xE6137B4C98CE7746;
    let client = connect("localhost:51837").await.unwrap();
    let cntrl = Controller::new(client, serial);

    tokio::spawn(controller_task(cntrl, rx));

    // slint
    let ui = AppWindow::new()?;
    ui.show()?;

    let tx_cl = tx.clone();
    ui.global::<HomeLogic>().on_light_switch({
        move |val| {
            let light_stat = match val {
                true => LightState::On,
                false => LightState::Off,
            };
            tx_cl.send(ControllerCommands::Light(light_stat)).unwrap();
        }
    });

    ui.run()?;
    Ok(())
}

pub enum ControllerCommands {
    Light(LightState),
}

async fn controller_task(cntrl: Controller, rx: mpsc::Receiver<ControllerCommands>) {
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
        self.client.proxy_endpoint::<SetLightEndpoint>(
            self.serial,
            self.ctr(),
            &light_state,
        ).await.unwrap();
    }
}
