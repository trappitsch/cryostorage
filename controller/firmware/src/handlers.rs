use core::sync::atomic::{Ordering, compiler_fence};

use icd::{BakingState, LightState, ValveState, VctHandshake};
use postcard_rpc::header::VarHeader;

use crate::{
    app::Context,
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
pub fn baking_set_handler(_context: &mut Context, _header: VarHeader, arg: BakingState) {
    baking_set(arg);
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

/// Set the pump valve state.
pub fn pump_valve_set_handler(_context: &mut Context, _header: VarHeader, arg: ValveState) {
    pump_valve_set(arg);
}

/// Set the transfer valve state.
pub fn transfer_valve_set_handler(_context: &mut Context, _header: VarHeader, arg: ValveState) {
    transfer_valve_set(arg);
}

/// Set the VCT handshake state.
pub fn vct_handshake_set_handler(_context: &mut Context, _header: VarHeader, arg: VctHandshake) {
    vct_set_handshake(arg);
}
