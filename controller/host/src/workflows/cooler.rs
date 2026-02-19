//! Workflows for starting and stopping the cryocooler.

use anyhow::{Result, anyhow, bail};

use icd::{BakingState, FlowMeterState};
use sunpower_cryotelgt::CoolerState;

use crate::{
    instruments::{InstrumentCommands, send_instr_cmd_now},
    prg_config::CryoCoolerAuthorization,
    status::PressureReading,
};

/// Get permission to start the cryocooler.
///
/// Checks for active watercooling and safe pressure levels.
fn get_start_cooler_permission(
    flow_meter: &FlowMeterState,
    baking: &BakingState,
    p_chamber: &PressureReading,
    authorization: &CryoCoolerAuthorization,
) -> Result<()> {
    let p_chamber_mbar = match p_chamber {
        PressureReading::Value(val) => val.as_millibars(),
        _ => bail!("Chamber pressure gauge error: Check gauge and try again."),
    };

    if !matches!(baking, BakingState::Off) {
        bail!("Baking is on: cannot start cooler.")
    };

    if !matches!(flow_meter, FlowMeterState::Ok) {
        bail!("Water flow error: cannot start cooler.")
    };

    if p_chamber_mbar < authorization.max_pressure_on_mbar {
        Ok(())
    } else {
        Err(anyhow!(
            "Chamber pressure is too high: cannot start cooler."
        ))
    }
}

/// Function to start the cryocooler.
pub fn start_cooler(
    flow_meter: &FlowMeterState,
    baking: &BakingState,
    p_chamber: &PressureReading,
    authorization: &CryoCoolerAuthorization,
) -> Result<()> {
    get_start_cooler_permission(flow_meter, baking, p_chamber, authorization)?;
    send_instr_cmd_now(InstrumentCommands::CryoCoolerState(CoolerState::Enabled));
    Ok(())
}

/// Function to stop the cryocooler (no permissions needed).
pub fn stop_cooler() -> Result<()> {
    send_instr_cmd_now(InstrumentCommands::CryoCoolerState(CoolerState::Disabled));
    Ok(())
}
