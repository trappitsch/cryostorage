//! Module to handle instrument utilities that don't fit anywhere else.

use serde::{Deserialize, Serialize};

/// The name of the thermocouple.
///
/// We only have three thermocouples that can be connected in various configs.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThermocoupleChannelName {
    /// The thermocouple connected to the samples.
    Sample,
    /// The thermocouple connected to the bridge/shield above the sample.
    Bridge,
    /// The thermocouple connected to the cryocooler head.
    Cooler,
}
