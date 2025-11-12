//! Subscribe to messages from the HiCubeClient and allow user to enable/disable venting via stdin.
//!
//! The values we listen to are coded into the `pfeiffer_hicube` crate.
//! Ideally, this would include all the things we potentially want to monitor.
//!
//! Writing: We only write venting state based on user input from stdin, however, 
//! the way this is implemented is also general and coded into the `pfeiffer_hicube` crate.
//! In principle, any variable we listen to should also be writable (assuming it's and RW
//! variable).

use std::net::IpAddr;

use pfeiffer_hicube::{HiCubeClient, Variables as HiCubeVariables, VentState};
use tokio::{
    io::{self as aio, AsyncBufReadExt},
    select,
    sync::oneshot,
};

#[tokio::main]
async fn main() {
    let ip_address = IpAddr::from([192, 168, 1, 100]);
    let port = 4840;

    let mut hicube = HiCubeClient::try_new_and_connect(ip_address, port)
        .await
        .expect("Failed to create HiCubeClient");

    // Subscribe to messages from the Client by getting a receiver with the values we subscribe to.
    let mut recv = hicube
        .subscribe()
        .await
        .expect("Failed to subscribe to HiCubeClient");

    // Break on Ctrl-C
    //
    // This is important for the demo in order to gracefully disconnect from the HiCubeClient.
    // Otherwise, the pump will at some point now allow any new connections until the previous ones
    // time out.
    let (tx, mut rx_ctrl_c) = oneshot::channel();
    let session_c = hicube.get_session();
    tokio::task::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to register CTRL-C handler: {e}");
            return;
        }
        let _ = session_c.disconnect().await;
        tx.send(()).unwrap();
    });

    // Let's spawn a task that reads from stdin and enables/disables venting
    //
    // This writes to the HiCubeClient to enable/disable venting based on user input.
    // Just a demo to show how the writing to the client can be done!
    tokio::spawn(async move {
        loop {
            println!("Type 'on' to enable venting, 'off' to disable it: ");

            // Prepare a mutable String to store the input.
            let mut input = String::new();

            let stdin = aio::stdin();
            let mut reader = aio::BufReader::new(stdin);

            reader
                .read_line(&mut input)
                .await
                .expect("Should be able to read a line.");
            match input.trim() {
                "on" => {
                    let to_send = HiCubeVariables::Venting(VentState::Enabled);
                    hicube
                        .write(to_send)
                        .await
                        .expect("Failed to write vent state");
                }
                "off" => {
                    let to_send = HiCubeVariables::Venting(VentState::Disabled);
                    hicube
                        .write(to_send)
                        .await
                        .expect("Failed to write vent state");
                }
                _ => {
                    println!("Unknown command: {}", input.trim());
                }
            }
        }
    });

    println!("Listening for new messages... until Ctrl-C is pressed.");
    // Listen for new messages, either an event we are subscribed to, or a Ctrl-C signal.
    loop {
        select! {
            // So we got a new message from the HiCubeClient, do something with it.
            Some(ms) = recv.recv() => {
                println!("Received message: {:?}", ms);
            }
            // We got a Ctrl-C signal, exit the loop.
            _ = &mut rx_ctrl_c => {
                println!("Disconnect signal received, exiting listener.");
                break;
            }
        }
    }
}
