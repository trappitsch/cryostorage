//! Implement log rotation functionality.
//!
//! This module provides two functions for log-like file rotation:
//!
//! `rotate`:
//! - Takes a file path argument.
//! - Errors if file does not exist
//! - If any files with appended numbers exist in ARCHIVE folder, they will be incremented by one
//!   if the number is smaller than NUM_LOGS_TO_KEEP.
//! - If the number is equal to NUM_LOGS_TO_KEEP, the file will be overwritten.
//! - The file from the config folder will be moved to the archive folder with appended 1.

use std::fs;

use anyhow::{Result, bail};

use crate::{ARCHIVE_FOLDER, CONFIG_FOLDER};

pub const NUM_LOGS_TO_KEEP: usize = 5;

pub fn rotate(fname: &str) -> Result<()> {
    let config_folder = CONFIG_FOLDER.get().expect("Config folder is initialized");
    let archive_folder = ARCHIVE_FOLDER.get().expect("Archive folder is initialized");

    let fname_conf = config_folder.join(fname);
    if !fname_conf.exists() {
        bail!("File to rotate does not exist: {:?}", fname_conf);
    }

    // rotate existing archived files up to NUM_LOGS_TO_KEEP
    for it in (1..NUM_LOGS_TO_KEEP).rev() {
        let fn_tmp_old = archive_folder.join(format!("{}.{it}", fname));
        if fn_tmp_old.exists() {
            let fn_tmp_new = archive_folder.join(format!("{}.{}", fname, it + 1));
            fs::rename(&fn_tmp_old, &fn_tmp_new)?;
        }
    }

    // move the file to the archive folder with appended 1
    let fn_new = archive_folder.join(format!("{}.1", fname));
    fs::rename(&fname_conf, &fn_new)?;

    Ok(())
}
