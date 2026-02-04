//! Module to interact with the Sunpower CryoTel GT cryocooler
//!
//! This module provides the interface for all the things we want to do with the cryocooler.
//! These tasks are:
//! - Manage connection to the cryocooler over TCP/IP (Moxa) and auto-reconnect if needed.
//! - Read temperature of the cold head.

use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
};

use anyhow::{Result, anyhow, bail};
use instrumentrs::{Instrument, TcpIpInterface};
use measurements::{Power, Temperature};
use serde::{Deserialize, Serialize};
use sunpower_cryotelgt::CryoTelGt;

use crate::connections::{TCP_IP_TIMEOUT, TcpIpAdapter};

pub struct CryoCoolerInst {
    /// Configuration of the cryocooler.
    config: CryoCoolerConfig,
    /// Instrument connection, if connected.
    instrument: Option<CryoTelGt<Instrument<TcpStream>>>,
}

impl CryoCoolerInst {
    /// Create a new Cryocooler instance.
    pub fn new(config: CryoCoolerConfig) -> Self {
        Self {
            config,
            instrument: None,
        }
    }

    /// Check the connection and if it's none, connect.
    fn check_connection(&mut self) -> Result<()> {
        if self.instrument.is_none() {
            self.connect()
        } else {
            Ok(())
        }
    }

    /// Connect to the cryocooler and store instrument in self.
    fn connect(&mut self) -> Result<()> {
        let addr = self.config.tcp_ip_adapter.get_address();
        let socket_addr: SocketAddr = addr.parse()?;

        let stream = TcpStream::connect_timeout(&socket_addr, TCP_IP_TIMEOUT)?;
        stream.set_write_timeout(Some(TCP_IP_TIMEOUT))?;
        stream.set_read_timeout(Some(TCP_IP_TIMEOUT))?;

        let interface = TcpIpInterface::full(stream)?;
        let instrument = CryoTelGt::try_new(interface)?;
        self.instrument = Some(instrument);
        Ok(())
    }

    /// Get the current power of the cryocooler cold head.
    pub fn get_current_power(&mut self) -> Result<Power> {
        self.check_connection()?;

        if let Some(inst) = &mut self.instrument {
            let power = inst.get_power()?;
            return Ok(power);
        }

        bail!("Cryocooler not connected (should be unreachable)");
    }

    /// Get the set temperature of the cryocooler cold head.
    pub fn get_set_temperature(&mut self) -> Result<Temperature> {
        self.check_connection()?;

        if let Some(inst) = &mut self.instrument {
            let set_temp = inst.get_temperature_setpoint()?;
            return Ok(set_temp);
        }

        bail!("Cryocooler not connected (should be unreachable)");
    }

    /// Get the name of the temperature probe connected to the cryocooler and its temperature.
    ///
    /// We return a HashMap, as for the Lakeshore temperature controller, with the name as the key
    /// and the temperature as the value.
    /// An error is returned if we cannot read the temperature for any reason, an error is
    /// returned.
    pub fn get_status_measurement(&mut self) -> Result<HashMap<String, Temperature>> {
        self.check_connection()?;

        if let Some(inst) = &mut self.instrument {
            let channel_name = self
                .config
                .channel_name
                .as_ref()
                .ok_or_else(|| anyhow!("Channel name for cryocooler not set"))?
                .clone();

            let temperature = inst.get_temperature()?;
            return Ok(HashMap::from([(channel_name, temperature)]));
        }

        bail!("Cryocooler not connected (should be unreachable)");
    }

    /// Reset the instrument to None, so that it reconnects on next use.
    pub fn reset_instrument(&mut self) {
        self.instrument = None;
    }
}

/// Configuration of the Cryocooler to be stored in program config.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CryoCoolerConfig {
    /// The TCP/IP adapter of the cryocooler (connected via Moxa).
    pub tcp_ip_adapter: TcpIpAdapter,
    /// Name of the temperature channel that is connected to the cold head.
    channel_name: Option<String>,
}

impl Default for CryoCoolerConfig {
    fn default() -> Self {
        Self {
            tcp_ip_adapter: TcpIpAdapter::new_from_str("192.168.1.2:4003"),
            channel_name: Some("Bridge".to_string()),
        }
    }
}
