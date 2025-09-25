use std::sync::{Arc, Mutex, mpsc};

use poststation_sdk::connect;

use crate::controller::{Controller, controller_task};

mod app;
mod controller;
mod prg_config;
mod samples;

#[tokio::main]
async fn main() {
    // config
    let conf = Arc::new(Mutex::new(prg_config::PrgConfig::try_new().unwrap()));
    println!("Loaded config: {:?}", conf);

    // comms
    let (tx, rx) = mpsc::channel();

    // controller
    let controller_config = conf
        .lock()
        .expect("Locking config must work")
        .get_controller_config();
    let client = connect(controller_config.address).await.unwrap();
    let cntrl = Controller::new(client, controller_config.serial);

    tokio::spawn(controller_task(cntrl, rx));

    match app::app_main(tx.clone(), Arc::clone(&conf)) {
        Ok(_) => println!("App exited normally"),
        Err(e) => eprintln!("App exited with error: {}", e),
    }
}
