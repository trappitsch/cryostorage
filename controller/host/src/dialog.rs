//! Create slint dialogs that display with the proper styling.

use std::fmt::Display;

use anyhow::Result;
use slint::ComponentHandle;

use crate::app::ErrorDialog;

/// Show a generic error dialog with the title Error and then the given message.
pub fn show_error_dialog<D: Display>(msg: D) -> Result<()> {
    let dialog = ErrorDialog::new()?;

    let msg = format!("{}", msg);
    dialog.set_error_message(msg.into());

    let dlg = dialog.as_weak();
    dialog.on_close_pressed(move || {
        dlg.unwrap().window().hide().unwrap();
    });

    dialog.show()?;
    Ok(())
}
