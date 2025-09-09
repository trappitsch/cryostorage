//! Module to provide valve control functionality.

use embassy_rp::gpio::{Input, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use icd::ValveState;

pub static VALVE_PUMP_SIGNAL: Signal<CriticalSectionRawMutex, ValveState> = Signal::new();
pub static VALVE_TRANSFER_SIGNAL: Signal<CriticalSectionRawMutex, ValveState> = Signal::new();

pub struct ValveCtrl {
    pin_open: Output<'static>,
    pin_close: Output<'static>,
    pulse_duration_ms: u64,
}

impl ValveCtrl {
    /// Create a new valve controller.
    pub fn new(
        pin_open: Output<'static>,
        pin_close: Output<'static>,
        pulse_duration_ms: u64,
    ) -> Self {
        Self {
            pin_open,
            pin_close,
            pulse_duration_ms,
        }
    }

    /// Close the valve.
    pub async fn close(&mut self) {
        self.pin_close.set_high();
        Timer::after(Duration::from_millis(self.pulse_duration_ms)).await;
        self.pin_close.set_low();
    }

    /// Open the valve.
    pub async fn open(&mut self) {
        self.pin_open.set_high();
        Timer::after(Duration::from_millis(self.pulse_duration_ms)).await;
        self.pin_open.set_low();
    }
}

/// Inputs for the status valves structure.
pub struct ValveStat {
    open: Input<'static>,
    closed: Input<'static>,
}

impl ValveStat {
    /// Get a new valve input status struct.
    pub fn new(open: Input<'static>, closed: Input<'static>) -> Self {
        Self { open, closed }
    }

    fn is_closed(&self) -> bool {
        self.closed.is_low()
    }
    fn is_open(&self) -> bool {
        self.open.is_low()
    }

    /// Get the current status of the valve.
    pub fn status(&self) -> ValveState {
        let vlv_open = self.is_open();
        let vlv_closed = self.is_closed();

        if vlv_open && !vlv_closed {
            ValveState::Open
        } else if !vlv_open && vlv_closed {
            ValveState::Close
        } else {
            ValveState::Undefined
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn valve_transfer_task(mut valve: ValveCtrl) {
    loop {
        match VALVE_TRANSFER_SIGNAL.wait().await {
            ValveState::Open => {
                valve.open().await;
            }
            ValveState::Close => {
                valve.close().await;
            }
            _ => {}
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn valve_pump_task(mut valve: ValveCtrl) {
    loop {
        match VALVE_PUMP_SIGNAL.wait().await {
            ValveState::Open => {
                valve.open().await;
            }
            ValveState::Close => {
                valve.close().await;
            }
            _ => {}
        }
    }
}
