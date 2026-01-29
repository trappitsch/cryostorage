//! Handle the instrument status and, when status changes, update UI.

use icd::{BakingState, FlowMeterState, InstrumentState, ValveState, VctState};
use slint::{ComponentHandle, Weak};

use crate::app::{AppWindow, BakingTime, Logic, ValveOrPumpState};

pub struct InstrumentStatus {
    ui: Option<Weak<AppWindow>>,
    baking_call: BakingState,
    baking_curr: BakingState,
    flow_meter_curr: FlowMeterState,
    valve_pump_call: ValveState,
    valve_pump_curr: ValveState,
    valve_transfer_call: ValveState,
    valve_transfer_curr: ValveState,
    vct_curr: VctState,
}

impl InstrumentStatus {
    /// Create a new InstrumentStatus with a given ui and default values.
    pub fn new() -> Self {
        Self {
            ui: None,
            baking_call: BakingState::default(),
            baking_curr: BakingState::default(),
            flow_meter_curr: FlowMeterState::default(),
            valve_pump_call: ValveState::default(),
            valve_pump_curr: ValveState::default(),
            valve_transfer_call: ValveState::default(),
            valve_transfer_curr: ValveState::default(),
            vct_curr: VctState::default(),
        }
    }

    /// Initialize the call states to the current states.
    ///
    /// This is used at startup to avoid false error states.
    /// FIXME: This is currently not used anywhere. Needs to be fixed.
    pub fn initialize_call_states(&mut self) {
        self.baking_call = self.baking_curr.clone();
        self.valve_pump_call = self.valve_pump_curr.clone();
        self.valve_transfer_call = self.valve_transfer_curr.clone();
    }

    /// Set the UI component of this class.
    ///
    /// Can be set later such that the new can initialize it as `None`.
    pub fn set_ui(&mut self, ui: Weak<AppWindow>) {
        self.ui = Some(ui);
    }

    /// Set valve pump called state.
    pub fn set_valve_pump_call(&mut self, state: ValveState) {
        self.valve_pump_call = state;
    }

    /// Set valve transfer called state.
    pub fn set_valve_transfer_call(&mut self, state: ValveState) {
        self.valve_transfer_call = state;
    }

    /// Update status from a broadcast message. Then update UI.
    pub fn update_from_controller_broadcast(&mut self, status: InstrumentState) {
        self.baking_curr = status.baking;
        self.flow_meter_curr = status.flow_meter;
        self.valve_pump_curr = status.pump_valve;
        self.valve_transfer_curr = status.transfer_valve;
        self.vct_curr = status.vct;
        self.update_ui_controler_broadcast();
    }

    /// Update the UI after a controller broadcast.
    fn update_ui_controler_broadcast(&self) {
        if let Some(ui) = &self.ui {
            // Baking
            let (baking_is_enabled, baking_time) = match self.baking_curr {
                BakingState::On { time_sec } => {
                    let baking_time = BakingTime {
                        hours: (time_sec.div_euclid(3600)) as i32,
                        minutes: ((time_sec % 3600).div_euclid(60)) as i32,
                        seconds: (time_sec % 60) as i32,
                    };
                    (true, baking_time)
                }
                BakingState::Off => (false, BakingTime::default()),
            };

            // Water flow
            let water_flow_ok = matches!(self.flow_meter_curr, FlowMeterState::Ok);

            // Pump valve status
            let valve_pump_state = if self.valve_pump_call == self.valve_pump_curr {
                match self.valve_pump_curr {
                    ValveState::Open => ValveOrPumpState::OpenOrOn,
                    ValveState::Closed => ValveOrPumpState::ClosedOrOff,
                    // Undefined in call state only possible at startup.
                    ValveState::Undefined => ValveOrPumpState::UndefinedOrError,
                }
            } else {
                match self.valve_pump_curr {
                    ValveState::Undefined => ValveOrPumpState::UndefinedOrError,
                    _ => ValveOrPumpState::SetDifferentFromCalled,
                }
            };

            // Transfer valve status
            let valve_transfer_state = if self.valve_transfer_call == self.valve_transfer_curr {
                match self.valve_transfer_curr {
                    ValveState::Open => ValveOrPumpState::OpenOrOn,
                    ValveState::Closed => ValveOrPumpState::ClosedOrOff,
                    // Undefined in call state only possible at startup.
                    ValveState::Undefined => ValveOrPumpState::UndefinedOrError,
                }
            } else {
                match self.valve_transfer_curr {
                    ValveState::Undefined => ValveOrPumpState::UndefinedOrError,
                    _ => ValveOrPumpState::SetDifferentFromCalled,
                }
            };

            // Update UI in event loop
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Logic>()
                    .set_baking_is_enabled(baking_is_enabled);
                if baking_is_enabled {
                    ui.global::<Logic>().set_baking_time(baking_time);
                };
                ui.global::<Logic>().set_water_flow_ok(water_flow_ok);
                ui.global::<Logic>().set_pump_valve_state(valve_pump_state);
                ui.global::<Logic>()
                    .set_transfer_valve_state(valve_transfer_state);
            })
            .unwrap();
        }
    }
}
