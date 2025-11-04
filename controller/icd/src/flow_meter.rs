//! Status of the flow meter

use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

/// State of the flow meter.
///
/// This represents how the water flow through the cryocooler is behaving.
#[derive(Debug, Default, Serialize, Deserialize, Schema)]
pub enum FlowMeterState {
    /// All is good.
    Ok,
    /// Flow is too low or off.
    #[default]
    FlowError,
}
