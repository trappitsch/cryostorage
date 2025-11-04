//! Baking messages.

use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

/// State of the baking system.
///
/// This enum is used to send and receive baking state messages.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Schema)]
pub enum BakingState {
    On {
        time_sec: u64,
    },
    #[default]
    Off,
}

impl BakingState {
    /// Get the residual baking time in seconds.
    ///
    /// This tries to cast to a u32 to set to the watchdog timer. If this fails, it returns 0.
    pub fn get_time_sec_u32(&self) -> u32 {
        match self {
            BakingState::On { time_sec } => (*time_sec).try_into().unwrap_or(0),
            BakingState::Off => 0,
        }
    }
}
