#![cfg_attr(not(feature = "use-std"), no_std)]

use postcard_rpc::{TopicDirection, endpoints, topics};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Schema)]
pub struct SleepMillis {
    pub millis: u16,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
pub struct SleptMillis {
    pub millis: u16,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Schema)]
pub enum BakingState {
    /// For how long in seconds to turn the baking on / is it on already
    On { time_sec: u64 },
    #[default]
    Off,
}

#[derive(Default, Debug, Serialize, Deserialize, Schema)]
pub enum FlowMeterState {
    Ok,
    #[default]
    FlowError,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
pub enum LightState {
    Off,
    On,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Schema)]
pub enum ValveState {
    Open,
    Close,
    #[default]
    Undefined,
}

/// VCT handshake. Only if OK, the VCT will proceed with attaching.
#[derive(Debug, Serialize, Deserialize, Schema)]
pub enum VctHandshakeState {
    /// Enable the handshake so the VCT can attach
    Attach,
    /// Disable the handshake so the VCT cannot be attached
    Detach,
}

/// Provide states for the VCT attach and gate signals that are read.
#[derive(Clone, Default, Debug, Serialize, Deserialize, Schema)]
pub enum VctState {
    /// Signals that the attach state or gate state is connected (attached and open)
    Connected,
    /// Signals that the attach state or gate state is disconnected (detached and closed)
    #[default]
    Disconnected,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Schema)]
pub struct VctStates {
    pub attach: VctState,
    pub gate: VctState,
}

/// Overall status for the whole controller -> broadcasted topic
#[derive(Debug, Default, Serialize, Deserialize, Schema)]
pub struct CtrlStatus {
    pub baking: BakingState,
    pub flow_meter: FlowMeterState,
    pub pump_valve: ValveState,
    pub transfer_valve: ValveState,
    pub vct: VctStates,
}

// ---

// Endpoints spoken by our device
//
// GetUniqueIdEndpoint is mandatory, the others are examples
endpoints! {
    list = ENDPOINT_LIST;
    | EndpointTy                | RequestTy         | ResponseTy            | Path                          |
    | ----------                | ---------         | ----------            | ----                          |
    | GetUniqueIdEndpoint       | ()                | u64                   | "poststation/unique_id/get"   |
    | RebootToPicoBoot          | ()                | ()                    | "sys/picoboot/reset"          |
    | SleepEndpoint             | SleepMillis       | SleptMillis           | "sys/sleep"                   |
    | GetBakingEndpoint         | ()                | BakingState           | "hw/baking/get"               |
    | SetBakingEndpoint         | BakingState       | ()                    | "hw/baking/set"               |
    | GetLightEndpoint          | ()                | LightState            | "hw/light/get"                |
    | SetLightEndpoint          | LightState        | ()                    | "hw/light/set"                |
    | GetValvePumpEndpoint      | ()                | ValveState            | "hw/valve_pump/get"           |
    | SetValvePumpEndpoint      | ValveState        | ()                    | "hw/valve_pump/set"           |
    | GetValveTransferEndpoint  | ()                | ValveState            | "hw/valve_transfer/get"       |
    | SetValveTransferEndpoint  | ValveState        | ()                    | "hw/valve_transfer/set"       |
    | GetVctHandshake           | ()                | VctHandshakeState     | "hw/vct_handshake/get"        |
    | SetVctHandshake           | VctHandshakeState | ()                    | "hw/vct_handshake/set"        |
    | GetVctStatus              | ()                | VctStates             | "hw/vct_status/get"           |
}

// incoming topics handled by our device
topics! {
    list = TOPICS_IN_LIST;
    direction = TopicDirection::ToServer;
    | TopicTy                   | MessageTy     | Path              |
    | -------                   | ---------     | ----              |
}

// outgoing topics handled by our device
topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy                   | MessageTy     | Path              | Cfg                           |
    | -------                   | ---------     | ----              | ---                           |
    | BcCtrlStatus              | CtrlStatus    | "bc/ctrl_status"  |                               |
}
