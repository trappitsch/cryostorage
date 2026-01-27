//! Module to handle saving and loading configuration files.

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{CONFIG_FOLDER, controller::ControllerConfig, samples::Samples};

pub const CONFIG_FNAME: &str = "cryostorage_config.ron";

#[derive(Debug, Serialize, Deserialize)]
pub struct PrgConfig {
    fname: PathBuf,
    admin_pin: String,
    controller_config: ControllerConfig,
    samples: Samples,
    limits: InstrumentLimits,
}

impl PrgConfig {
    /// Create a new PrgConfig instance with saved or default values.
    pub fn try_new() -> Result<Self> {
        let fname = env::home_dir()
            .expect("Home directory must be known")
            .join(CONFIG_FOLDER);
        fs::create_dir_all(&fname)?;
        let fname = fname.join(CONFIG_FNAME);

        let mut ret_self = Self {
            fname,
            admin_pin: String::from("1234"),
            controller_config: ControllerConfig::default(),
            samples: Samples::new(),
            limits: InstrumentLimits::default(),
        };

        ret_self.load_from_file();
        ret_self.save_to_file()?; // FIXME: Remove this line later
        println!("Configuration loaded from {:?}", ret_self.fname);

        Ok(ret_self)
    }

    fn load_from_file(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.fname)
            && let Ok(cont_ron) = ron::de::from_str::<PrgConfig>(&content)
        {
            *self = cont_ron;
        }
    }

    fn save_to_file(&self) -> Result<()> {
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        let mut f = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.fname)?;
        writeln!(&mut f, "{}", content)?;
        Ok(())
    }

    /// Get a clone of the admin pin.
    pub fn get_admin_pin(&self) -> String {
        self.admin_pin.clone()
    }

    /// Get a clone of the controller configuration.
    pub fn get_controller_config(&self) -> ControllerConfig {
        self.controller_config.clone()
    }

    /// Get a clone of the samples.
    pub fn get_samples(&self) -> Samples {
        self.samples.clone()
    }

    /// Update the samples, save to file, and return index of updated entry.
    pub fn update_sample(&mut self, pos: &str, value: &str) -> Result<usize> {
        let res = self.samples.update_sample(pos, value);
        self.save_to_file()?;
        res
    }
}

/// Limits of the instrument.
///
/// These limits are used to ensure safe opearations and to prevent damage to the system and the
/// chamber. They are checked against with current values to allow or disallow certain operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentLimits {
    /// Maximum pressure allowable in main chamber to initiate a sample transfer.
    pub max_main_pressure_transfer: f64,
    /// Pressure difference threshold between main and pump to allow opening the pump valve. This
    /// is a factor between the two pressures.
    pub max_pressure_diff_pump_valve: f64,
}

impl Default for InstrumentLimits {
    fn default() -> Self {
        Self {
            max_main_pressure_transfer: 5e-8,
            max_pressure_diff_pump_valve: 10.0,
        }
    }
}
