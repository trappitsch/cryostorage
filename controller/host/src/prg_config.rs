//! Module to handle saving and loading configuration files.

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{controller::ControllerConfig, samples::Samples};

pub const CONFIG_FNAME: &str = "cryostorage_config.ron";

#[derive(Debug, Serialize, Deserialize)]
pub struct PrgConfig {
    fname: PathBuf,
    controller_config: ControllerConfig,
    samples: Samples,
}

impl PrgConfig {
    /// Create a new PrgConfig instance with saved or default values.
    pub fn try_new() -> Result<Self> {
        let fname = env::home_dir()
            .expect("Home directory must be known")
            .join(".config")
            .join("cryostorage");
        fs::create_dir_all(&fname)?;
        let fname = fname.join(CONFIG_FNAME);

        let mut ret_self = Self {
            fname,
            controller_config: ControllerConfig::default(),
            samples: Samples::new(),
        };

        ret_self.load_from_file();
        ret_self.save_to_file()?; // FIXME: Remove this line later

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

    /// Get a clone of the controller configuration.
    pub fn get_controller_config(&self) -> ControllerConfig {
        self.controller_config.clone()
    }

    /// Get a clone of the samples.
    pub fn get_samples(&self) -> Samples {
        self.samples.clone()
    }

    /// Update the samples and save to file.
    pub fn update_samples(&mut self, samples: Samples) -> Result<()> {
        self.samples = samples;
        self.save_to_file()
    }
}
