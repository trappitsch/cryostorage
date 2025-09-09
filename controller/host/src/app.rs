//! This module contains the complete GUI logic for the application.

use std::{error::Error, sync::mpsc};

use icd::LightState;
use slint::Weak;

use crate::controller::ControllerCommands;

slint::include_modules!();

pub fn app_main(tx: mpsc::Sender<ControllerCommands>) -> Result<(), Box<dyn Error>> {
    // slint
    let ui = AppWindow::new()?;

    // initialize the different screens
    let _home_screen = HomeScreen::new(ui.as_weak(), tx.clone());

    ui.show()?;

    ui.run()?;
    Ok(())
}

struct HomeScreen {
    ui: AppWindow,
    tx: mpsc::Sender<ControllerCommands>,
}

impl HomeScreen {
    /// Initialize all switches
    fn new(ui: Weak<AppWindow>, tx: mpsc::Sender<ControllerCommands>) -> Self {
        let hs = Self { ui: ui.unwrap(), tx };
        hs.light_switch();
        hs
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
}

