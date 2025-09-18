use core::sync::atomic::{Ordering, compiler_fence};

use defmt::error;
use embassy_time::{Instant, Timer};
use icd::{
    BakingState, GetBakingEndpoint, GetValvePumpEndpoint, GetValveTransferEndpoint, GetVctStatus, LightState, SleepEndpoint, SleepMillis, SleptMillis, ValveState, VctHandshakeState
};
use postcard_rpc::{header::VarHeader, server::Sender};

use crate::{
    app::{AppTx, Context, TaskContext},
    baking::{GET_BAKING_SIGNAL, SET_BAKING_SIGNAL, WATCH_BAKING},
    valve::{
        GET_VALVE_PUMP_SIGNAL, GET_VALVE_TRANSFER_SIGNAL, SET_VALVE_PUMP_SIGNAL,
        SET_VALVE_TRANSFER_SIGNAL, WATCH_VALVE_PUMP, WATCH_VALVE_TRANSFER,
    }, vct::{GET_VCT_STATUS, WATCH_VCT_STATUS},
};

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
///
/// Pool size of one as we only have one watch receiver available for this!
#[embassy_executor::task(pool_size = 1)]
pub async fn get_baking(_ctx: TaskContext, header: VarHeader, _args: (), sender: Sender<AppTx>) {
    if let Some(mut rec) = WATCH_BAKING.receiver() {
        GET_BAKING_SIGNAL.signal(());
        let res = rec.get().await;
        let _ = sender.reply::<GetBakingEndpoint>(header.seq_no, &res).await;
    } else {
        error!("handler get baking status could not obtain a watch receiver");
    }
}

/// Set the baking
pub fn set_baking(_ctx: &mut Context, _header: VarHeader, arg: BakingState) {
    SET_BAKING_SIGNAL.signal(arg);
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
///
/// pool size limited to one, as we have only one watch receiver for this task reserved
#[embassy_executor::task(pool_size = 1)]
pub async fn get_valve_pump(_ctx: TaskContext, header: VarHeader, _arg: (), sender: Sender<AppTx>) {
    if let Some(mut rec) = WATCH_VALVE_PUMP.receiver() {
        GET_VALVE_PUMP_SIGNAL.signal(());
        let res = rec.get().await;
        let _ = sender
            .reply::<GetValvePumpEndpoint>(header.seq_no, &res)
            .await;
    } else {
        error!("handler get valve pump could not obtain a watch receiver");
    }
}

/// Set the pump valve
pub fn set_valve_pump(_context: &mut Context, _header: VarHeader, arg: ValveState) {
    SET_VALVE_PUMP_SIGNAL.signal(arg);
}

/// Get the status of the transfer valve
#[embassy_executor::task(pool_size = 1)]
pub async fn get_valve_transfer(
    _ctx: TaskContext,
    header: VarHeader,
    _arg: (),
    sender: Sender<AppTx>,
) {
    if let Some(mut rec) = WATCH_VALVE_TRANSFER.receiver() {
        GET_VALVE_TRANSFER_SIGNAL.signal(());
        let res = rec.get().await;
        let _ = sender
            .reply::<GetValveTransferEndpoint>(header.seq_no, &res)
            .await;
    } else {
        error!("handler get valve pump could not obtain a watch receiver");
    }
}

/// Set the transfer valve
pub fn set_valve_transfer(_context: &mut Context, _header: VarHeader, arg: ValveState) {
    SET_VALVE_TRANSFER_SIGNAL.signal(arg)
}

/// Get the status of the VCT handshake
pub fn get_vct_handshake(context: &mut Context, _header: VarHeader, _arg: ()) -> VctHandshakeState {
    context.vct_ctrl.get_handshake_state()
}

/// Set the status of the VCT handshake
pub fn set_vct_handshake(context: &mut Context, _header: VarHeader, arg: VctHandshakeState) {
    context.vct_ctrl.set_handshake_state(arg);
}

/// Get the VCT status
#[embassy_executor::task(pool_size = 1)]
pub async fn get_vct_status(
    _ctx: TaskContext,
    header: VarHeader,
    _arg: (),
    sender: Sender<AppTx>,
) {
    if let Some(mut rec) = WATCH_VCT_STATUS.receiver() {
        GET_VCT_STATUS.signal(());
        let res = rec.get().await;
        let _ = sender
            .reply::<GetVctStatus>(header.seq_no, &res)
            .await;
    } else {
        error!("handler get vct could not obtain a watch receiver");
    }
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
