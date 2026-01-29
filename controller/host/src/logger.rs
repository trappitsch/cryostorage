//! Add a simple logging utility that logs messages to the GUI.
//!
//! These messages should also ultimately be saved to a log file whenever a new message shows up.

use std::{env, fmt::Display, fs::OpenOptions, io::Write, path::PathBuf};

use chrono::{DateTime, Local};
use slint::{ComponentHandle, Model, Weak};
use tokio::sync::{mpsc, oneshot};

use crate::{
    CONFIG_FOLDER, LOG_LEVEL_DISPLAY,
    app::{AppWindow, Logic},
};

/// The severity level of a log message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    // Informational message, the most verbose.
    Info,
    // Warning level: no error occured but something is off.
    Warning,
    // An error occured. An unwrap would have panicked here!
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Info => write!(f, "INFO"),
            Level::Warning => write!(f, "WARN"),
            Level::Error => write!(f, "ERR "),
        }
    }
}

/// A log message.
#[derive(Debug, Clone)]
pub struct LogMessage {
    timestamp: DateTime<Local>,
    pub level: Level,
    message: String,
}

impl LogMessage {
    /// Create a new log message.
    fn new(level: Level, message: &str) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            message: message.to_string(),
        }
    }

    /// Create a new info log message.
    pub fn new_info(message: &str) -> Self {
        Self::new(Level::Info, message)
    }

    /// Create a new warning log message.
    pub fn new_warning(message: &str) -> Self {
        Self::new(Level::Warning, message)
    }

    /// Create a new error log message.
    pub fn new_error(message: &str) -> Self {
        Self::new(Level::Error, message)
    }
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}: {}] {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.level,
            self.message
        )
    }
}

/// A handler for log messages.
pub struct LogHandler {
    /// The log message receiver.
    rx: mpsc::Receiver<LogMessage>,
    /// Weak reference to the UI for upgrading. Has to be set after initializing the LogHandler.
    ui: Option<Weak<AppWindow>>,
    /// File name of log file, will be stored in CONFIG_FOLDER.
    log_file: PathBuf,
}

impl LogHandler {
    /// Create a new LogHandler.
    pub fn new(rx: mpsc::Receiver<LogMessage>) -> Self {
        let log_file = env::home_dir()
            .expect("Home directory must be known")
            .join(CONFIG_FOLDER)
            .join("cryostorage.log");

        Self {
            rx,
            ui: None,
            log_file,
        }
    }

    /// Allows setting the UI once the UI is created.
    pub fn set_ui(&mut self, ui: Weak<AppWindow>) {
        self.ui = Some(ui);
    }

    /// Add a new log message to the handler and process it.
    pub fn add_new_message(&mut self, msg: LogMessage) {
        // Save the message to the file.
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.log_file)
        {
            Ok(mut fout) => {
                if writeln!(&mut fout, "{}", msg).is_err() {
                    eprintln!("Could not write to log file {:?}", self.log_file);
                }
            }
            Err(e) => {
                eprintln!(
                    "Could not open log file {:?} for writing: {}",
                    self.log_file, e
                );
            }
        }

        // Save message to buffer if required.
        if msg.level >= LOG_LEVEL_DISPLAY {
            self.update_ui(msg);
        }
    }

    /// Update the UI, if it is already set. Otherwise just do nothing.
    pub fn update_ui(&self, msg: LogMessage) {
        if let Some(ui) = &self.ui {
            ui.upgrade_in_event_loop(move |ui| {
                let model = ui.global::<Logic>().get_log_messages();
                let model = model
                    .as_any()
                    .downcast_ref::<slint::VecModel<slint::SharedString>>()
                    .unwrap();
                model.insert(0, slint::SharedString::from(msg.to_string()));
            })
            .unwrap();
        }
    }
}

/// Async handler that runs the LogHandler.
///
/// Shuts down when a shutdown signal is received.
pub async fn log_handler_task(
    mut log_handler: LogHandler,
    rx_ui_set: oneshot::Receiver<Weak<AppWindow>>,
) {
    // Wait for the UI to be set
    if let Ok(ui) = rx_ui_set.await {
        log_handler.set_ui(ui);
    }

    let mut rx_shutdown = crate::HALT_SENDER.get().unwrap().subscribe();
    loop {
        tokio::select! {
            message = log_handler.rx.recv() => {
                if let Some(msg) = message {
                    dbg!(&msg);
                    log_handler.add_new_message(msg);
                }
            }
            _ = rx_shutdown.recv() => {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convince myself how Ordering in enums work.
    #[test]
    fn log_level_ordering() {
        let info = Level::Info;
        let warning = Level::Warning;
        let error = Level::Error;

        assert!(info == Level::Info);
        assert!(warning == Level::Warning);
        assert!(error == Level::Error);

        assert!(info < warning);
        assert!(warning < error);
        assert!(info < error);
    }
}
