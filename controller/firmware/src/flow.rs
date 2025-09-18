//! Query the status of the flow meter.

use embassy_rp::gpio::Input;
use icd::FlowMeterState;

pub struct FlowMeterCtrl {
    pin_nerr: Input<'static>,
}

impl FlowMeterCtrl {
    /// Create a new flow meter struct.
    pub fn new(pin_nerr: Input<'static>) -> Self {
        Self { pin_nerr }
    }

    pub fn status(&mut self) -> FlowMeterState {
        match self.pin_nerr.is_high() {
            true => FlowMeterState::Ok,
            false => FlowMeterState::FlowError,
        }
    }
}
