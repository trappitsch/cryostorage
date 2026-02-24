//! Workflows for opening and closing valves.

use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow, bail};
use icd::ValveState;

use crate::{
    controller::{ControllerCommands, send_cntrl_cmd_now},
    prg_config::Authorizations,
    status::{InstrumentStatus, PressureReading},
};

/// Get permission to close the transfer valve based on the VCT state.
///
/// If the VCT dock gate valve is currently open, permission to close the transfer valve is 
/// denied as the arm could be in. This check allows raising an appropriate error message 
/// to the user.
fn get_close_transfer_valve_permission(inst_status: Arc<Mutex<InstrumentStatus>>) -> Result<()> {
    if inst_status.lock().expect("Poisoned").is_vct_gate_open() {
        Err(anyhow!("Unsafe to close transfer valve: VCT is connected."))
    } else {
        Ok(())
    }
}

/// Get permission to open a valve based on the pressure gauges.
///
/// We do the following checks:
/// - If any of the gauges show 0 mbar (error state), we do not give permission.
/// - If the ratio of the pressures is within a specified safe range, we give permission.
/// - If both pressures are below a specified low pressure limit, regardless of the ratio, we give
///   permission (as we are in a safe low pressure regime).
fn get_open_valve_permission(
    inst_status: Arc<Mutex<InstrumentStatus>>,
    auths: &Authorizations,
) -> Result<()> {
    let (p_chamber, p_transfer) = inst_status.lock().expect("Poisoned").get_pressures();
    let auth = &auths.open_valve;

    let p_chamber_mbar = match p_chamber {
        PressureReading::Value(val) => val.as_millibars(),
        _ => bail!("Chamber pressure gauge error: Check gauge and try again."),
    };
    let p_transfer_mbar = match p_transfer {
        PressureReading::Value(val) => val.as_millibars(),
        _ => bail!("Transfer pressure gauge error: Check gauge and try again."),
    };

    let p_ratio = p_chamber_mbar / p_transfer_mbar;

    #[allow(clippy::if_same_then_else)] // We want to be explicit about both conditions for
    // readability and comparison with the workflow
    if p_ratio > auth.valve_ratio_range.lower_limit && p_ratio < auth.valve_ratio_range.upper_limit
    {
        Ok(())
    } else if p_chamber_mbar < auth.low_pressure_limit_mbar
        && p_transfer_mbar < auth.low_pressure_limit_mbar
    {
        Ok(())
    } else {
        Err(anyhow!("Unsafe to open valve."))
    }
}

/// Function to set the transfer valve.
pub fn set_transfer_valve(
    inst_status: Arc<Mutex<InstrumentStatus>>,
    auths: &Authorizations,
    valve_state: ValveState,
) -> Result<()> {
    match valve_state {
        ValveState::Open => get_open_valve_permission(inst_status, auths)?,
        ValveState::Closed => get_close_transfer_valve_permission(inst_status)?,
        ValveState::Undefined => unreachable!("Can only open or close value in workflows."),
    }
    send_cntrl_cmd_now(ControllerCommands::TransferValve(valve_state));
    Ok(())
}

/// Function to set the pump valve.
pub fn set_pump_valve(
    inst_status: Arc<Mutex<InstrumentStatus>>,
    auths: &Authorizations,
    valve_state: ValveState,
) -> Result<()> {
    if valve_state == ValveState::Open {
        get_open_valve_permission(inst_status, auths)?;
    }
    send_cntrl_cmd_now(ControllerCommands::PumpValve(valve_state));
    Ok(())
}
