//! Handle the instrument status and, when status changes, update UI.

use std::collections::HashMap;

use anyhow::{Result, bail};

use icd::{BakingState, FlowMeterState, InstrumentState, LightState, ValveState, VctState};
use measurements::{Power, Pressure, Temperature};
use pfeiffer_hicube::PumpStandState;
use slint::{ComponentHandle, Weak};
use sunpower_cryotelgt::CoolerState;

use crate::{
    app::{AppWindow, BakingTime, Logic, PumpStandStateGUI, ValveOrPumpState},
    instruments::omnicontrol::{Gauge, GaugeStatus},
    plots::{
        PressureDataPoint, PressurePlotCommands, TemperatureDataPoint, TemperaturePlotCommands,
        send_pressure_plot_cmd_now, send_temperature_plot_cmd_now,
    },
};

pub struct InstrumentStatus {
    ui: Option<Weak<AppWindow>>,
    baking_call: BakingState,
    baking_curr: BakingState,
    chamber_light: LightState,
    cooler_state: CoolerState,
    flow_meter_curr: FlowMeterState,
    ion_pump_state: ValveOrPumpState,
    hi_cube_pump_stand_state: PumpStandState,
    hi_cube_venting_valve: ValveOrPumpState,
    power_cooler_current: Power,
    pressure_chamber_current: PressureReading,
    pressure_chamber_gauge: GaugeStatus,
    pressure_transfer_current: PressureReading,
    pressure_transfer_gauge: GaugeStatus,
    temperature_bridge: Temperature,
    temperature_cooler: Temperature,
    temperature_sample: Temperature,
    temperature_setpoint: Temperature,
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
            chamber_light: LightState::Off,
            cooler_state: CoolerState::Disabled,
            flow_meter_curr: FlowMeterState::default(),
            hi_cube_pump_stand_state: PumpStandState::Other,
            hi_cube_venting_valve: ValveOrPumpState::UndefinedOrError,
            ion_pump_state: ValveOrPumpState::UndefinedOrError,
            power_cooler_current: Power::default(), // 0.0 W
            pressure_chamber_current: PressureReading::default(), // Off
            pressure_chamber_gauge: GaugeStatus::Off,
            pressure_transfer_current: PressureReading::default(), // Off
            pressure_transfer_gauge: GaugeStatus::Off,
            temperature_bridge: Temperature::default(), // 0.0 K
            temperature_cooler: Temperature::default(), // 0.0 K
            temperature_sample: Temperature::default(), // 0.0 K
            temperature_setpoint: Temperature::default(), // 0.0 K
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
    /// If the status is set properly, this will return an `Ok(())`, otherwise an error.
    pub fn initialize_call_states_from_bc(&mut self) -> Result<()> {
        self.baking_call = self.baking_curr.clone();
        self.valve_pump_call = self.valve_pump_curr.clone();
        self.valve_transfer_call = self.valve_transfer_curr.clone();

        // set UI with buttons (baking is set from broadcast)
        let valve_pump_is_open = self.valve_pump_call.is_open();
        let valve_transfer_is_open = self.valve_transfer_call.is_open();
        if let Some(ui) = &self.ui {
            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Logic>()
                    .set_pump_valve_is_open(valve_pump_is_open);
                ui.global::<Logic>()
                    .set_transfer_valve_is_open(valve_transfer_is_open);
            })?;
            Ok(())
        } else {
            bail!("UI not set");
        }
    }

    /// Get if the UI is set.
    pub fn get_ui_is_set(&self) -> bool {
        self.ui.is_some()
    }

    /// Set the UI component of this class.
    ///
    /// Can be set later such that the new can initialize it as `None`.
    pub fn set_ui(&mut self, ui: Weak<AppWindow>) {
        self.ui = Some(ui);
    }

    /// Set the state of the light.
    pub fn set_chamber_light(&mut self, state: LightState) {
        self.chamber_light = state;
    }

    /// Set the state of the light and update UI.
    pub fn set_chamber_light_and_ui(&mut self, state: LightState) -> Result<()> {
        self.chamber_light = state.clone();

        let light_is_on = matches!(state, LightState::On);
        let ui = self
            .ui
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UI not set"))?
            .clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Logic>().set_light_is_on(light_is_on);
        })?;

        Ok(())
    }

    /// Set the Pfeiffer HiCube pump stand state and update the UI.
    ///
    /// Note that we only call the pump in an ON state when the whole pump stand is on!
    pub fn set_hicube_pump_stand_state_and_ui(&mut self, state: PumpStandState) -> Result<()> {
        self.hi_cube_pump_stand_state = state.clone();

        let gui_state = match state {
            PumpStandState::On => PumpStandStateGUI::On,
            PumpStandState::Off => PumpStandStateGUI::Off,
            PumpStandState::SpinningUp => PumpStandStateGUI::SpinningUp,
            PumpStandState::SpinningDown => PumpStandStateGUI::SpinningDown,
            PumpStandState::Other => PumpStandStateGUI::Error,
        };

        let pump_stand_is_on = match gui_state {
            PumpStandStateGUI::On => true,
            PumpStandStateGUI::Off => false,
            PumpStandStateGUI::SpinningUp => true,
            PumpStandStateGUI::SpinningDown => false,
            PumpStandStateGUI::Error => false,
        };
        let ui = self
            .ui
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UI not set"))?
            .clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Logic>().set_pump_stand_state(gui_state);
            ui.global::<Logic>().set_pump_stand_is_on(pump_stand_is_on);
        })?;

        Ok(())
    }

    /// Set the Pfeiffer HiCube vent valve state and update UI.
    pub fn set_hicube_vent_valve_state_and_ui(&mut self, state: ValveOrPumpState) -> Result<()> {
        self.hi_cube_venting_valve = state;
        let valve_is_open = matches!(state, ValveOrPumpState::OpenOrOn);

        let ui = self
            .ui
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UI not set"))?
            .clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Logic>().set_vent_valve_state(state);
            ui.global::<Logic>().set_vent_valve_is_open(valve_is_open);
        })?;

        Ok(())
    }

    /// Set ion pump state and update UI.
    pub fn set_ion_pump_state_and_ui(&mut self, state: ValveOrPumpState) -> Result<()> {
        self.ion_pump_state = state.clone();

        let ion_pump_switch_state = matches!(state, ValveOrPumpState::OpenOrOn);
        let ui = self
            .ui
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UI not set"))?
            .clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Logic>().set_ion_pump_state(state);
            ui.global::<Logic>()
                .set_ion_pump_is_on(ion_pump_switch_state);
        })?;

        Ok(())
    }

    /// Set the setpoint temperature for the cooler and update UI.
    pub fn set_temperature_setpoint_and_ui(&mut self, setpoint: Temperature) -> Result<()> {
        self.temperature_setpoint = setpoint;

        let ui = self
            .ui
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UI not set"))?
            .clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Logic>()
                .set_target_temp(setpoint.as_kelvin().round() as i32);
        })?;

        Ok(())
    }

    /// Set the cooler state and the UI.
    pub fn set_cooler_state_and_ui(&mut self, state: CoolerState) -> Result<()> {
        self.cooler_state = state.clone();

        let ui = self
            .ui
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UI not set"))?
            .clone();
        ui.upgrade_in_event_loop(move |ui| {
            ui.global::<Logic>()
                .set_cryocooler_is_on(matches!(state, CoolerState::Enabled));
        })?;
        Ok(())
    }

    /// Set the current power of the cooler.
    pub fn set_power_cooler_current(&mut self, power: Power) {
        self.power_cooler_current = power;
    }

    /// Set the current pressures and gauge statuses.
    pub fn set_pressures(&mut self, pressure_hm: HashMap<Gauge, Option<Pressure>>) {
        // pressures
        self.pressure_chamber_current = match pressure_hm.get(&Gauge::Chamber) {
            Some(Some(p)) => PressureReading::Value(*p),
            Some(None) => PressureReading::Off,
            None => PressureReading::Error,
        };
        self.pressure_transfer_current = match pressure_hm.get(&Gauge::Transfer) {
            Some(Some(p)) => PressureReading::Value(*p),
            Some(None) => PressureReading::Off,
            None => PressureReading::Error,
        };

        // gauge statuses
        match self.pressure_chamber_current {
            PressureReading::Off => self.pressure_chamber_gauge = GaugeStatus::Off,
            PressureReading::Value(_) => self.pressure_chamber_gauge = GaugeStatus::On,
            _ => {}
        }
        match self.pressure_transfer_current {
            PressureReading::Off => self.pressure_transfer_gauge = GaugeStatus::Off,
            PressureReading::Value(_) => self.pressure_transfer_gauge = GaugeStatus::On,
            _ => {}
        }

        self.send_pressures_to_plot();
    }

    fn send_pressures_to_plot(&self) {
        if let (PressureReading::Value(p_chamber), PressureReading::Value(p_transfer)) = (
            self.pressure_chamber_current,
            self.pressure_transfer_current,
        ) {
            let dp = PressureDataPoint {
                ts: chrono::Local::now(),
                chamber: p_chamber.as_millibars(),
                transfer: p_transfer.as_millibars(),
            };

            send_pressure_plot_cmd_now(PressurePlotCommands::AddDataPoint(dp));
        }
    }

    /// Set temperature values from instrument status.
    pub fn set_temperatures(
        &mut self,
        bridge: Temperature,
        cooler: Temperature,
        sample: Temperature,
    ) {
        self.temperature_bridge = bridge;
        self.temperature_cooler = cooler;
        self.temperature_sample = sample;

        self.send_temperatures_to_plot();
    }

    /// Send temperatures to plot
    pub fn send_temperatures_to_plot(&self) {
        // if a temperature is out 0 -> below yaxis minimum!
        let dp = TemperatureDataPoint {
            ts: chrono::Local::now(),
            sample: self.temperature_sample.as_kelvin(),
            bridge: self.temperature_bridge.as_kelvin(),
            cooler: self.temperature_cooler.as_kelvin(),
        };
        send_temperature_plot_cmd_now(TemperaturePlotCommands::AddDataPoint(dp));
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

    pub fn update_ui_instrument_status(&self) {
        if let Some(ui) = &self.ui {
            let sample_temp = self.temperature_sample.as_kelvin().round() as i32;
            let bridge_temp = self.temperature_bridge.as_kelvin().round() as i32;
            let cooler_temp = self.temperature_cooler.as_kelvin().round() as i32;

            let cooler_current_power = if self.cooler_state == CoolerState::Enabled {
                self.power_cooler_current.as_watts().round() as i32
            } else {
                0
            };

            let pressure_chamber_display = self.pressure_chamber_current.as_shared_string();
            let pressure_chamber_gauge_is_on: bool = self.pressure_chamber_gauge.into();
            let pressure_transfer_display = self.pressure_transfer_current.as_shared_string();
            let pressure_transfer_gauge_is_on: bool = self.pressure_transfer_gauge.into();

            ui.upgrade_in_event_loop(move |ui| {
                ui.global::<Logic>().set_sample_temp(sample_temp);
                ui.global::<Logic>().set_bridge_temp(bridge_temp);
                ui.global::<Logic>().set_cooler_temp(cooler_temp);
                ui.global::<Logic>().set_current_power(cooler_current_power);
                ui.global::<Logic>()
                    .set_chamber_pressure(pressure_chamber_display);
                ui.global::<Logic>()
                    .set_chamber_gauge_is_on(pressure_chamber_gauge_is_on);
                ui.global::<Logic>()
                    .set_transfer_pressure(pressure_transfer_display);
                ui.global::<Logic>()
                    .set_transfer_gauge_is_on(pressure_transfer_gauge_is_on);
            })
            .unwrap();
        }
    }
}

/// Pressure reading of a givne gauge.
#[derive(Debug, Default, Copy, Clone)]
pub enum PressureReading {
    /// Valid pressure reading.
    Value(Pressure),
    /// Gauge is off.
    #[default]
    Off,
    /// Error occured.
    Error,
}

impl PressureReading {
    /// Get a string to directly display on the UI.
    pub fn as_shared_string(&self) -> slint::SharedString {
        let display_str = match self {
            PressureReading::Value(p) => &format!("{:.2E} mbar", p.as_millibars()),
            PressureReading::Off => "Off",
            PressureReading::Error => "Error",
        };
        display_str.into()
    }
}
