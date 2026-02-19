//! Workflows for opening and closing valves.

use anyhow::{Result, anyhow, bail};
use icd::{ValveState, VctState};

use crate::{
    controller::{ControllerCommands, send_cntrl_cmd_now},
    prg_config::OpenValveAuthorization,
    status::PressureReading,
};

/// Get permission to close the transfer valve based on the VCT state.
///
/// If the VCT is connected, permission to close the transfer valve is denied as the arm could be
/// in. While this is also interlocked in electronics, this check allows us to also raise an
/// appropriate error message to the user.
fn get_close_transfer_valve_permission(vct_state: &VctState) -> Result<()> {
    if vct_state.is_connected() {
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
/// permission (as we are in a safe low pressure regime).
fn get_open_valve_permission(
    p_chamber: &PressureReading,
    p_transfer: &PressureReading,
    authorization: &OpenValveAuthorization,
) -> Result<()> {
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
    if p_ratio > authorization.valve_ratio_range.lower_limit
        && p_ratio < authorization.valve_ratio_range.upper_limit
    {
        Ok(())
    } else if p_chamber_mbar < authorization.low_pressure_limit_mbar
        && p_transfer_mbar < authorization.low_pressure_limit_mbar
    {
        Ok(())
    } else {
        Err(anyhow!("Unsafe to open valve."))
    }
}

/// Function to close the transfer valve.
pub fn close_transfer_valve(vct_state: &VctState) -> Result<()> {
    get_close_transfer_valve_permission(vct_state)?;
    send_cntrl_cmd_now(ControllerCommands::TransferValve(ValveState::Closed));
    Ok(())
}

/// Function to open the transfer valve.
pub fn open_transfer_valve(
    p_chamber: &PressureReading,
    p_transfer: &PressureReading,
    authorization: &OpenValveAuthorization,
) -> Result<()> {
    get_open_valve_permission(p_chamber, p_transfer, authorization)?;
    send_cntrl_cmd_now(ControllerCommands::TransferValve(ValveState::Open));
    Ok(())
}

/// Function to close the pump valve (no permissions required).
pub fn close_pump_valve() -> Result<()> {
    send_cntrl_cmd_now(ControllerCommands::PumpValve(ValveState::Closed));
    Ok(())
}

/// Function to open the pump valve.
pub fn open_pump_valve(
    p_chamber: &PressureReading,
    p_transfer: &PressureReading,
    authorization: &OpenValveAuthorization,
) -> Result<()> {
    get_open_valve_permission(p_chamber, p_transfer, authorization)?;
    send_cntrl_cmd_now(ControllerCommands::PumpValve(ValveState::Open));
    Ok(())
}
