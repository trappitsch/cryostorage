//! Handles all the pump commands that we want to read, write.

use anyhow::{Result, bail};
use lazy_static::lazy_static;
use opcua::{
    client::MonitoredItem,
    types::{DataValue, Identifier, NodeId, NumericRange, UAString, Variant, WriteValue},
};

lazy_static! {
    /// The overall pump stand status
    ///
    /// NodeId: ns=1;s=SYS_STATUS
    /// Value: float, 1.0 for on, 4.0 for off.
    /// Can be turned on by setting 1.0, but cannot be turned off from here.
    /// To turn off, first turn off turbo pump, then roughing pump.
    pub static ref PUMP_STAND_STRING: UAString = UAString::from("SYS_STATUS");
}

lazy_static! {
    /// Status of the roughing pump
    ///
    /// NodeId: ns=1;s=P2_010_PumpgStatn
    /// Bool
    pub static ref ROUGHING_PUMP_STRING: UAString = UAString::from("P2_010_PumpgStatn");
}

lazy_static! {
    /// Status of the turbomolecular pump
    ///
    /// NodeId: ns=1;s=P1_010_PumpgStatn
    /// Bool
    pub static ref TURBO_PUMP_STRING: UAString = UAString::from("P1_010_PumpgStatn");
}

lazy_static! {
    /// Vent string
    ///
    /// NodeId: ns=1;s=P1_012_EnableVent
    /// Bool
    pub static ref VENT_STRING: UAString = UAString::from("P1_012_EnableVent");
}

/// The state of the venting valve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VentState {
    /// Venting is enabled.
    Enabled,
    /// Venting is disabled.
    Disabled,
}

/// The state of the pump stand overall.
///
/// This is the state of the overall pump state, set as a float
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpStandState {
    /// The pumps are off (float val 0.0).
    Off,
    /// The pumps are on (float val 1.0).
    On,
    /// Spinning down state (float val 4.0).
    SpinningDown,
    /// Spinning up state (float val 5.0).
    SpinningUp,
    /// Other states, that we don't know about.
    Other,
}

/// State of an individual pump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpState {
    /// The pump is on.
    On,
    /// The pump is off.
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
    /// Roughing pump state.
    RoughingPump(PumpState),
    /// Turbo pump state.
    TurboPump(PumpState),
    /// State of the venting valve.
    Venting(VentState),
}

impl From<Variables> for WriteValue {
    fn from(value: Variables) -> Self {
        match value {
            Variables::PumpStand(state) => {
                let node_id = NodeId::new(1, PUMP_STAND_STRING.as_ref());
                let attribute_id = 13;
                let index_range = NumericRange::default();
                let mut value = DataValue::null();
                let float_val = match state {
                    PumpStandState::Off => 0.0,
                    PumpStandState::On => 1.0,
                    PumpStandState::SpinningDown => 4.0,
                    PumpStandState::SpinningUp => 5.0,
                    PumpStandState::Other => panic!("Cannot write _Other state to pump stand."),
                };
                value.value = Some(Variant::Float(float_val));
                WriteValue {
                    node_id,
                    attribute_id,
                    index_range,
                    value,
                }
            }
            Variables::RoughingPump(state) => {
                let node_id = NodeId::new(1, ROUGHING_PUMP_STRING.as_ref());
                let attribute_id = 13;
                let index_range = NumericRange::default();
                let mut value = DataValue::null();
                value.value = Some(Variant::Boolean(matches!(state, PumpState::On)));
                WriteValue {
                    node_id,
                    attribute_id,
                    index_range,
                    value,
                }
            }
            Variables::TurboPump(state) => {
                let node_id = NodeId::new(1, TURBO_PUMP_STRING.as_ref());
                let attribute_id = 13;
                let index_range = NumericRange::default();
                let mut value = DataValue::null();
                value.value = Some(Variant::Boolean(matches!(state, PumpState::On)));
                WriteValue {
                    node_id,
                    attribute_id,
                    index_range,
                    value,
                }
            }
            Variables::Venting(state) => {
                let node_id = NodeId::new(1, VENT_STRING.as_ref());
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

        let node_id = &item.item_to_monitor().node_id;

        match &node_id.identifier {
            Identifier::String(s) if s == &*PUMP_STAND_STRING => match dv.value {
                Some(Variant::Float(f)) => match f {
                    0.0 => Ok(Variables::PumpStand(PumpStandState::Off)),
                    1.0 => Ok(Variables::PumpStand(PumpStandState::On)),
                    4.0 => Ok(Variables::PumpStand(PumpStandState::SpinningDown)),
                    5.0 => Ok(Variables::PumpStand(PumpStandState::SpinningUp)),
                    _ => Ok(Variables::PumpStand(PumpStandState::Other)),
                },
                _ => bail!("Unexpected variant type for turbo pump state."),
            },
            Identifier::String(s) if s == &*ROUGHING_PUMP_STRING => match dv.value {
                Some(Variant::Boolean(b)) if b => Ok(Variables::RoughingPump(PumpState::On)),
                Some(Variant::Boolean(b)) if !b => Ok(Variables::RoughingPump(PumpState::Off)),
                _ => bail!("Unexpected variant type for roughing pump state."),
            },
            Identifier::String(s) if s == &*TURBO_PUMP_STRING => match dv.value {
                Some(Variant::Boolean(b)) if b => Ok(Variables::TurboPump(PumpState::On)),
                Some(Variant::Boolean(b)) if !b => Ok(Variables::TurboPump(PumpState::Off)),
                _ => bail!("Unexpected variant type for turbo pump state."),
            },
            Identifier::String(s) if s == &*VENT_STRING => match dv.value {
                Some(Variant::Boolean(b)) if b => Ok(Variables::Venting(VentState::Enabled)),
                Some(Variant::Boolean(b)) if !b => Ok(Variables::Venting(VentState::Disabled)),
                _ => bail!("Unexpected variant type for venting state."),
            },
            _ => bail!("Unsupported node ID for conversion to Variables enum."),
        }
    }
}
