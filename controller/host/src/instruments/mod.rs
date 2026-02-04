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

use measurements::Temperature;
use slint::Weak;

use crate::app::AppWindow;
use crate::instruments::lakeshore_temp::LakeshoreTempInst;
use crate::logger::{LogMessage, send_log_message};
use crate::prg_config::PrgConfig;
use crate::status::InstrumentStatus;

pub mod lakeshore_temp;

const POLLING_INTERVAL_SECS: u64 = 10;

/// Monitoring task that polls the instruments periodically.
pub async fn instruments_task(
    prg_conf: Arc<Mutex<PrgConfig>>,
    inst_status: Arc<Mutex<InstrumentStatus>>,
) {
    // Get all the instruments
    let mut lakeshore_temp_inst = {
        let conf = prg_conf
            .lock()
            .expect("PrgConfig lock poisoned")
            .get_lakeshore_temp_config();
        LakeshoreTempInst::new(conf)
    };

    // Get shutdown receiver
    let mut rx_shutdown = crate::HALT_SENDER.get().unwrap().subscribe();

    loop {
        tokio::select! {
            // Loop that polls all instruments
            _ = tokio::time::sleep(Duration::from_secs(POLLING_INTERVAL_SECS)) => {
                // Temperatures
                let temperatures = match lakeshore_temp_inst.get_status_measurements() {
                    Ok(temps) => temps,
                    Err(e) => {
                        send_log_message( LogMessage::new_error(
                            &format!("Failed to read temperatures from Lakeshore336: {}", e)
                        )).await;
                        lakeshore_temp_inst.reset_instrument();
                        HashMap::new()
                    }
                };
                // TODO: get temperature from cryocooler thermocouple and add to temperatures map
                inst_status
                    .lock()
                    .expect("InstrumentStatus lock poisoned")
                    .set_temperatures(
                        *temperatures.get("Bridge").unwrap_or(&Temperature::default()),
                        *temperatures.get("Cooler").unwrap_or(&Temperature::default()),
                        *temperatures.get("Sample").unwrap_or(&Temperature::default())
                    )
            }
            // Shutdown signal received
            _ = rx_shutdown.recv() => {
                break;
            }
        }
    }
}

/// Update the UI with the latest instrument status.
///
/// This is called
fn update_ui_instrument_status(ui: Weak<AppWindow>) {}
