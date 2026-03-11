use std::sync::{Arc, Mutex};

use tokio::sync::{OnceCell, broadcast, mpsc, oneshot};

use crate::{
    controller::{ControllerCommands, start_controller_tasks},
    instruments::{
        InstrumentCommands,
        hi_cube::{HiCubeCommands, pfeiffer_hicube_task},
        instruments_task,
    },
    logger::{LogHandler, LogMessage, send_log_message},
    plots::{
        PressurePlotCommands, TemperaturePlotCommands, pressure_plot_task, temperature_plot_task,
    },
    status::InstrumentStatus,
};

mod app;
mod connections;
mod controller;
mod dialog;
mod instruments;
mod logger;
mod plots;
mod prg_config;
mod samples;
mod status;
mod workflows;

pub const CONFIG_FOLDER: &str = ".cryostorage";
pub const LOG_LEVEL_DISPLAY: logger::Level = logger::Level::Warning;

pub static HALT_SENDER: OnceCell<broadcast::Sender<()>> = OnceCell::const_new();
pub static LOG_SENDER: OnceCell<mpsc::Sender<LogMessage>> = OnceCell::const_new();

pub static CONTROLLER_COMMAND_SENDER: OnceCell<mpsc::Sender<ControllerCommands>> =
    OnceCell::const_new();
pub static INSTRUMENT_COMMAND_SENDER: OnceCell<mpsc::Sender<InstrumentCommands>> =
    OnceCell::const_new();
pub static HICUBE_COMMAND_SENDER: OnceCell<mpsc::Sender<HiCubeCommands>> = OnceCell::const_new();

pub static PLOT_PRESSURE_SENDER: OnceCell<mpsc::Sender<PressurePlotCommands>> =
    OnceCell::const_new();
pub static PLOT_TEMPERATURE_SENDER: OnceCell<mpsc::Sender<TemperaturePlotCommands>> =
    OnceCell::const_new();

#[tokio::main]
async fn main() {
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

    // Pressure plotting task
    let (tx_p_plot, rx_p_plot) = mpsc::channel(32);
    PLOT_PRESSURE_SENDER
        .set(tx_p_plot.clone())
        .expect("Uninitialized");
    let p_plot_task = tokio::spawn(pressure_plot_task(rx_p_plot));

    // Temperature plotting task
    let (tx_t_plot, rx_t_plot) = mpsc::channel(32);
    PLOT_TEMPERATURE_SENDER
        .set(tx_t_plot.clone())
        .expect("Uninitialized");
    let t_plot_task = tokio::spawn(temperature_plot_task(rx_t_plot));

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
    let (tx_instr, rx_instr) = mpsc::channel(32);
    INSTRUMENT_COMMAND_SENDER
        .set(tx_instr.clone())
        .expect("Uninitialized");
    let instr_tsk = tokio::spawn(instruments_task(
        Arc::clone(&conf),
        Arc::clone(&inst_status),
        rx_instr,
    ));

    // HiCube task
    let (tx_hicube, rx_hicube) = mpsc::channel(32);
    HICUBE_COMMAND_SENDER
        .set(tx_hicube.clone())
        .expect("Uninitialized");
    let hicube_conf = conf
        .lock()
        .expect("Locking config must work")
        .get_pfeiffer_hicube_config();
    let hicube_task = tokio::spawn(pfeiffer_hicube_task(
        hicube_conf,
        Arc::clone(&inst_status),
        rx_hicube,
    ));

    send_log_message(LogMessage::new_info(&format!(
        "Started cryostorage_host: BuildInfo - {}",
        env!("BUILD_INFO")
    )))
    .await;
    println!(
        "Started cryostorage_host: BuildInfo - {}",
        env!("BUILD_INFO")
    );

    // start the app
    match app::app_main(Arc::clone(&conf), Arc::clone(&inst_status), tx_ui_set) {
        Ok(_) => {
            tx_halt.send(()).unwrap();
            let _ = tokio::join!(
                cntrl_tsk,
                cntrl_bc_listen,
                hicube_task,
                instr_tsk,
                log_handler_listen,
                p_plot_task,
                t_plot_task
            );
            println!("App exited normally")
        }
        Err(e) => eprintln!("App exited with error: {}", e),
    }
}
