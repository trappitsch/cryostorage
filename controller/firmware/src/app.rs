//! A basic postcard-rpc/poststation-compatible application

use crate::handlers::{
    baking_set_handler, light_get_handler, light_set_handler, picoboot_reset,
    pump_valve_set_handler, transfer_valve_set_handler, unique_id, vct_handshake_set_handler,
};
use embassy_rp::gpio::Output;
use embassy_rp::{peripherals::USB, usb};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use icd::{ENDPOINT_LIST, TOPICS_IN_LIST, TOPICS_OUT_LIST};
use icd::{
    GetLightEndpoint, GetUniqueIdEndpoint, RebootToPicoBoot, SetBakingEndpoint, SetLightEndpoint,
    SetPumpValveEndpoint, SetTransferValveEndpoint, SetVctHandshakeEndpoint,
};

use postcard_rpc::server::impls::embassy_usb_v0_4::{
    PacketBuffers,
    dispatch_impl::{WireRxBuf, WireRxImpl, WireSpawnImpl, WireStorage, WireTxImpl},
};
use postcard_rpc::{
    define_dispatch,
    server::{Server, SpawnContext},
};
use static_cell::ConstStaticCell;

/// Context contains the data that we will pass (as a mutable reference)
/// to each endpoint or topic handler
pub struct Context {
    /// We'll use this unique ID to identify ourselves to the poststation
    /// server. This should be unique per device.
    pub unique_id: u64,
    pub light_output: Output<'static>,
}

impl SpawnContext for Context {
    type SpawnCtxt = TaskContext;

    fn spawn_ctxt(&mut self) -> Self::SpawnCtxt {
        TaskContext {
            unique_id: self.unique_id,
        }
    }
}

pub struct TaskContext {
    #[allow(dead_code)]
    pub unique_id: u64,
}

/// Type Aliases
pub type AppDriver = usb::Driver<'static, USB>;
/// Storage describes the things we need to keep as a static, so it can be shared
/// with anyone who needs to send messages.
pub type AppStorage = WireStorage<ThreadModeRawMutex, AppDriver, 256, 256, 64, 256>;
/// BufStorage is the space used for receiving and sending frames. These values
/// control the largest frames we can send or receive.
pub type BufStorage = PacketBuffers<1024, 1024>;
/// AppTx is the type of our sender, which is how we send information to the client
pub type AppTx = WireTxImpl<ThreadModeRawMutex, AppDriver>;
/// AppRx is the type of our receiver, which is how we receive information from the client
pub type AppRx = WireRxImpl<AppDriver>;
/// AppServer is the type of the postcard-rpc server we are using
pub type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

/// Statically store our packet buffers
pub static PBUFS: ConstStaticCell<BufStorage> = ConstStaticCell::new(BufStorage::new());
/// Statically store our USB app buffers
pub static STORAGE: AppStorage = AppStorage::new();

// Macro to define the application
define_dispatch! {
    // Name of the app
    app: MyApp;
    // This chooses how we spawn functions. Here, we use the implementation
    // from the `embassy_usb_v0_5` implementation
    spawn_fn: spawn_fn;
    // This is our TX impl, which we aliased above
    tx_impl: AppTx;
    // This is our spawn impl, which also comes from `embassy_usb_v0_4`.
    spawn_impl: WireSpawnImpl;
    // This is the context type we defined above
    context: Context;

    // Endpoints are how we handle request/response pairs from the client.
    endpoints: {
        // This list comes from our ICD crate. All of the endpoint handlers we
        // define below MUST be contained in this list.
        list: ENDPOINT_LIST;

        | EndpointTy                | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | GetUniqueIdEndpoint       | blocking  | unique_id                     |
        | RebootToPicoBoot          | blocking  | picoboot_reset                |
        | SetBakingEndpoint         | blocking  | baking_set_handler            |
        | GetLightEndpoint          | blocking  | light_get_handler             |
        | SetLightEndpoint          | blocking  | light_set_handler             |
        | SetPumpValveEndpoint      | blocking  | pump_valve_set_handler        |
        | SetTransferValveEndpoint  | blocking  | transfer_valve_set_handler    |
        | SetVctHandshakeEndpoint   | blocking  | vct_handshake_set_handler     |
    };

    // Topics IN are messages we receive from the client, but that we do not reply
    // directly to. These have the same "kinds" and "handlers" as endpoints, however
    // these handlers never return a value
    topics_in: {
        // This list comes from our ICD crate. All of the topic handlers we
        // define below MUST be contained in this list.
        list: TOPICS_IN_LIST;

        | TopicTy                   | kind      | handler                       |
        | ----------                | ----      | -------                       |
    };

    // Topics OUT are the messages we send to the client whenever we'd like. Since
    // these are outgoing, we do not need to define handlers for them.
    topics_out: {
        // This list comes from our ICD crate.
        list: TOPICS_OUT_LIST;
    };
}
