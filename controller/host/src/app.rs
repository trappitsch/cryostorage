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
        hs.light_switch();
        hs.cryocooler_set_on();
        hs.edit_sample_name();
        hs.transfer_valve_set_open();

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
            .global::<SampleLogic>()
            .set_sample_model(samples.into());

        // FIXME: bogus inits below
        self.ui
            .global::<HomeLogic>()
            .set_transfer_pressure(format!("{:.2E} mbar", 0.3).into());
        // set chamber pressure scientifically formatted
        self.ui
            .global::<HomeLogic>()
            .set_chamber_pressure(format!("{:.2E} mbar", 0.0001234).into());
        self.ui.global::<HomeLogic>().set_cryocooler_is_on(false);
        self.ui
            .global::<HomeLogic>()
            .set_transfer_valve_is_open(true);
    }

    fn light_switch(&self) {
        let tx = self.tx.clone();
        self.ui.global::<HomeLogic>().on_light_switch({
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
        self.ui.global::<HomeLogic>().on_cryocooler_set_on({
            move |val| {
                println!("Cryocooler on: {}", val); // TODO
            }
        });
    }

    fn edit_sample_name(&self) {
        self.ui.global::<SampleLogic>().on_edit_sample_name({
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

                        let model = ui.unwrap().global::<SampleLogic>().get_sample_model();
                        model.set_row_data(idx, (new_name, pos));
                        dialog.unwrap().hide().unwrap();
                    }
                });
            }
        });
    }

    fn transfer_valve_set_open(&self) {
        self.ui.global::<HomeLogic>().on_transfer_valve_set_open({
            move |val| {
                println!("Transfer valve open: {}", val); // TODO
            }
        });
    }
}
