use core::sync::atomic::{Ordering, compiler_fence};

use icd::{BakingState, LightState, ValveState, VctHandshake};
use postcard_rpc::{header::VarHeader, server::Sender};

use crate::{
    app::{AppTx, Context, TaskContext},
    baking::baking_set,
    valve::{pump_valve_set, transfer_valve_set},
    vct::vct_set_handshake,
};

/// Get the unique ID of this device
pub fn unique_id(context: &mut Context, _header: VarHeader, _arg: ()) -> u64 {
    context.unique_id
}

/// Reboot into picoboot mode
pub fn picoboot_reset(_context: &mut Context, _header: VarHeader, _arg: ()) {
    embassy_rp::rom_data::reboot(0x0002, 500, 0x0000, 0x0000);
    loop {
        // Wait for reset...
        compiler_fence(Ordering::SeqCst);
    }
}

/// Set the baking state.
#[embassy_executor::task(pool_size = 4)]
pub async fn baking_set_handler(
    _context: TaskContext,
    _header: VarHeader,
    arg: BakingState,
    _sender: Sender<AppTx>,
) {
    baking_set(arg).await;
}

/// Set the light state.
pub fn light_get_handler(context: &mut Context, _header: VarHeader, _arg: ()) -> LightState {
    match context.light_output.is_set_high() {
        true => LightState::On,
        false => LightState::Off,
    }
}

/// Set the light state.
pub fn light_set_handler(context: &mut Context, _header: VarHeader, arg: LightState) {
    match arg {
        LightState::Off => context.light_output.set_low(),
        LightState::On => context.light_output.set_high(),
    }
}

/// Toggle the light state.
pub fn light_toggle_handler(context: &mut Context, _header: VarHeader, _arg: ()) {
    context.light_output.toggle();
}

/// Set the pump valve state.
#[embassy_executor::task(pool_size = 4)]
pub async fn pump_valve_set_handler(
    _context: TaskContext,
    _header: VarHeader,
    arg: ValveState,
    _sender: Sender<AppTx>,
) {
    pump_valve_set(arg).await;
}

/// Set the transfer valve state.
#[embassy_executor::task(pool_size = 4)]
pub async fn transfer_valve_set_handler(
    _context: TaskContext,
    _header: VarHeader,
    arg: ValveState,
    _sender: Sender<AppTx>,
) {
    transfer_valve_set(arg).await;
}

/// Set the VCT handshake state.
#[embassy_executor::task(pool_size = 4)]
pub async fn vct_handshake_set_handler(
    _context: TaskContext,
    _header: VarHeader,
    arg: VctHandshake,
    _sender: Sender<AppTx>,
) {
    vct_set_handshake(arg).await;
}
