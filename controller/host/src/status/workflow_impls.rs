//! Implementations of status for the workflows.

use icd::{BakingState, FlowMeterState, VctState};

use crate::status::{InstrumentStatus, PressureReading};

impl InstrumentStatus {
    /// Is baking turned off?
    pub fn get_baking_state(&self) -> BakingState {
        self.baking_curr.clone()
    }

    /// Get the flow meter state.
    pub fn get_flow_meter_state(&self) -> FlowMeterState {
        self.flow_meter_curr.clone()
    }

    /// Get the pressures as a tuple.
    ///
    /// Returns: (p_chamber, p_transfer)
    pub fn get_pressures(&self) -> (PressureReading, PressureReading) {
        (
            self.pressure_chamber_current,
            self.pressure_transfer_current,
        )
    }

    /// Get the VCT state.
    pub fn get_vct_state(&self) -> VctState {
        self.vct_curr.clone()
    }
}
