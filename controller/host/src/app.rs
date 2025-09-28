//! This module contains the complete GUI logic for the application.

use std::{
    error::Error,
    sync::{Arc, Mutex, mpsc},
};

use icd::LightState;
use slint::{Model, SharedString, Weak};
use tokio::sync::mpsc as tokio_mpsc;

use crate::{controller::ControllerCommands, prg_config::PrgConfig};

slint::include_modules!();

pub fn app_main(
    tx: mpsc::Sender<ControllerCommands>,
    conf: Arc<Mutex<PrgConfig>>,
) -> Result<(), Box<dyn Error>> {
    // slint
    let ui = AppWindow::new()?;

    // initialize the different screens
    let _home_screen = HomeScreen::new(ui.as_weak(), tx.clone(), Arc::clone(&conf));
    let _settings_screen = SettingsScreen::new(ui.as_weak(), tx.clone());

    ui.show()?;

    ui.run()?;
    Ok(())
}

struct HomeScreen {
    conf: Arc<Mutex<PrgConfig>>,
    tx: mpsc::Sender<ControllerCommands>,
    ui: AppWindow,
}

impl HomeScreen {
    /// Initialize all switches
    fn new(
        ui: Weak<AppWindow>,
        tx: mpsc::Sender<ControllerCommands>,
        conf: Arc<Mutex<PrgConfig>>,
    ) -> Self {
        let hs = Self {
            conf,
            tx,
            ui: ui.unwrap(),
        };
        hs.init();

        hs
    }

    /// Initialize the screen with the current and saved values.right
    fn init(&self) {
        let samples = self
            .conf
            .lock()
            .expect("Poisoned")
            .get_samples()
            .get_for_slint();
        self.ui.global::<Logic>().set_sample_model(samples.into());

        // FIXME: bogus inits below
        self.ui
            .global::<Logic>()
            .set_transfer_pressure(format!("{:.2E} mbar", 0.3).into());
        // set chamber pressure scientifically formatted
        self.ui
            .global::<Logic>()
            .set_chamber_pressure(format!("{:.2E} mbar", 0.0001234).into());
        self.ui.global::<Logic>().set_cryocooler_is_on(false);
        self.ui.global::<Logic>().set_transfer_valve_is_open(true);

        // init buttons
        self.light_switch();
        self.cryocooler_set_on();
        self.edit_sample_name();
        self.transfer_valve_set_open();
    }

    fn light_switch(&self) {
        let tx = self.tx.clone();
        self.ui.global::<Logic>().on_light_switch({
            move |val| {
                let light_stat = match val {
                    true => LightState::On,
                    false => LightState::Off,
                };
                tx.send(ControllerCommands::Light(light_stat)).unwrap();
            }
        });
    }

    fn cryocooler_set_on(&self) {
        self.ui.global::<Logic>().on_cryocooler_set_on({
            move |val| {
                println!("Cryocooler on: {}", val); // TODO
            }
        });
    }

    fn edit_sample_name(&self) {
        self.ui.global::<Logic>().on_edit_sample_name({
            let ui = self.ui.as_weak();
            let cfg = Arc::clone(&self.conf);
            move |pos, name| {
                let message = format!("Edit name of sample at position {}", pos.clone());
                let (tx, mut rx) = tokio_mpsc::channel(1);
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
                            eprintln!("Failed to update sample position");
                        };
                    }
                };

                slint::spawn_local(async_compat::Compat::new(fut)).unwrap();

                //         }
                //     });
            }
        });
    }

    fn transfer_valve_set_open(&self) {
        self.ui.global::<Logic>().on_transfer_valve_set_open({
            move |val| {
                println!("Transfer valve open: {}", val); // TODO
            }
        });
    }
}

struct SettingsScreen {
    ui: AppWindow,
    tx: mpsc::Sender<ControllerCommands>,
}

impl SettingsScreen {
    fn new(ui: Weak<AppWindow>, tx: mpsc::Sender<ControllerCommands>) -> Self {
        let ss = Self {
            tx,
            ui: ui.unwrap(),
        };
        ss.init();

        ss
    }

    fn init(&self) {
        self.close_button();
        self.test_button();
    }

    // FIXME: Delete
    fn test_button(&self) {
        let ui = self.ui.as_weak();
        self.ui.global::<Logic>().on_test_button_pressed({
            move || {
                let ui = ui.unwrap();
                let text = ui.global::<Logic>().get_test_button_text();
                println!("Test button text: {}", text);
                let (tx, mut rx) = tokio_mpsc::channel(1);
                let kb = KeyboardInput::new(tx);
                kb.get_text_input("asdf", text);
                let fut = async move {
                    let result = rx.recv().await;
                    if let Some(res) = result {
                        ui.global::<Logic>().set_test_button_text(res);
                    }
                };
                slint::spawn_local(async_compat::Compat::new(fut)).unwrap();
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
}

struct KeyboardInput {
    tx: tokio_mpsc::Sender<SharedString>,
}

impl KeyboardInput {
    fn new(tx: tokio_mpsc::Sender<SharedString>) -> Self {
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
