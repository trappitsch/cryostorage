//! Implementations of status for the workflows.

use icd::VctState;

use crate::status::{InstrumentStatus, PressureReading};

impl InstrumentStatus {
    /// Get the pressures as a tuple.
    ///
    /// Returns: (p_chamber, p_transfer)
    pub fn get_pressures(&self) -> (PressureReading, PressureReading) {
        (self.pressure_chamber_current, self.pressure_transfer_current)
    }

    /// Get the VCT state.
    pub fn get_vct_state(&self) -> VctState {
        self.vct_curr.clone()
    }
}
