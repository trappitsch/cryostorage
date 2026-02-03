//! This module contains the complete GUI logic for the application.

use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use icd::{BakingState, LightState, ValveState, VctHandshake};
use slint::{Model, SharedString, Weak};
use tokio::sync::{mpsc, oneshot};

use crate::{controller::{ControllerCommands, send_cntrl_cmd_now}, prg_config::PrgConfig, status::InstrumentStatus};

slint::include_modules!();

pub fn app_main(
    conf: Arc<Mutex<PrgConfig>>,
    inst_status: Arc<Mutex<InstrumentStatus>>,
    tx_ui_set_logger: oneshot::Sender<Weak<AppWindow>>,
) -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    if tx_ui_set_logger.send(ui.as_weak()).is_err() {
        panic!("Failed to send UI to the logger");
    };

    // initialize the various GUI handlers
    let _controller_cmd_handler =
        ControllerCommandHandler::new(ui.as_weak(), Arc::clone(&conf));
    let _gui_cmd_handler = GuiCommandHandler::new(ui.as_weak(), Arc::clone(&conf));
    let _instrument_cmd_handler =
        InstrumentCommandHandler::new(ui.as_weak(), Arc::clone(&conf));

    // pass the ui to the instrument status handler
    inst_status.lock().expect("Poisoned").set_ui(ui.as_weak());

    // FIXME:
    let mut vh = crate::vacuum_history::VacuumHistory::new(
        crate::vacuum_history::PlotSizePx {
            width: 800,
            height: 400,
        },
        ui.as_weak(),
    );
    vh.add_measurement(
        measurements::Pressure::from_millibars(1.0e-7),
        measurements::Pressure::from_millibars(1.0e-5),
    )?;

    // Debug builds
    #[cfg(debug_assertions)]
    ui.global::<Logic>().set_admin_mode(true);

    ui.show()?;

    ui.run()?;
    Ok(())
}

/// Controller command handler for the GUI.
///
/// This handler controls all the Cryocooler controller commands and sends ControllerCommands to
/// the controller handler, which talks to poststation. This here is the intermediary between the
/// GUI and the rest of the logic.
struct ControllerCommandHandler {
    ui: AppWindow,
    conf: Arc<Mutex<PrgConfig>>,
}

impl ControllerCommandHandler {
    /// Initialize all switches
    fn new(
        ui: Weak<AppWindow>,
        conf: Arc<Mutex<PrgConfig>>,
    ) -> Self {
        let sf = Self {
            ui: ui.unwrap(),
            conf,
        };
        sf.init();
        sf
    }

    /// Initialize the controller command handler with the current and saved values.
    fn init(&self) {
        // FIXME: bogus inits below
        self.ui.global::<Logic>().set_cryocooler_is_on(false);
        self.ui.global::<Logic>().set_transfer_valve_is_open(true);
        self.ui.global::<Logic>().set_pump_valve_is_open(false);

        // init buttons
        self.baking();
        self.light_switch();
        self.transfer_valve_set_open();
        self.pump_valve_set_open();
    }

    fn baking(&self) {
        self.ui.global::<Logic>().on_baking_enabled({
            let ui = self.ui.as_weak();
            move |val| {
                let ui = ui.unwrap();
                let baking_state = match val {
                    true => {
                        let baking_time = ui.global::<Logic>().get_baking_time();
                        let time_sec = baking_time.hours * 3600
                            + baking_time.minutes * 60
                            + baking_time.seconds;
                        BakingState::On {
                            time_sec: time_sec as u64,
                        }
                    }
                    false => BakingState::Off,
                };
                send_cntrl_cmd_now(ControllerCommands::Baking(baking_state.clone()));
            }
        });
    }

    fn light_switch(&self) {
        self.ui.global::<Logic>().on_light_switch({
            move |val| {
                let light_stat = match val {
                    true => LightState::On,
                    false => LightState::Off,
                };
                send_cntrl_cmd_now(ControllerCommands::Light(light_stat));
            }
        });
    }

    fn pump_valve_set_open(&self) {
        let ui = self.ui.as_weak();
        let conf = Arc::clone(&self.conf);
        self.ui.global::<Logic>().on_pump_valve_set_open({
            move |val| {
                let vst = match val {
                    true => ValveState::Open,
                    false => ValveState::Closed,
                };
                send_cntrl_cmd_now(ControllerCommands::PumpValve(vst));
                ui.unwrap().global::<Logic>().set_pump_valve_is_open(val);
            }
        });
    }

    fn transfer_valve_set_open(&self) {
        let ui = self.ui.as_weak();
        let conf = Arc::clone(&self.conf);
        self.ui.global::<Logic>().on_transfer_valve_set_open({
            move |val| {
                let vst = match val {
                    true => ValveState::Open,
                    false => ValveState::Closed,
                };
                send_cntrl_cmd_now(ControllerCommands::TransferValve(vst));
                ui.unwrap()
                    .global::<Logic>()
                    .set_transfer_valve_is_open(val);
            }
        });
    }
}

/// Handler for GUI-related command.
///
/// Examples here are: admin mode, exit, etc.
struct GuiCommandHandler {
    ui: AppWindow,
    conf: Arc<Mutex<PrgConfig>>,
}

impl GuiCommandHandler {
    fn new(
        ui: Weak<AppWindow>,
        conf: Arc<Mutex<PrgConfig>>,
    ) -> Self {
        let sf = Self {
            ui: ui.unwrap(),
            conf,
        };
        sf.init();
        sf
    }

    fn init(&self) {
        // set the sample names from config
        let model = self.ui.global::<Logic>().get_sample_model();
        let curr_samples = {
            self.conf.lock().expect("Poisoned").get_samples()
        };
        for (idx, (pos, name)) in curr_samples.into_iter().enumerate() {
            model.set_row_data(idx, (name.into(), pos.into()));
        }


        // buttons
        self.test_button(); // FIXME: Delete
        self.admin_mode();
        self.close_button();
        self.edit_sample_name();
    }

    // FIXME: Delete
    fn test_button(&self) {
        self.ui
            .global::<Logic>()
            .set_test_button_text("Toggle VCT handshake".into());
        let ui = self.ui.as_weak();
        self.ui.global::<Logic>().on_test_button_pressed({
            move || {
                let ui = ui.unwrap();
                let new_state = !ui.global::<Logic>().get_test_button_state();
                let vct_state = if new_state {
                    VctHandshake::Ready
                } else {
                    VctHandshake::NotReady
                };
                send_cntrl_cmd_now(ControllerCommands::VctHandshake(vct_state));
                ui.global::<Logic>().set_test_button_state(new_state);
            }
        });
    }

    fn admin_mode(&self) {
        self.ui.global::<Logic>().on_enter_admin_mode({
            let ui = self.ui.as_weak();
            let conf = Arc::clone(&self.conf);
            move || {
                let ui = ui.unwrap();
                if ui.global::<Logic>().get_admin_mode() {
                    ui.global::<Logic>().set_admin_mode(false);
                } else {
                    let keypad = Keypad::new().unwrap();
                    keypad.show().unwrap();

                    keypad.global::<KeypadLogic>().on_cancel_pressed({
                        let keypad = keypad.as_weak();
                        move || {
                            keypad.unwrap().hide().unwrap();
                        }
                    });

                    keypad.global::<KeypadLogic>().on_ok_pressed({
                        let keypad = keypad.as_weak();
                        let ui = ui.as_weak();
                        let pin_expected = conf.lock().expect("Poisoned").get_admin_pin();
                        move |code| {
                            if code.as_str() == pin_expected.as_str() {
                                ui.unwrap().global::<Logic>().set_admin_mode(true);
                            }
                            keypad.unwrap().hide().unwrap();
                        }
                    });
                }
            }
        });
    }

    fn close_button(&self) {
        self.ui.global::<Logic>().on_close_program({
            move || {
                slint::quit_event_loop().unwrap();
            }
        });
    }

    fn edit_sample_name(&self) {
        self.ui.global::<Logic>().on_edit_sample_name({
            let ui = self.ui.as_weak();
            let cfg = Arc::clone(&self.conf);
            move |pos, name| {
                let message = format!("Edit name of sample at position {}", pos.clone());
                let (tx, mut rx) = mpsc::channel(1);
                let kb = KeyboardInput::new(tx);
                kb.get_text_input(message.as_str(), name);

                let cfg = Arc::clone(&cfg);
                let ui = ui.clone();
                let fut = async move {
                    let answer = rx.recv().await;
                    if let Some(ans) = answer {
                        if let Ok(idx) = cfg.lock().expect("Poisoned").update_sample(&pos, &ans) {
                            let model = ui.unwrap().global::<Logic>().get_sample_model();
                            model.set_row_data(idx, (ans, pos));
                        } else {
                            eprintln!("Failed to update sample name: no position {pos}");
                        }
                    } else {
                        eprintln!("Failed to update sample name: no answer from keyboard");
                    };
                };

                slint::spawn_local(async_compat::Compat::new(fut)).unwrap();

                //         }
                //     });
            }
        });
    }
}

/// Command handler for connected instruments.
struct InstrumentCommandHandler {
    ui: AppWindow,
    conf: Arc<Mutex<PrgConfig>>,
}

impl InstrumentCommandHandler {
    fn new(
        ui: Weak<AppWindow>,
        conf: Arc<Mutex<PrgConfig>>,
    ) -> Self {
        let sf = Self {
            ui: ui.unwrap(),
            conf,
        };
        sf.init();
        sf
    }

    fn init(&self) {
        self.cryocooler_set_on();

        // FIXME: bogus inits below
        self.ui
            .global::<Logic>()
            .set_transfer_pressure(format!("{:.2E} mbar", 0.3).into());
        // set chamber pressure scientifically formatted
        self.ui
            .global::<Logic>()
            .set_chamber_pressure(format!("{:.2E} mbar", 0.0001234).into());
        self.ui.global::<Logic>().set_cryocooler_is_on(false);
    }

    fn cryocooler_set_on(&self) {
        self.ui.global::<Logic>().on_cryocooler_set_on({
            move |val| {
                println!("Cryocooler on: {}", val); // TODO:
            }
        });
    }
}

struct KeyboardInput {
    tx: mpsc::Sender<SharedString>,
}

impl KeyboardInput {
    fn new(tx: mpsc::Sender<SharedString>) -> Self {
        Self { tx }
    }

    fn get_text_input(&self, message: &str, initial_text: SharedString) {
        let keyboard = Keyboard::new().unwrap();
        keyboard
            .global::<KeyboardLogic>()
            .set_message(message.into());
        keyboard
            .global::<KeyboardLogic>()
            .set_text_entered(initial_text.clone());

        keyboard.show().unwrap();

        keyboard.global::<KeyboardLogic>().on_cancel_pressed({
            let keyboard = keyboard.as_weak();
            let tx = self.tx.clone();
            move || {
                let initial_text = initial_text.clone();
                let tx = tx.clone();
                let fut = async move {
                    tx.send(initial_text.clone()).await.unwrap();
                };
                slint::spawn_local(async_compat::Compat::new(fut)).unwrap();
                keyboard.unwrap().hide().unwrap();
            }
        });

        keyboard.global::<KeyboardLogic>().on_ok_pressed({
            let keyboard = keyboard.as_weak();
            let tx = self.tx.clone();
            move |new_text| {
                let tx = tx.clone();
                let fut = async move {
                    tx.send(new_text).await.unwrap();
                };
                slint::spawn_local(async_compat::Compat::new(fut)).unwrap();
                keyboard.unwrap().hide().unwrap();
            }
        });

        keyboard.global::<KeyboardLogic>().on_backspace_pressed({
            let keyboard = keyboard.as_weak();
            move || {
                let keyboard = keyboard.unwrap();
                let text = keyboard.global::<KeyboardLogic>().get_text_entered();
                // cut the last character off the string
                let text = match text.as_str().char_indices().next_back() {
                    Some((idx, _)) => &text[0..idx],
                    None => text.as_str(),
                };
                keyboard
                    .global::<KeyboardLogic>()
                    .set_text_entered(text.into());
            }
        });
    }
}
