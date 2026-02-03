//! Module to poll the Lakeshore temperature controller.
//!
//! This module provides the interface for all the things we want to do with the Lakeshore
//! temperature controller from the GUI. These tasks are:
//! - Reading the temperatures of two channels and returning the channel names and temperatures.

use std::collections::HashMap;

use anyhow::{Result, bail};

use instrumentrs::Instrument;
use lakeshore_336::{Lakeshore336, SerialInterfaceLakeshore};

use measurements::Temperature;
use serde::{Deserialize, Serialize};
use serialport::SerialPort;

use crate::connections::SerialAdapter;

/// Cryostorage chamber's way of looking at a Lakeshore temperature controller.
pub struct LakeshoreTempInst {
    config: LakeshoreTempConfig,
    instrument: Option<Lakeshore336<Instrument<Box<dyn SerialPort>>>>,
}

impl LakeshoreTempInst {
    /// Create a new Lakeshore temperature controller instance.
    pub fn new(config: LakeshoreTempConfig) -> Self {
        Self {
            config,
            instrument: None,
        }
    }

    // Connect to the Lakeshore temperature controller and store the interface in self.
    fn connect(&mut self) -> Result<()> {
        let interface = SerialInterfaceLakeshore::simple(&self.config.adapter.get_address())?;
        let instrument = Lakeshore336::try_new(interface)?;
        self.instrument = Some(instrument);
        Ok(())
    }

    /// Read the temperatures and return them.
    ///
    /// We return a HashMap with the name of the channel as the key and the temperature as the
    /// value. 
    /// An error is returned if the we cannot read the temperatures for any reason.
    pub fn get_status_measurements(&mut self) -> Result<HashMap<String, Temperature>> {
        // Do we need to connect again?
        if self.instrument.is_none() {
            let _ = self.connect(); // if fails, instrument stays None
        }

        if let Some(inst) = self.instrument.as_mut() {
            let mut ret_map = HashMap::new();

            for (idx, name) in self.config.channel_iter().enumerate() {
                // for each populated channel, get the temperature
                if let Some(ch_name) = name {
                    let temp_k = inst
                        .get_channel(idx)?
                        .get_temperature()?;
                    ret_map.insert(ch_name.clone(), temp_k);
                }
            }

            return Ok(ret_map);
        }
        bail!("Lakeshore temperature controller not connected");
    }
}

/// Configuration to store for the Lakeshore temperature controller.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LakeshoreTempConfig {
    /// Adapter configuration.
    pub adapter: SerialAdapter,
    /// Channel 1/A name, or None if not present.
    pub channel_a_name: Option<String>,
    /// Channel 2/B name, or None if not present.
    pub channel_b_name: Option<String>,
    /// Channel 3/C name, or None if not present.
    pub channel_c_name: Option<String>,
    /// Channel 4/D name, or None if not present.
    pub channel_d_name: Option<String>,
}

impl LakeshoreTempConfig {
    /// Iterator over the four channels, yielding `&Option<String>`.
    pub fn channel_iter(&self) -> impl Iterator<Item = &Option<String>> {
        // Build a temporary slice of tuples; the slice lives only for the
        // duration of the call, so returning an `impl Iterator` is safe.
        [
            &self.channel_a_name,
            &self.channel_b_name,
            &self.channel_c_name,
            &self.channel_d_name,
        ]
        .into_iter()
    }
}

/// Some default values, according to the initial setup.
///
/// These can be changed in the configuration file, only used for writing the file the first time.
impl Default for LakeshoreTempConfig {
    fn default() -> Self {
        Self {
            adapter: SerialAdapter {
                port_name: String::from("/dev/ttyUSB0"),
            },
            channel_a_name: Some(String::from("Cooler")),
            channel_b_name: Some(String::from("Sample")),
            channel_c_name: None,
            channel_d_name: None,
        }
    }
}
