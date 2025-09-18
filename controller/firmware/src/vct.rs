//! Control the VCT handshake and status.

use embassy_rp::gpio::{Input, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use icd::{VctHandshakeState, VctState, VctStates};

pub static GET_VCT_STATUS: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static WATCH_VCT_STATUS: Watch<CriticalSectionRawMutex, VctStates, 2> = Watch::new();

/// Control the VCT handshake (set and get)
pub struct VctCtrl {
    pin_handshake: Output<'static>,
}

impl VctCtrl {
    /// Create a new VCT controller.
    pub fn new(pin_handshake: Output<'static>) -> Self {
        Self { pin_handshake }
    }

    pub fn get_handshake_state(&mut self) -> VctHandshakeState {
        match self.pin_handshake.is_set_high() {
            true => VctHandshakeState::Attach,
            false => VctHandshakeState::Detach,
        }
    }

    pub fn set_handshake_state(&mut self, state: VctHandshakeState) {
        match state {
            VctHandshakeState::Attach => self.pin_handshake.set_high(),
            VctHandshakeState::Detach => self.pin_handshake.set_low(),
        }
    }
}

/// Get the status of the VCT attach state and VCT gate state
pub struct VctStatus {
    pin_gate_status: Input<'static>,
    pin_attach_status: Input<'static>,
}

impl VctStatus {
    pub fn new(pin_gate_status: Input<'static>, pin_attach_status: Input<'static>) -> Self {
        Self {
            pin_gate_status,
            pin_attach_status,
        }
    }

    pub fn get_status(&mut self) -> VctStates {
        VctStates {
            attach: self.get_attach_status(),
            gate: self.get_gate_status(),
        }
    }

    fn get_attach_status(&mut self) -> VctState {
        match self.pin_attach_status.is_high() {
            true => VctState::Connected,
            false => VctState::Disconnected,
        }
    }

    fn get_gate_status(&mut self) -> VctState {
        match self.pin_gate_status.is_high() {
            true => VctState::Connected,
            false => VctState::Disconnected,
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn vct_status_task(mut vct_status: VctStatus) {
    let watch_sender = WATCH_VCT_STATUS.sender();
    loop {
        GET_VCT_STATUS.wait().await;
        watch_sender.send(vct_status.get_status());
    }
}
