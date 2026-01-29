#![cfg_attr(not(feature = "use-std"), no_std)]

mod baking;
mod flow_meter;
mod light;
mod valve;
mod vct;

use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

pub use baking::*;
pub use flow_meter::*;
pub use light::*;
pub use valve::*;
pub use vct::*;

use postcard_rpc::{TopicDirection, endpoints, topics};

pub const BROADCAST_INTERVAL_MS: u64 = 1000;

/// State of the entire instrument.
#[derive(Debug, Default, Serialize, Deserialize, Schema)]
pub struct InstrumentState {
    pub baking: BakingState,
    pub flow_meter: FlowMeterState,
    pub pump_valve: ValveState,
    pub transfer_valve: ValveState,
    pub vct: VctState,
}

// Endpoints spoken by our device
//
// GetUniqueIdEndpoint is mandatory, the others are examples
endpoints! {
    list = ENDPOINT_LIST;
    | EndpointTy                | RequestTy     | ResponseTy            | Path                          |
    | ----------                | ---------     | ----------            | ----                          |
    | GetUniqueIdEndpoint       | ()            | u64                   | "poststation/unique_id/get"   |
    | RebootToPicoBoot          | ()            | ()                    | "hw/picoboot/reset"           |
    | SetLightEndpoint          | LightState    | ()                    | "hw/light/set"                |
    | SetBakingEndpoint         | BakingState   | ()                    | "hw/baking/set"               |
    | SetPumpValveEndpoint      | ValveState    | ()                    | "hw/valve/pump/set"           |
    | SetTransferValveEndpoint  | ValveState    | ()                    | "hw/valve/transfer/set"       |
    | SetVctHandshakeEndpoint   | VctHandshake  | ()                    | "hw/vct_handshake/set"        |
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
    | TopicTy                   | MessageTy       | Path              | Cfg                           |
    | -------                   | ---------       | ----              | ---                           |
    | BcInstStatus              | InstrumentState | "bc/status"       |                               |
}
