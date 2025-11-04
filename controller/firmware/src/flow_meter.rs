//! Handle the flow meter sensor connected to the cryocooler (read only).

use embassy_rp::gpio::Input;
use icd::FlowMeterState;

/// Flow meter control.
///
/// This flow meter is read only, nothing can be set.
pub struct FlowMeter {
    pin_signal: Input<'static>,
}

impl FlowMeter {
    /// Create a new flow meter from the given signal pin.
    pub fn new(pin_signal: Input<'static>) -> Self {
        Self { pin_signal }
    }

    /// Get the state of the flow meter signal.
    ///
    /// If the flow is okay, this signal is low.
    pub fn get_state(&self) -> FlowMeterState {
        match self.pin_signal.is_low() {
            true => FlowMeterState::Ok,
            false => FlowMeterState::FlowError,
        }
    }
}
