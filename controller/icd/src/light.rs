//! All messages regarding the light output.

use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

/// State of the light output.
///
/// The light output is simply a 5V output that can be turned on or off.
/// By default, the light should be set to off.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Schema)]
pub enum LightState {
    On,
    #[default]
    Off,
}

impl core::ops::Not for LightState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            LightState::On => LightState::Off,
            LightState::Off => LightState::On,
        }
    }
}
