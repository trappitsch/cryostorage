//! Module to control and poll the Omnicontrol vacuum gauge controller.
//!
//! This module interfaces with the Omnicontrol vacuum gauge controller and reads the pressures
//! from the configured channels. Furthermore, it can turn the channels on and off.
//!
//! Important: If channels are manually turned on or off, the program might not receive the
//! correct state of the channels from the Omnicontroller. Why is completely unclear, but likely
//! related to non-ideal firmware on the controller itself. Commands sent are correct and work as
//! expected when we set the channel status remotely! There's currently no clear solution to this,
//! especially as it is also unclear if the controller runs the latest firmware or not (no info
//! available as far as I could find). For now, we work around these issues by only dealing with
//! the statuses of the channels when we set them remotely.
//!
//! Note: If the status is turned off on the device, the pressure reads back something very low,
//! sometimes I get <1e-13 mbar. Since we will not get the pressure to <1e-10 mbar, we will use
//! <1e-11mbar as the cutoff for deciding if the channel is on or off.

use std::{
    collections::HashMap,
    fmt::Display,
    net::{SocketAddr, TcpStream},
};

use anyhow::Result;

use instrumentrs::{Instrument, TcpIpInterface};
use measurements::Pressure;
use pfeiffer_omnicontrol::{BaseAddress, Channel, Omnicontrol, SensorStatus};
use serde::{Deserialize, Serialize};

use crate::connections::{TCP_IP_TIMEOUT, TcpIpAdapter};

pub const OMNICONTROL_PRESSURE_CUTOFF_MBAR: f64 = 1e-11;

type GaugeChannel = Channel<Instrument<TcpStream>>;

/// Cryostorage chamber's way of looking at the Omnicontrol vacuum gauge controller
pub struct OmniControlInst {
    /// Configuration of the Omnicontrol, stored in config file.
    config: OmniControlConfig,
    /// The instrument interface of the Omnicontrol, which allows for re-inits
    instrument: Option<Omnicontrol<Instrument<TcpStream>>>,
    /// Channels for option 1 and option 2 of the Omnicontrol.
    channels: Option<(GaugeChannel, GaugeChannel)>,
}

impl OmniControlInst {
    /// Create a new Omnicontrol instance.
    pub fn new(config: OmniControlConfig) -> Self {
        Self {
            config,
            instrument: None,
            channels: None,
        }
    }

    /// Check the connection and if it's none, connect.
    fn check_connection(&mut self) -> Result<()> {
        if self.instrument.is_none() || self.channels.is_none() {
            self.connect()
        } else {
            Ok(())
        }
    }

    /// Connect to the Omnicontrol and store instrument in self.
    fn connect(&mut self) -> Result<()> {
        let addr = self.config.tcp_ip_adapter.get_address();
        let socket_addr: SocketAddr = addr.parse()?;

        let stream = TcpStream::connect_timeout(&socket_addr, TCP_IP_TIMEOUT)?;
        stream.set_write_timeout(Some(TCP_IP_TIMEOUT))?;
        stream.set_read_timeout(Some(TCP_IP_TIMEOUT))?;

        let interface = TcpIpInterface::full(stream)?;
        let mut instrument = Omnicontrol::new(interface, self.config.base_address);

        let ch_opt1 = instrument.get_channel(1)?;
        let ch_opt2 = instrument.get_channel(2)?;
        self.instrument = Some(instrument);
        self.channels = Some((ch_opt1, ch_opt2));

        Ok(())
    }

    /// Get the pressures for both channels.
    ///
    /// Returns a hashmap with the two channels as keys and the `Option<Pressure>` as values. If
    /// this option is `None`, the read value is below the cutoff threshold we define and thus we
    /// determine the channel to be off (see module docstring for details).
    pub fn get_pressures(&mut self) -> Result<HashMap<Gauge, Option<Pressure>>> {
        self.check_connection()?;

        let mut ret_map = HashMap::new();
        if let Some((ch_opt1, ch_opt2)) = &mut self.channels {
            let press_opt1 = ch_opt1.get_pressure()?;
            let press_opt2 = ch_opt2.get_pressure()?;

            // If the pressure value is below the cutoff, return `None` to represent off state.
            let press_value_opt1 = if press_opt1.as_millibars() < OMNICONTROL_PRESSURE_CUTOFF_MBAR {
                None
            } else {
                Some(press_opt1)
            };
            ret_map.insert(self.config.gauge_option1, press_value_opt1);
            let press_value_opt2 = if press_opt2.as_millibars() < OMNICONTROL_PRESSURE_CUTOFF_MBAR {
                None
            } else {
                Some(press_opt2)
            };
            ret_map.insert(self.config.gauge_option2, press_value_opt2);
        };

        Ok(ret_map)
    }

    /// Set a channel to on or off.
    ///
    /// Detection of this does not necessarily work when the channel is turned off manually, so a
    /// getter for this status is not implemented but is rather processed from the pressure
    /// reading.
    pub fn set_status(&mut self, gauge: Gauge, status: GaugeStatus) -> Result<()> {
        self.check_connection()?;

        if let Some((ch_opt1, ch_opt2)) = &mut self.channels {
            // detect the correct channel
            let mut ch = if self.config.gauge_option1 == gauge {
                ch_opt1
            } else {
                ch_opt2
            };

            ch.set_status(status.into())?;
        };

        Ok(())
    }

    /// Reset the instrument to requrie a reconnect on next access.
    pub fn reset_instrument(&mut self) {
        self.instrument = None;
        self.channels = None;
    }
}

/// Configuration to store for the Omnicontrol.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OmniControlConfig {
    /// Connection adapter
    pub tcp_ip_adapter: TcpIpAdapter,
    /// Base address of the Omnicontrol RS-485 instrument.
    pub base_address: BaseAddress,
    /// Gauge that is connected to option 1 of the Omnicontrol.
    pub gauge_option1: Gauge,
    /// Gauge that is connected to option 2 of the Omnicontrol.
    pub gauge_option2: Gauge,
}

impl Default for OmniControlConfig {
    fn default() -> Self {
        Self {
            tcp_ip_adapter: TcpIpAdapter::new_from_str("192.168.1.2:4002"),
            base_address: BaseAddress::Zero,
            gauge_option1: Gauge::Transfer,
            gauge_option2: Gauge::Chamber,
        }
    }
}

/// Enum to hold the gauge name, either "Chamber" or "Transfer".
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gauge {
    /// Vacuum gauge connected to the sample chamber.
    Chamber,
    /// Vacuum gauge connected to the sample transfer line.
    Transfer,
}

impl Display for Gauge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gauge::Chamber => write!(f, "Chamber"),
            Gauge::Transfer => write!(f, "Transfer"),
        }
    }
}

/// Status of a vacuum gauge.
#[derive(Debug, Default, Copy, Clone)]
pub enum GaugeStatus {
    /// Gauge is off.
    #[default]
    Off,
    /// Gauge is on.
    On,
}

impl Display for GaugeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GaugeStatus::Off => write!(f, "Off"),
            GaugeStatus::On => write!(f, "On"),
        }
    }
}

impl From<bool> for GaugeStatus {
    fn from(is_on: bool) -> Self {
        match is_on {
            true => GaugeStatus::On,
            false => GaugeStatus::Off,
        }
    }
}

impl From<GaugeStatus> for bool {
    fn from(status: GaugeStatus) -> Self {
        match status {
            GaugeStatus::On => true,
            GaugeStatus::Off => false,
        }
    }
}

impl From<GaugeStatus> for SensorStatus {
    fn from(status: GaugeStatus) -> Self {
        match status {
            GaugeStatus::On => SensorStatus::On,
            GaugeStatus::Off => SensorStatus::Off,
        }
    }
}
