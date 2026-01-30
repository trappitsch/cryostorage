//! Broadcast the instrument state at regular intervals.

use embassy_rp::watchdog::Watchdog;
use embassy_time::{Duration, Timer};
use icd::{BROADCAST_INTERVAL_MS, BcInstStatus, InstrumentState};
use postcard_rpc::server::Sender;

use crate::{
    app::AppTx,
    baking::baking_get,
    flow_meter::FlowMeter,
    valve::{pump_valve_get, transfer_valve_get},
    vct::vct_get_state,
};

/// Broadcasting task to periodically send the instrument state.
#[embassy_executor::task(pool_size = 1)]
pub async fn broadcaster_task(
    sender: Sender<AppTx>,
    mut watchdog: Watchdog,
    flow_meter: FlowMeter,
) {
    let mut seq = 0u32;
    let mut instrument_state = InstrumentState::default();

    loop {
        Timer::after(Duration::from_millis(BROADCAST_INTERVAL_MS)).await;

        // Update the baking state and store time in watchdog scratch
        let baking_state = baking_get().await;
        watchdog.set_scratch(0, baking_state.get_time_sec_u32());
        instrument_state.baking = baking_state;

        // Update the flow meter
        instrument_state.flow_meter = flow_meter.get_state();

        // Update the valves
        instrument_state.pump_valve = pump_valve_get().await;
        instrument_state.transfer_valve = transfer_valve_get().await;

        // Update the VCT state
        instrument_state.vct = vct_get_state().await;

        // Broadcast the message
        if sender
            .publish::<BcInstStatus>(seq.into(), &instrument_state)
            .await
            .is_err()
        {
            defmt::warn!("Broadcast #{} failed", seq)
        }
        seq = seq.wrapping_add(1);

        // Feed the watchdog
        watchdog.feed();
    }
}
