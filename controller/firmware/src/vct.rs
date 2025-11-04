//! Deal with the VCT.

use embassy_rp::gpio::{Input, Output};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use icd::{VctConnectedState, VctHandshake, VctState};

static VCT_SIGNAL: Signal<ThreadModeRawMutex, VctCommand> = Signal::new();
static VCT_STATE_SIGNAL: Signal<ThreadModeRawMutex, VctState> = Signal::new();

/// VCT command enum.
enum VctCommand {
    /// Set the handshake to the given value.
    SetHandshake(VctHandshake),
    /// Get the current state.
    GetState,
}

/// VCT controller.
pub struct Vct {
    pin_handshake: Output<'static>,
    pin_gate: Input<'static>,
    pin_attach: Input<'static>,
}

impl Vct {
    /// Create a new VCT controller from the given pins.
    pub fn new(
        pin_handshake: Output<'static>,
        pin_gate: Input<'static>,
        pin_attach: Input<'static>,
    ) -> Self {
        Self {
            pin_handshake,
            pin_gate,
            pin_attach,
        }
    }

    /// Set the handshake.
    pub fn set_handshake(&mut self, value: VctHandshake) {
        match value {
            VctHandshake::Ready => self.pin_handshake.set_high(),
            VctHandshake::NotReady => self.pin_handshake.set_low(),
        }
    }

    /// Get the overall state of the VCT.
    pub fn get_state(&self) -> VctState {
        let gate = match self.pin_gate.is_high() {
            true => VctConnectedState::Connected,
            false => VctConnectedState::Disconnected,
        };
        let attach = match self.pin_attach.is_high() {
            true => VctConnectedState::Connected,
            false => VctConnectedState::Disconnected,
        };
        VctState { gate, attach }
    }
}

/// The task for controlling the VCT.
///
/// If this task dies, the VCT will no longer respond to commands and thus, the broadcaster will
/// stop feeding the watchdog and the system will reset.
#[embassy_executor::task(pool_size = 1)]
pub async fn vct_task(mut vct: Vct) {
    loop {
        let command = VCT_SIGNAL.wait().await;
        match command {
            VctCommand::SetHandshake(value) => {
                vct.set_handshake(value);
            }
            VctCommand::GetState => {
                VCT_STATE_SIGNAL.signal(vct.get_state());
            }
        }
    }
}

/// Set the VCT handshake state.
pub fn vct_set_handshake(value: VctHandshake) {
    VCT_SIGNAL.signal(VctCommand::SetHandshake(value));
}

/// Get the current VCT state.
pub async fn vct_get_state() -> VctState {
    VCT_SIGNAL.signal(VctCommand::GetState);
    VCT_STATE_SIGNAL.wait().await
}
