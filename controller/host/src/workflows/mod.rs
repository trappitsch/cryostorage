//! Module to handle workflows and system safety checks before executing commands.
//!
//! On commands from the UI, the functions in this module will be called to check if the command
//! can be executed safely and, if it can, will execute the command. Otherwise, error dialogs will
//! be shown in the UI.
//!
//! The module itself handles receiving workflow commands and running them in a blocking or async
//! way, depending on the function.
//!
//! The submodules handle the actual workflows for different components and execute the commands by
//! sending using the adequate command senders.
//! Blocking tasks will either run the command and return `Ok(())`, or return an error with a
//! message to be shown to the user if they fail.
//! TODO: Async task description
//! - Block the UI?
//! - How do we prevent long running background tasks from being started multiple times?

use std::sync::{Arc, Mutex};

use icd::ValveState;
use slint::{ComponentHandle, Weak};
use tokio::sync::{OnceCell, mpsc};

use crate::{
    app::{AppWindow, Logic},
    dialog::show_error_dialog,
    logger::{LogMessage, send_log_message_now},
    prg_config::Authorizations,
    status::InstrumentStatus,
    workflows::valves::{close_transfer_valve, open_transfer_valve, open_pump_valve, close_pump_valve},
};

mod valves;

/// Sender for the controller commands.
pub static WORKFLOW_COMMAND_SENDER: OnceCell<mpsc::Sender<WorkflowCommands>> =
    OnceCell::const_new();

/// Commands for workflows.
///
/// These are the commands that the UI will send to activate the various workflows.
pub enum WorkflowCommands {
    TransferValve(ValveState),
    PumpValve(ValveState),
}

/// The workflow task: Listen to WorkflowCommands and execute them.
pub async fn workflow_task(
    inst_status: Arc<Mutex<InstrumentStatus>>,
    authorizations: Authorizations,
    ui: Weak<AppWindow>,
    mut rx_wf: mpsc::Receiver<WorkflowCommands>,
) {
    let mut rx_shutdown = crate::HALT_SENDER.get().unwrap().subscribe();

    loop {
        tokio::select! {
            Some(cmd) = rx_wf.recv() => {
                match cmd {
                    WorkflowCommands::TransferValve(state) => {
                        let button_value: bool;
                        let res = match state {
                            ValveState::Open => {
                                button_value = true;
                                let (p_ch, p_tr) = inst_status.lock().expect("Poisoned").get_pressures();
                                open_transfer_valve(&p_ch, &p_tr, &authorizations.open_valve)
                            }
                            ValveState::Closed => {
                                button_value = false;
                                let vct_state = inst_status.lock().expect("Poisoned").get_vct_state();
                                close_transfer_valve(&vct_state)
                            }
                            ValveState::Undefined => unreachable!("Can only open or close value in workflows."),
                        };
                        ui.upgrade_in_event_loop(move |ui| {
                            if let Err(e) = res {
                                if let Err(e) = show_error_dialog(e) {
                                    send_log_message_now(LogMessage::new_error(&format!(
                                        "Failed to show error dialog: {e}"
                                    )));
                                };
                            } else {
                                ui.global::<Logic>().set_transfer_valve_is_open(button_value);
                            }
                        }).expect("UI must be alive");
                    },
                    WorkflowCommands::PumpValve(state) => {
                        let button_value: bool;
                        let res = match state {
                            ValveState::Open => {
                                button_value = true;
                                let (p_ch, p_tr) = inst_status.lock().expect("Poisoned").get_pressures();
                                open_pump_valve(&p_ch, &p_tr, &authorizations.open_valve)
                            }
                            ValveState::Closed => {
                                button_value = false;
                                close_pump_valve()
                            }
                            ValveState::Undefined => unreachable!("Can only open or close value in workflows."),
                        };
                        ui.upgrade_in_event_loop(move |ui| {
                            if let Err(e) = res {
                                if let Err(e) = show_error_dialog(e) {
                                    send_log_message_now(LogMessage::new_error(&format!(
                                        "Failed to show error dialog: {e}"
                                    )));
                                };
                            } else {
                                ui.global::<Logic>().set_transfer_valve_is_open(button_value);
                            }
                        }).expect("UI must be alive");
                    }
                }
            }
            _ = rx_shutdown.recv() => {
                println!("Workflow task shutting down");
                break;
            }
        }
    }
}

/// Get a cone of the workflow command sender.
fn get_workflow_command_sender() -> mpsc::Sender<WorkflowCommands> {
    WORKFLOW_COMMAND_SENDER.get().expect("Unitialized").clone()
}

/// Convenience function to send a workflow command now.
///
/// An error here means that something aside from the permissions checks has gone wrong. This will
/// be logged as any other error.
pub fn send_workflow_command_now(cmd: WorkflowCommands) {
    let sender = get_workflow_command_sender();
    if let Err(e) = sender.try_send(cmd) {
        send_log_message_now(LogMessage::new_error(&format!(
            "Failed to send workflow command now: {e}"
        )));
    }
}
