use core::sync::atomic::{Ordering, compiler_fence};

use embassy_time::{Instant, Timer};
use postcard_rpc::{header::VarHeader, server::Sender};
use icd::{BakingState, LightState, SleepEndpoint, SleepMillis, SleptMillis, ValveState};

use crate::{app::{AppTx, Context, TaskContext}, valve::{VALVE_PUMP_SIGNAL, VALVE_TRANSFER_SIGNAL}};

/// This is an example of a BLOCKING handler.
pub fn unique_id(context: &mut Context, _header: VarHeader, _arg: ()) -> u64 {
    context.unique_id
}

/// Also a BLOCKING handler
pub fn picoboot_reset(_context: &mut Context, _header: VarHeader, _arg: ()) {
    embassy_rp::rom_data::reboot(0x0002, 500, 0x0000, 0x0000);
    loop {
        // Wait for reset...
        compiler_fence(Ordering::SeqCst);
    }
}

/// Get the state of the baking
pub fn get_baking(context: &mut Context, _header: VarHeader, _args: ()) -> BakingState {
    context.baking_ctrl.get_status()
}

/// Set the baking
pub fn set_baking(context: &mut Context, _header: VarHeader, arg: BakingState) {
    context.baking_ctrl.control(arg);
}

pub fn set_light(context: &mut Context, _header: VarHeader, arg: LightState) {
    match arg {
        LightState::Off => context.p_light.set_low(),
        LightState::On => context.p_light.set_high(),
    }
}

pub fn get_light(context: &mut Context, _header: VarHeader, _arg: ()) -> LightState {
    match context.p_light.is_set_low() {
        true => LightState::Off,
        false => LightState::On,
    }
}

/// Get the status of the pump valve
pub fn get_valve_pump(context: &mut Context, _header: VarHeader, _arg: ()) -> ValveState {
    context.valve_pump_status.status()
}

/// Set the pump valve
pub fn set_valve_pump(_context: &mut Context, _header: VarHeader, arg: ValveState) {
    VALVE_PUMP_SIGNAL.signal(arg);
}
///
/// Get the status of the transfer valve
pub fn get_valve_transfer(context: &mut Context, _header: VarHeader, _arg: ()) -> ValveState {
    context.valve_transfer_status.status()
}

/// Set the transfer valve
pub fn set_valve_transfer(_context: &mut Context, _header: VarHeader, arg: ValveState) {
    VALVE_TRANSFER_SIGNAL.signal(arg)
}

/// This is a SPAWN handler
///
/// The pool size of three means we can have up to three of these requests "in flight"
/// at the same time. We will return an error if a fourth is requested at the same time
#[embassy_executor::task(pool_size = 3)]
pub async fn sleep_handler(
    _context: TaskContext,
    header: VarHeader,
    arg: SleepMillis,
    sender: Sender<AppTx>,
) {
    // We can send string logs, using the sender
    let _ = sender.log_str("Starting sleep...").await;
    let start = Instant::now();
    Timer::after_millis(arg.millis.into()).await;
    let _ = sender.log_str("Finished sleep").await;
    // Async handlers have to manually reply, as embassy doesn't support returning by value
    let _ = sender
        .reply::<SleepEndpoint>(
            header.seq_no,
            &SleptMillis {
                millis: start.elapsed().as_millis() as u16,
            },
        )
        .await;
}
