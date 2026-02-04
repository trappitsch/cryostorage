//! Instruments module.
//!
//! This module handles communication with all peripherals. These are:
//! - Pfeiffer HiCube turbomolecular pump.
//! - Pfeiffer Vacuum gauge controller (two gauges, transfer and main chamber).
//! - Lakeshore temperature controller, to measure two different temperatures.
//! - Cryocooler
//!
//! This file contains two parts:
//! - A monitoring task that needs to run in its own thread to poll instruments periodically.
//! - An async executor to send commands to instruments and change their state.
//!
//! The monitoring tasks will need to run in its own thread as the instruments are polled
//! frequently and blocking calls are needed. However, we don't want to block the entire program
//! regularly.
//!
//! The command task on the other hand can run as an async task as commands will be sent very
//! infrequently, and we will simply accept the fact that it may take half a second to set
//! something. This is acceptable for our use case. Worse case scenario will be that the interface
//! freezes until a timeout is hit.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use measurements::{Power, Temperature};
use sunpower_cryotelgt::CoolerState;
use tokio::sync::mpsc;

use crate::instruments::cryocooler::CryoCoolerInst;
use crate::instruments::lakeshore_temp::LakeshoreTempInst;
use crate::logger::{LogMessage, send_log_message, send_log_message_now};
use crate::prg_config::PrgConfig;
use crate::status::InstrumentStatus;

pub mod cryocooler;
pub mod lakeshore_temp;

const POLLING_INTERVAL: Duration = Duration::from_secs(5);

/// Commands that can be sent to instruments.
pub enum InstrumentCommands {
    /// Set temperature of the cryocooler.
    CryoCoolerSetpoint(Temperature),
    CryoCoolerState(CoolerState),
}

/// Monitoring task that polls the instruments periodically.
pub async fn instruments_task(
    prg_conf: Arc<Mutex<PrgConfig>>,
    inst_status: Arc<Mutex<InstrumentStatus>>,
    mut rx_instr: mpsc::Receiver<InstrumentCommands>,
) {
    // Get all the instruments
    let mut lakeshore_temp_inst = {
        let conf = prg_conf
            .lock()
            .expect("PrgConfig lock poisoned")
            .get_lakeshore_temp_config();
        LakeshoreTempInst::new(conf)
    };

    let mut cryocooler_inst = {
        let conf = prg_conf
            .lock()
            .expect("PrgConfig lock poisoned")
            .get_cryocooler_config();
        CryoCoolerInst::new(conf)
    };

    // Get shutdown receiver
    let mut rx_shutdown = crate::HALT_SENDER.get().unwrap().subscribe();

    // Wait for UI to be set in InstrumentStatus
    while !inst_status
        .lock()
        .expect("InstrumentStatus lock poisoned")
        .get_ui_is_set()
    {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Set the instrument initial states
    // cryocooler
    let cooler_setpoint_temperature = match cryocooler_inst.get_setpoint_temperature() {
        Ok(t) => t,
        Err(e) => {
            send_log_message(LogMessage::new_error(&format!(
                "Failed to set initial cryocooler setpoint temperature: {}",
                e,
            )))
            .await;
            cryocooler_inst.reset_instrument();
            Temperature::default()
        }
    };

    let cooler_state = match cryocooler_inst.get_state() {
        Ok(s) => s,
        Err(e) => {
            send_log_message(LogMessage::new_error(&format!(
                "Failed to set initial cryocooler state: {}",
                e,
            )))
            .await;
            cryocooler_inst.reset_instrument();
            CoolerState::Disabled
        }
    };

    // set all initial statuses of various instruments
    {
        let mut inst_status = inst_status.lock().expect("InstrumentStatus lock poisoned");

        if inst_status.set_temperature_setpoint_and_ui(cooler_setpoint_temperature).is_err() {
            send_log_message_now(LogMessage::new_error(
                "Failed to set initial cryocooler setpoint temperature in UI",
            ));
        }

        if inst_status.set_cooler_state_and_ui(cooler_state).is_err() {
            send_log_message_now(LogMessage::new_error(
                "Failed to set initial cryocooler state in UI",
            ));
        }
    } // drop lock

    let mut polling_interval = Duration::from_millis(1);

    loop {
        tokio::select! {
            // Loop that polls all instruments
            _ = tokio::time::sleep(polling_interval) => {
                // Temperatures
                let mut temperatures = match lakeshore_temp_inst.get_status_measurements() {
                    Ok(temps) => temps,
                    Err(e) => {
                        send_log_message( LogMessage::new_error(
                            &format!("Failed to read temperatures from Lakeshore336: {}", e)
                        )).await;
                        lakeshore_temp_inst.reset_instrument();
                        HashMap::new()
                    }
                };

                let temperature_cooler = match cryocooler_inst.get_status_measurement() {
                    Ok(temps) => temps,
                    Err(e) => {
                        send_log_message( LogMessage::new_error(
                            &format!("Failed to read temperature from Cryocooler: {}", e)
                        )).await;
                        cryocooler_inst.reset_instrument();
                        HashMap::new()
                    }
                };

                temperatures.extend(temperature_cooler);

                inst_status
                    .lock()
                    .expect("InstrumentStatus lock poisoned")
                    .set_temperatures(
                        *temperatures.get("Bridge").unwrap_or(&Temperature::default()),
                        *temperatures.get("Cooler").unwrap_or(&Temperature::default()),
                        *temperatures.get("Sample").unwrap_or(&Temperature::default())
                    );

                // Cryocooler current power
                let current_power = match cryocooler_inst.get_current_power() {
                    Ok(p) => p,
                    Err(e) => {
                        send_log_message( LogMessage::new_error(
                            &format!("Failed to read current power from Cryocooler: {}", e)
                        )).await;
                        cryocooler_inst.reset_instrument();
                        Power::default()
                    }
                };
                inst_status.lock().expect("InstrumentStatus lock poisoned")
                    .set_power_cooler_current(current_power);

                // Update UI
                inst_status.lock().expect("InstrumentStatus lock poisoned")
                    .update_ui_instrument_status();

                // after init, set polling interval to normal value
                if polling_interval != POLLING_INTERVAL {
                    polling_interval = POLLING_INTERVAL;
                }
            }
            // Instrument command received
            Some(cmd) = rx_instr.recv() => {
                match cmd {
                    InstrumentCommands::CryoCoolerSetpoint(temperature) => {
                        match cryocooler_inst.set_setpoint_temperature(temperature) {
                            Ok(_) => {
                                inst_status.lock().expect("InstrumentStatus lock poisoned")
                                    .set_temperature_setpoint_and_ui(temperature)
                                    .expect("UI set before this loop started.");
                            },
                            Err(e) => {
                                send_log_message(LogMessage::new_error(
                                    &format!(
                                        "Failed to set cryocooler setpoint temperature: {}", e)
                                    )
                                ).await;
                            }
                        }
                    }
                    InstrumentCommands::CryoCoolerState(state) => {
                        match cryocooler_inst.set_state(state.clone()) {
                            Ok(_) => {
                                inst_status.lock().expect("InstrumentStatus lock poisoned")
                                    .set_cooler_state_and_ui(state)
                                    .expect("UI set before this loop started.");
                            },
                            Err(e) => {
                                send_log_message(LogMessage::new_error(
                                    &format!(
                                        "Failed to set cryocooler state: {}", e)
                                    )
                                ).await;
                            }
                        }
                    }
                    // next match here
                }
            }
            // Shutdown signal received
            _ = rx_shutdown.recv() => {
                break;
            }
        }
    }
}

/// Get a clone of the instrument command sender.
fn get_instr_cmd_sender() -> mpsc::Sender<InstrumentCommands> {
    crate::INSTRUMENT_COMMAND_SENDER
        .get()
        .expect("Uninitialized")
        .clone()
}

/// Convenience function to await sending an instrument command.
///
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub async fn send_instr_cmd(cmd: InstrumentCommands) {
    let sender = get_instr_cmd_sender();
    if let Err(e) = sender.send(cmd).await {
        send_log_message_now(LogMessage::new_error(&format!(
            "Failed to send instrument command: {}",
            e
        )));
    }
}

/// Convenience function to send an instrument command without awaiting.
///
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub fn send_instr_cmd_now(cmd: InstrumentCommands) {
    let sender = get_instr_cmd_sender();
    if let Err(e) = sender.try_send(cmd) {
        send_log_message_now(LogMessage::new_error(&format!(
            "Failed to send instrument command now: {}",
            e
        )));
    }
}
