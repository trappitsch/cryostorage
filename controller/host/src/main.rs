use std::sync::{Arc, Mutex};

use poststation_sdk::connect;
use tokio::sync::{broadcast, mpsc, OnceCell};

use crate::{controller::{controller_broadcast_listener, controller_task, Controller}, status::InstrumentStatus};

mod app;
mod controller;
mod prg_config;
mod samples;
mod status;

pub static HALT_SENDER: OnceCell<broadcast::Sender<()>> = OnceCell::const_new();

#[tokio::main]
async fn main() {
    // config
    let conf = Arc::new(Mutex::new(prg_config::PrgConfig::try_new().unwrap()));

    // status of instrument
    let inst_status = Arc::new(Mutex::new(InstrumentStatus::new()));

    // Shutdown signal and receiver for tasks
    let (tx_halt, _) = broadcast::channel(1);
    HALT_SENDER.set(tx_halt.clone()).expect("Uninitialized");

    // comms for controller task 
    let (tx_ctrl, rx_ctrl) = mpsc::channel(32);

    // controller
    let controller_config = conf
        .lock()
        .expect("Locking config must work")
        .get_controller_config();
    let client = connect(controller_config.address).await.unwrap();
    let cntrl = Controller::new(client.clone(), controller_config.serial);

    let cntrl_tsk = tokio::spawn(controller_task(cntrl, rx_ctrl));

    let controller_config = conf.lock().expect("Poisoned").get_controller_config();
    let cntrl_bc_listen = tokio::spawn(controller_broadcast_listener(
        client,
        controller_config.serial,
        Arc::clone(&inst_status),
    ));

    match app::app_main(tx_ctrl.clone(), Arc::clone(&conf), Arc::clone(&inst_status)) {
        Ok(_) => {
            tx_halt.send(()).unwrap();
            let _ = tokio::join!(cntrl_tsk, cntrl_bc_listen);
            println!("App exited normally");
        }
        Err(e) => eprintln!("App exited with error: {}", e),
    }
}
