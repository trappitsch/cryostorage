//! Deals with solenoid valves.

use embassy_rp::gpio::{Input, Output};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use icd::ValveState;

static PUMP_VALVE_SIGNAL: Signal<ThreadModeRawMutex, ValveCommand> = Signal::new();
static PUMP_VALVE_STATE_SIGNAL: Signal<ThreadModeRawMutex, ValveState> = Signal::new();
static TRANSFER_VALVE_SIGNAL: Signal<ThreadModeRawMutex, ValveCommand> = Signal::new();
static TRANSFER_VALVE_STATE_SIGNAL: Signal<ThreadModeRawMutex, ValveState> = Signal::new();

static VALVE_PULSE_TIME: Duration = Duration::from_millis(500);

/// Selector for the different valves.
///
/// All static signals must have been declared before!
pub enum ValveSelector {
    /// Represents the pump valve.
    Pump,
    /// Represents the transfer valve.
    Transfer,
}

/// Control and state pins for a single valve.
pub struct Valve {
    pin_ctrl_open: Output<'static>,
    pin_ctrl_close: Output<'static>,
    pin_state_open: Input<'static>,
    pin_state_closed: Input<'static>,
}

impl Valve {
    /// Create a new valve from the given pins.
    pub fn new(
        pin_ctrl_open: Output<'static>,
        pin_ctrl_close: Output<'static>,
        pin_state_open: Input<'static>,
        pin_state_closed: Input<'static>,
    ) -> Self {
        Self {
            pin_ctrl_open,
            pin_ctrl_close,
            pin_state_open,
            pin_state_closed,
        }
    }

    /// Close the valve.
    pub async fn close(&mut self) {
        self.pin_ctrl_close.set_high();
        Timer::after(VALVE_PULSE_TIME).await;
        self.pin_ctrl_close.set_low();
    }

    /// Open the valve.
    pub async fn open(&mut self) {
        self.pin_ctrl_open.set_high();
        Timer::after(VALVE_PULSE_TIME).await;
        self.pin_ctrl_open.set_low();
    }

    /// Get the current state of the valve.
    pub fn state(&self) -> ValveState {
        let is_open = self.pin_state_open.is_low();
        let is_closed = self.pin_state_closed.is_low();
        if is_open && !is_closed {
            ValveState::Open
        } else if !is_open && is_closed {
            ValveState::Closed
        } else {
            ValveState::Undefined
        }
    }
}

/// Commands for the valve task.
pub enum ValveCommand {
    /// Open the valve.
    Open,
    /// Close the valve.
    Close,
    /// Get the current state of the valve.
    GetState,
}

/// The task for the pump valve.
///
/// If this task dies, the valve will no longer respond to get state commands and thus the
/// broadcaster will stop feeding the watchdog and the system will reset.
#[embassy_executor::task(pool_size = 2)]
pub async fn valve_task(mut valve: Valve, which: ValveSelector) {
    let ctrl_signal = match which {
        ValveSelector::Pump => &PUMP_VALVE_SIGNAL,
        ValveSelector::Transfer => &TRANSFER_VALVE_SIGNAL,
    };

    let state_signal = match which {
        ValveSelector::Pump => &PUMP_VALVE_STATE_SIGNAL,
        ValveSelector::Transfer => &TRANSFER_VALVE_STATE_SIGNAL,
    };

    loop {
        let command = ctrl_signal.wait().await;
        match command {
            ValveCommand::Open => {
                valve.open().await;
            }
            ValveCommand::Close => {
                valve.close().await;
            }
            ValveCommand::GetState => {
                let state = valve.state();
                state_signal.signal(state);
            }
        }
    }
}

/// Wait for a pump valve state update and return the new state.
pub async fn pump_valve_get() -> ValveState {
    PUMP_VALVE_SIGNAL.signal(ValveCommand::GetState);
    PUMP_VALVE_STATE_SIGNAL.wait().await
}

/// Set the pump valve to a given state.
pub fn pump_valve_set(state: ValveState) {
    match state {
        ValveState::Open => {
            PUMP_VALVE_SIGNAL.signal(ValveCommand::Open);
        }
        ValveState::Closed => {
            PUMP_VALVE_SIGNAL.signal(ValveCommand::Close);
        }
        ValveState::Undefined => {
            defmt::warn!("Cannot set pump valve to undefined state");
        }
    }
}

/// Wait for a transfer valve state update and return the new state.
pub async fn transfer_valve_get() -> ValveState {
    TRANSFER_VALVE_SIGNAL.signal(ValveCommand::GetState);
    TRANSFER_VALVE_STATE_SIGNAL.wait().await
}

/// Set the transfer valve to a given state.
pub fn transfer_valve_set(state: ValveState) {
    match state {
        ValveState::Open => {
            TRANSFER_VALVE_SIGNAL.signal(ValveCommand::Open);
        }
        ValveState::Closed => {
            TRANSFER_VALVE_SIGNAL.signal(ValveCommand::Close);
        }
        ValveState::Undefined => {
            defmt::warn!("Cannot set pump valve to undefined state");
        }
    }
}
