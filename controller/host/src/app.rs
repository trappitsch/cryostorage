//! This module contains the complete GUI logic for the application.

use std::{
    error::Error,
    sync::{Arc, Mutex, mpsc},
};

use icd::LightState;
use slint::{Model, Weak};

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
        self.ui
            .global::<Logic>()
            .set_sample_model(samples.into());

        // FIXME: bogus inits below
        self.ui
            .global::<Logic>()
            .set_transfer_pressure(format!("{:.2E} mbar", 0.3).into());
        // set chamber pressure scientifically formatted
        self.ui
            .global::<Logic>()
            .set_chamber_pressure(format!("{:.2E} mbar", 0.0001234).into());
        self.ui.global::<Logic>().set_cryocooler_is_on(false);
        self.ui
            .global::<Logic>()
            .set_transfer_valve_is_open(true);

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
                let dialog = SampleEditDialog::new().unwrap();
                dialog.set_sample_position(pos);
                dialog.set_sample_name(name);
                dialog.show().unwrap();

                dialog.on_cancel_clicked({
                    let dialog = dialog.as_weak();
                    move || {
                        dialog.unwrap().hide().unwrap();
                    }
                });

                dialog.on_ok_clicked({
                    let ui = ui.clone();
                    let dialog = dialog.as_weak();
                    let cfg = Arc::clone(&cfg);
                    move |new_name| {
                        println!("New name: {}", new_name); // TODO
                        let pos = dialog.unwrap().get_sample_position();

                        let Ok(idx) = cfg.lock().expect("Poisoned").update_sample(&pos, &new_name)
                        else {
                            eprintln!("Failed to update sample name");
                            return;
                        };

                        let model = ui.unwrap().global::<Logic>().get_sample_model();
                        model.set_row_data(idx, (new_name, pos));
                        dialog.unwrap().hide().unwrap();
                    }
                });
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
    }

    fn close_button(&self) {
        self.ui.global::<Logic>().on_close_program({
            move || {
                slint::quit_event_loop().unwrap();
            }
        });
    }
}
