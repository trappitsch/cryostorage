//! Module to handle saving and loading configuration files.

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::{
    CONFIG_FOLDER,
    controller::ControllerConfig,
    instruments::{
        cryocooler::CryoCoolerConfig, hi_cube::PfeifferHiCubeConf, ion_pump::IonPumpConfig,
        lakeshore_temp::LakeshoreTempConfig, omnicontrol::OmniControlConfig,
    },
    logger::{LogMessage, send_log_message_now},
    samples::Samples,
};

pub const CONFIG_FNAME: &str = "cryostorage_config.ron";
pub const CONFIG_OLD_FOLDER: &str = "config_history";

#[derive(Debug, Serialize, Deserialize)]
pub struct PrgConfig {
    fname: PathBuf,
    admin_pin: String,
    agilent_ion_pump: IonPumpConfig,
    controller_config: ControllerConfig,
    pfeiffer_hicube: PfeifferHiCubeConf,
    pfeiffer_omnicontrol: OmniControlConfig,
    samples: Samples,
    limits: InstrumentLimits,
    lakeshore_temperature: LakeshoreTempConfig,
    suntel_cryocooler: CryoCoolerConfig,
}

impl PrgConfig {
    /// Create a new PrgConfig instance with saved or default values.
    ///
    /// The configuration folder is created, if it does not exist, in `main.rs`.
    pub fn try_new() -> Result<Self> {
        // Create the configuration folder if it doesn't exist
        let conf_folder_pth = env::home_dir()
            .expect("Home directory must be known")
            .join(CONFIG_FOLDER);
        fs::create_dir_all(&conf_folder_pth).expect("Could not create config folder");

        // Create the folder for the old config files, if it doesn't exist
        let old_conf_folder_pth = conf_folder_pth.join(CONFIG_OLD_FOLDER);
        fs::create_dir_all(&old_conf_folder_pth).expect("Could not create old config folder");

        let fname = conf_folder_pth.join(CONFIG_FNAME);

        let mut ret_self = Self {
            fname,
            admin_pin: String::from("1234"),
            agilent_ion_pump: IonPumpConfig::default(),
            controller_config: ControllerConfig::default(),
            pfeiffer_hicube: PfeifferHiCubeConf::default(),
            pfeiffer_omnicontrol: OmniControlConfig::default(),
            samples: Samples::new(),
            limits: InstrumentLimits::default(),
            lakeshore_temperature: LakeshoreTempConfig::default(),
            suntel_cryocooler: CryoCoolerConfig::default(),
        };

        if let Err(e) = ret_self.load_from_file() {
            eprintln!("Error loading config file: {e}");
        };

        Ok(ret_self)
    }

    fn load_from_file(&mut self) -> Result<()> {
        if let Ok(content) = fs::read_to_string(&self.fname)
            && let Ok(cont_ron) = ron::de::from_str::<PrgConfig>(&content)
        {
            *self = cont_ron;
            Ok(())
        } else {
            self.save_to_file()?;
            Err(anyhow!(
                "Config file likely invalid or not found, saving a new one with default values."
            ))
        }
    }

    fn save_to_file(&self) -> Result<()> {
        // backup the previous config file with same name and timestamp if it exists
        if self.fname.exists() {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d-%H:%M:%S");
            let stem = self
                .fname
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cryostorage_config");
            let ext = self
                .fname
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("ron");
            let backup_fname = self
                .fname
                .parent()
                .expect("Config file has a parent folder")
                .join(CONFIG_OLD_FOLDER)
                .join(format!("{timestamp}_{stem}.{ext}"));
            if let Err(e) = fs::copy(&self.fname, backup_fname) {
                send_log_message_now(LogMessage::new_error(&format!(
                    "Failed to backup config file: {e}"
                )));
            };
        };

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

    /// Get a clone of the cryocooler configuration.
    pub fn get_cryocooler_config(&self) -> CryoCoolerConfig {
        self.suntel_cryocooler.clone()
    }

    /// Get a clone of the ion pump configuration.
    pub fn get_ion_pump_config(&self) -> IonPumpConfig {
        self.agilent_ion_pump.clone()
    }

    /// Get a clone of the Omnicontrol configuration.
    pub fn get_omnicontrol_config(&self) -> OmniControlConfig {
        self.pfeiffer_omnicontrol.clone()
    }

    /// Get a clone of the lakeshore temperature configuration.
    pub fn get_lakeshore_temp_config(&self) -> LakeshoreTempConfig {
        self.lakeshore_temperature.clone()
    }

    /// Get a clone of the HiCube configuration.
    pub fn get_pfeiffer_hicube_config(&self) -> PfeifferHiCubeConf {
        self.pfeiffer_hicube.clone()
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
