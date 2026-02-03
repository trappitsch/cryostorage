use std::{
    env, fs,
    sync::{Arc, Mutex},
};

use tokio::sync::{OnceCell, broadcast, mpsc, oneshot};

use crate::{
    controller::{ControllerCommands, start_controller_tasks}, instruments::instruments_task, logger::{LogHandler, LogMessage}, status::InstrumentStatus
};

mod app;
mod connections;
mod controller;
mod instruments;
mod logger;
mod prg_config;
mod samples;
mod status;
mod vacuum_history;

pub const CONFIG_FOLDER: &str = ".cryostorage";
pub const LOG_LEVEL_DISPLAY: logger::Level = logger::Level::Warning;

pub static HALT_SENDER: OnceCell<broadcast::Sender<()>> = OnceCell::const_new();
pub static LOG_SENDER: OnceCell<mpsc::Sender<LogMessage>> = OnceCell::const_new();
pub static CONTROLLER_COMMAND_SENDER: OnceCell<mpsc::Sender<ControllerCommands>> =
    OnceCell::const_new();

#[tokio::main]
async fn main() {
    // Create the configuration folder if it doesn't exist
    let conf_folder_pth = env::home_dir()
        .expect("Home directory must be known")
        .join(CONFIG_FOLDER);
    fs::create_dir_all(&conf_folder_pth).expect("Could not create config folder");

    // config
    let conf = Arc::new(Mutex::new(prg_config::PrgConfig::try_new().unwrap()));

    // status of instrument
    let inst_status = Arc::new(Mutex::new(InstrumentStatus::new()));

    // Shutdown signal and receiver for tasks
    let (tx_halt, _) = broadcast::channel(1);
    HALT_SENDER.set(tx_halt.clone()).expect("Uninitialized");

    // LogHandler
    let (tx_log, rx_log) = mpsc::channel(128);
    let (tx_ui_set, rx_ui_set) = oneshot::channel();
    let log_handler = LogHandler::new(rx_log);
    LOG_SENDER.set(tx_log).expect("Uninitialized");

    let log_handler_listen = tokio::spawn(logger::log_handler_task(log_handler, rx_ui_set));

    // comms for controller task
    let (tx_ctrl, rx_ctrl) = mpsc::channel(32);
    CONTROLLER_COMMAND_SENDER
        .set(tx_ctrl.clone())
        .expect("Uninitialized");

    // controller
    let controller_config = conf
        .lock()
        .expect("Locking config must work")
        .get_controller_config();
    let (cntrl_tsk, cntrl_bc_listen) =
        start_controller_tasks(controller_config, Arc::clone(&inst_status), rx_ctrl).await;

    // instruments monitoring task
    let instr_tsk = tokio::spawn(instruments_task(Arc::clone(&conf), Arc::clone(&inst_status)));

    // start the app
    match app::app_main(Arc::clone(&conf), Arc::clone(&inst_status), tx_ui_set) {
        Ok(_) => {
            tx_halt.send(()).unwrap();
            let _ = tokio::join!(cntrl_tsk, cntrl_bc_listen, instr_tsk, log_handler_listen);
            println!("App exited normally");
        }
        Err(e) => eprintln!("App exited with error: {}", e),
    }
}
