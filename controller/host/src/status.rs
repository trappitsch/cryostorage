//! Handle the instrument status and, when status changes, update UI.

use icd::{BakingState, FlowMeterState, InstrumentState, ValveState, VctState};
use slint::{ComponentHandle, Weak};

use crate::app::{AppWindow, BakingTime, Logic};

pub struct InstrumentStatus {
    ui: Option<Weak<AppWindow>>,
    baking: BakingState,
    flow_meter: FlowMeterState,
    valve_pump: ValveState,
    valve_transfer: ValveState,
    vct: VctState,
}

impl InstrumentStatus {
    /// Create a new InstrumentStatus with a given ui and default values.
    pub fn new() -> Self {
        Self {
            ui: None,
            baking: BakingState::default(),
            flow_meter: FlowMeterState::default(),
            valve_pump: ValveState::default(),
            valve_transfer: ValveState::default(),
            vct: VctState::default(),
        }
    }

    /// Set the UI component.
    pub fn set_ui(&mut self, ui: Weak<AppWindow>) {
        self.ui = Some(ui);
    }

    /// Update status from a broadcast message. Then update UI.
    pub fn update_from_bc(&mut self, status: InstrumentState) {
        self.baking = status.baking;
        self.flow_meter = status.flow_meter;
        self.valve_pump = status.pump_valve;
        self.valve_transfer = status.transfer_valve;
        self.vct = status.vct;
        self.update_ui();
    }

    /// Update the UI.
    fn update_ui(&self) {
        if let Some(ui) = &self.ui {
            let (baking_is_enabled, baking_time) = match self.baking {
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
            let water_flow_ok = matches!(self.flow_meter, FlowMeterState::Ok);
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Logic>()
                    .set_baking_is_enabled(baking_is_enabled);
                if baking_is_enabled {
                    ui.global::<Logic>().set_baking_time(baking_time);
                };
                ui.global::<Logic>().set_water_flow_ok(water_flow_ok);
            })
            .unwrap();
        }
    }
}
