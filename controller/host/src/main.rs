use std::sync::mpsc;

use poststation_sdk::connect;

use crate::controller::{controller_task, Controller};


mod app;
mod controller;


#[tokio::main]
async fn main() {
    // comms
    let (tx, rx) = mpsc::channel();
    let serial = 0xE6137B4C98CE7746;
    let client = connect("localhost:51837").await.unwrap();
    let cntrl = Controller::new(client, serial);

    tokio::spawn(controller_task(cntrl, rx));

    match app::app_main(tx.clone()) {
        Ok(_) => println!("App exited normally"),
        Err(e) => eprintln!("App exited with error: {}", e),
    }
}

