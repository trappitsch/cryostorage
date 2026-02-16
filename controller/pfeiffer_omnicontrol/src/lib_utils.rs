//! Library utilities, re-exported in `lib.rs`.
//!
//! Helps to keep the main file cleaner.

use std::fmt::{Display, Formatter};

use instrumentrs::InstrumentError;

/// RS-485 base address of the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseAddress {
    /// Address 0
    Zero = 0,
    /// Address 100
    OneHundred = 1,
    /// Address 200
    TwoHundred = 2,
}

impl Display for BaseAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseAddress::Zero => write!(f, "0"),
    BaseAddress::OneHundred => write!(f, "100"),
            BaseAddress::TwoHundred => write!(f, "200"),
        }
    }
}

impl TryFrom<usize> for BaseAddress {
    type Error = InstrumentError;

    fn try_from(value: usize) -> Result<Self, InstrumentError> {
        let res = match value {
            0 => BaseAddress::Zero,
            1 => BaseAddress::OneHundred,
            100 => BaseAddress::OneHundred,
            2 => BaseAddress::TwoHundred,
            200 => BaseAddress::TwoHundred,
            _ => { 
                return Err(InstrumentError::InvalidArgument(
                    "Base address must be one of 0, 100, or 200".to_string(),
                ));
                
            }
        };
        Ok(res)
    }
}

impl From<BaseAddress> for usize {
    fn from(addr: BaseAddress) -> Self {
        match addr {
            BaseAddress::Zero => 0,
            BaseAddress::OneHundred => 100,
            BaseAddress::TwoHundred => 200,
        }
    }
}

/// Status of an Omicontrol sensor.
///
/// This enum represents if a given sensor is active (On) or inactive (Off).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStatus {
    /// Sensor turned on.
    On,
    /// Sensor turned off.
    Off,
}

impl SensorStatus {
    /// Get the string representation of the sensor status, as expected by instrument.
    pub fn as_str(&self) -> &str {
        match self {
            SensorStatus::Off => "000",
            SensorStatus::On => "001",
        }
    }
}

impl TryFrom<usize> for SensorStatus {
    type Error = InstrumentError;

    fn try_from(value: usize) -> Result<Self, InstrumentError> {
        let res = match value {
            0 => SensorStatus::Off,
            1 => SensorStatus::On,
            _ => { 
                return Err(InstrumentError::InvalidArgument(
                    "Sensor status must be either 0 (Off) or 1 (On)".to_string(),
                ));
                
            }
        };
        Ok(res)
    }
}

impl From<SensorStatus> for usize {
    fn from(status: SensorStatus) -> Self {
        match status {
            SensorStatus::Off => 0,
            SensorStatus::On => 1,
        }
    }
}

