#![cfg_attr(not(feature = "use-std"), no_std)]

use postcard_rpc::{endpoints, topics, TopicDirection};
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Schema)]
pub enum BakingState {
    /// For how long in seconds to turn the baking on / is it on already
    On { time_sec: u64 },
    Off,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
pub enum LightState {
    Off,
    On,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
pub enum ValveState {
    Open,
    Close,
    Undefined,
}

// ---

// Endpoints spoken by our device
//
// GetUniqueIdEndpoint is mandatory, the others are examples
endpoints! {
    list = ENDPOINT_LIST;
    | EndpointTy                | RequestTy     | ResponseTy            | Path                          |
    | ----------                | ---------     | ----------            | ----                          |
    | GetUniqueIdEndpoint       | ()            | u64                   | "poststation/unique_id/get"   |
    | RebootToPicoBoot          | ()            | ()                    | "sys/picoboot/reset"          |
    | SleepEndpoint             | SleepMillis   | SleptMillis           | "sys/sleep"                   |
    | GetBakingEndpoint         | ()            | BakingState           | "hw/baking/get"               |
    | SetBakingEndpoint         | BakingState   | ()                    | "hw/baking/set"               |
    | GetLightEndpoint          | ()            | LightState            | "hw/light/get"                |
    | SetLightEndpoint          | LightState    | ()                    | "hw/light/set"                |
    | GetValvePumpEndpoint      | ()            | ValveState            | "hw/valve_pump/get"           |
    | SetValvePumpEndpoint      | ValveState    | ()                    | "hw/valve_pump/set"           |
    | GetValveTransferEndpoint  | ()            | ValveState            | "hw/valve_transfer/get"       |
    | SetValveTransferEndpoint  | ValveState    | ()                    | "hw/valve_transfer/set"       |
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
}
