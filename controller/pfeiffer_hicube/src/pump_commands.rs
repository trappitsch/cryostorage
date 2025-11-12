//! Handles all the pump commands that we want to read, write.

use anyhow::{Result, bail};
use opcua::{
    client::MonitoredItem,
    types::{DataValue, Identifier, NodeId, NumericRange, UAString, Variant, WriteValue},
};

pub const VENT_STRING: &str = "P1_012_EnableVent";

/// The state of the venting valve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VentState {
    /// Venting is enabled.
    Enabled,
    /// Venting is disabled.
    Disabled,
}

/// The state of the pump stand overall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpStandState {
    /// The pumps are on.
    On,
    /// The pumps are off.
    Off,
}

/// An enum to represent the various variables that can be set/read from the pump stand.
///
/// Note: Not all of these can actually be set, some are read-only (i.e., pressure values).
/// This enum will be sent through channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variables {
    /// State of the pump stand.
    PumpStand(PumpStandState),
    /// State of the venting valve.
    Venting(VentState),
}

impl From<Variables> for WriteValue {
    fn from(value: Variables) -> Self {
        match value {
            Variables::PumpStand(state) => {
                todo!()
            }
            Variables::Venting(state) => {
                let node_id = NodeId::new(1, VENT_STRING);
                let attribute_id = 13;
                let index_range = NumericRange::default();
                let mut value = DataValue::null();
                value.value = Some(Variant::Boolean(matches!(state, VentState::Enabled)));
                WriteValue {
                    node_id,
                    attribute_id,
                    index_range,
                    value,
                }
            }
        }
    }
}

impl Variables {
    /// Convert a DataValue and MonitoredItem into a Variables enum.
    pub fn from_data_value_monitored_item(dv: &DataValue, item: &MonitoredItem) -> Result<Self> {
        // All the UAStrings we need
        let vent_ua = UAString::from(VENT_STRING);

        let node_id = &item.item_to_monitor().node_id;

        match node_id.identifier.clone() {
            Identifier::String(str_id) => {
                if str_id == vent_ua {
                    match dv.value {
                        Some(Variant::Boolean(b)) => {
                            if b {
                                Ok(Variables::Venting(VentState::Enabled))
                            } else {
                                Ok(Variables::Venting(VentState::Disabled))
                            }
                        }
                        _ => bail!("Unexpected variant type for venting state."),
                    }
                } else {
                    bail!("Unsupported string node ID for conversion to Variables enum.")
                }
            }
            _ => bail!("Unsupported node ID for conversion to Variables enum."),
        }
    }
}
