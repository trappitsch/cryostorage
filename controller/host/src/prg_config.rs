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
    authorizations: Authorizations,
    agilent_ion_pump: IonPumpConfig,
    controller_config: ControllerConfig,
    pfeiffer_hicube: PfeifferHiCubeConf,
    pfeiffer_omnicontrol: OmniControlConfig,
    samples: Samples,
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
            authorizations: Authorizations::default(),
            agilent_ion_pump: IonPumpConfig::default(),
            controller_config: ControllerConfig::default(),
            pfeiffer_hicube: PfeifferHiCubeConf::default(),
            pfeiffer_omnicontrol: OmniControlConfig::default(),
            samples: Samples::new(),
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

    /// Get a clone of the authorizations.
    pub fn get_authorizations(&self) -> Authorizations {
        self.authorizations.clone()
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

/// As structure to provide certain authorizations and limits for the safe operation of the system.
///
/// These authorizations are used to ensure safe operations and to prevent damage to the system and the
/// chamber. They are checked against with current values to allow or disallow certain operations.
///
/// We add some `__doc_xxx` fields to the struct to be able to display some documentation on
/// certain variables. These are not used for the program and are solely for providing explanations
/// in the configuration file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authorizations{
    pub open_valve: OpenValveAuthorization,
    pub cryo_cooler: CryoCoolerAuthorization,
}

impl Default for Authorizations{
    fn default() -> Self {
        Self {
            open_valve: OpenValveAuthorization::default(),
            cryo_cooler: CryoCoolerAuthorization::default(),
        }
    }
}

/// Authorization limits for opening valves.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenValveAuthorization {
    __doc_valve_ratio: String,
    pub valve_ratio_range: SafeRangeLimits,
    __doc_low_pressure_limit: String,
    pub low_pressure_limit_mbar: f64,
}

impl Default for OpenValveAuthorization {
    fn default() -> Self {
        Self {
            __doc_valve_ratio: String::from("Authoization give if: lower_limit < Gauge1/Gauge2 < upper_limit."),
            valve_ratio_range: SafeRangeLimits {
                lower_limit: 0.001,
                upper_limit: 100.0,
            },
            __doc_low_pressure_limit: String::from("Authorization give independent of range if both gauges show pressure below the low_pressure_limit."),
            low_pressure_limit_mbar: 0.00001,
        }
    }
}

/// Authorization limits for the cryocooler.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CryoCoolerAuthorization {
    __doc_min_pressure_on: String,
    pub min_pressure_on_mbar: f64,
}

impl CryoCoolerAuthorization {
    pub fn default() -> Self {
        Self {
            __doc_min_pressure_on: String::from("Authorization to turn on the cryocooler is given if the pressure in the chamber is below this limit."),
            min_pressure_on_mbar: 0.00001,
        }
    }
}

/// Limits for ensuring defining safe ranges.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeRangeLimits {
    pub lower_limit: f64,
    pub upper_limit: f64,
}
