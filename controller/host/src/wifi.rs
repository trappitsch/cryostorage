//! Deal with the current status of WiFi and turn it on/off.
//!
//! `rfkill` must be installed for this to work!

use std::process::Command;

use anyhow::{Result, bail};

#[derive(Debug, Copy, Clone)]
pub enum WifiStatus {
    On,
    Off,
}

/// Get the soft status of `wlan` from rfkill.
pub fn get_wifi_status() -> Result<WifiStatus> {
    let output = Command::new("rfkill").args(["list", "wlan"]).output()?;

    if !output.status.success() {
        bail!("Getting WiFi status failed.")
    }

    let out = String::from_utf8_lossy(output.stdout.as_slice());

    if out.contains("Soft blocked: no") {
        Ok(WifiStatus::On)
    } else {
        Ok(WifiStatus::Off)
    }
}

impl From<WifiStatus> for bool {
    fn from(value: WifiStatus) -> Self {
        match value {
            WifiStatus::On => true,
            WifiStatus::Off => false,
        }
    }
}

impl From<bool> for WifiStatus {
    fn from(value: bool) -> Self {
        if value {
            WifiStatus::On
        } else {
            WifiStatus::Off
        }
    }
}

/// Set the wlan status.
pub fn set_wifi_status(st: WifiStatus) -> Result<()> {
    let arg0 = match st {
        WifiStatus::On => "unblock",
        WifiStatus::Off => "block",
    };
    let out = Command::new("rfkill").args([arg0, "wlan"]).output()?;

    if !out.status.success() {
        bail!("Could not set WiFi Status.")
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::assert_matches;

    #[test]
    fn test_wifi_toggling() {
        assert!(set_wifi_status(WifiStatus::Off).is_ok());
        assert_matches!(get_wifi_status().unwrap(), WifiStatus::Off);

        assert!(set_wifi_status(WifiStatus::On).is_ok());
        assert_matches!(get_wifi_status().unwrap(), WifiStatus::On);
    }
}
