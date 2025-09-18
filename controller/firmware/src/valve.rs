//! Module to provide valve control functionality.

use embassy_futures::select::{Either, select};
use embassy_rp::gpio::{Input, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::{Duration, Timer};
use icd::ValveState;

pub static SET_VALVE_PUMP_SIGNAL: Signal<CriticalSectionRawMutex, ValveState> = Signal::new();
pub static GET_VALVE_PUMP_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static WATCH_VALVE_PUMP: Watch<CriticalSectionRawMutex, ValveState, 2> = Watch::new();

pub static SET_VALVE_TRANSFER_SIGNAL: Signal<CriticalSectionRawMutex, ValveState> = Signal::new();
pub static GET_VALVE_TRANSFER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static WATCH_VALVE_TRANSFER: Watch<CriticalSectionRawMutex, ValveState, 2> = Watch::new();

pub struct ValveCtrl {
    set_open: Output<'static>,
    set_close: Output<'static>,
    pulse_duration_ms: u64,
    get_open: Input<'static>,
    get_close: Input<'static>,
}

impl ValveCtrl {
    /// Create a new valve controller.
    pub fn new(
        set_open: Output<'static>,
        set_close: Output<'static>,
        pulse_duration_ms: u64,
        get_open: Input<'static>,
        get_close: Input<'static>,
    ) -> Self {
        Self {
            set_open,
            set_close,
            pulse_duration_ms,
            get_open,
            get_close,
        }
    }

    /// Close the valve.
    pub async fn close(&mut self) {
        self.set_close.set_high();
        Timer::after(Duration::from_millis(self.pulse_duration_ms)).await;
        self.set_close.set_low();
    }

    /// Open the valve.
    pub async fn open(&mut self) {
        self.set_open.set_high();
        Timer::after(Duration::from_millis(self.pulse_duration_ms)).await;
        self.set_open.set_low();
    }

    fn is_closed(&self) -> bool {
        self.get_close.is_low()
    }

    fn is_open(&self) -> bool {
        self.get_open.is_low()
    }
    ///
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

/// Inputs for the status valves structure.
pub struct ValveStatus {
    open: Input<'static>,
    closed: Input<'static>,
}

impl ValveStatus {
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
    let watch_sender = WATCH_VALVE_TRANSFER.sender();
    loop {
        match select(
            SET_VALVE_TRANSFER_SIGNAL.wait(),
            GET_VALVE_TRANSFER_SIGNAL.wait(),
        )
        .await
        {
            Either::First(stat) => match stat {
                ValveState::Open => {
                    valve.open().await;
                }
                ValveState::Close => {
                    valve.close().await;
                }
                _ => {}
            },
            Either::Second(_) => watch_sender.send(valve.status()),
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn valve_pump_task(mut valve: ValveCtrl) {
    loop {
        let watch_sender = WATCH_VALVE_PUMP.sender();
        match select(
            SET_VALVE_PUMP_SIGNAL.wait(),
            GET_VALVE_PUMP_SIGNAL.wait(),
        )
        .await
        {
            Either::First(stat) => match stat {
                ValveState::Open => {
                    valve.open().await;
                }
                ValveState::Close => {
                    valve.close().await;
                }
                _ => {}
            },
            Either::Second(_) => watch_sender.send(valve.status()),
        }
    }
}
