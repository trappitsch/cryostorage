//! Deals with baking the instrument.

use embassy_rp::gpio::Output;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Duration, Instant};
use icd::BakingState;

static BAKING_SIGNAL: Signal<ThreadModeRawMutex, BakingCommand> = Signal::new();
static BAKING_STATE_SIGNAL: Signal<ThreadModeRawMutex, BakingState> = Signal::new();

/// Controller for baking the instrument
///
/// Getting the current baking state will update the timer and return the actual state. However,
/// this will also check if the baking time has elapsed and if so, turn the baking off. Thus,
/// regular calls to `get_state` (which are done from the broadcaster) are needed to ensure that
/// the baking will turn off when the time is up.
pub struct Baking {
    /// Output pin that controls the baking relay. High means on.
    pin_ctrl: Output<'static>,
    /// Current state of the baking controller. Updated when `update` is called.
    current_state: BakingState,
    /// Option of time when baking was started (if on) or None.
    start_time: Option<Instant>,
    /// Option for the originally called baking duration (if on) or None.
    baking_duration: Option<Duration>,
}

impl Baking {
    /// Create a new baking controller.
    ///
    /// If `time_sec` > 0 is supplied, the baking will start immediately for that duration.
    /// This allows us to store the baking state in a watchdog scratch area and restore it if
    /// needed. As the scratch area is initialized with zeros on a cold boot and a `time_sec` of
    /// zero will initialize in `BakingState::Off`, this feature should be safe.
    ///
    /// # Arguments
    ///
    /// - `pin_ctrl` is the output pin that controls the baking relay.
    /// - `time_sec` is the initial baking time in seconds.
    pub fn new(pin_ctrl: Output<'static>, time_sec: u64) -> Self {
        let mut bk = Self {
            pin_ctrl,
            current_state: BakingState::Off,
            start_time: None,
            baking_duration: None,
        };
        if time_sec > 0 {
            bk.set_state(BakingState::On { time_sec });
        };
        bk
    }

    /// Get the current baking state.
    ///
    /// This will update the timer and then return the actual current state.
    pub fn get_state(&mut self) -> BakingState {
        self.update();
        self.current_state.clone()
    }

    /// Set a new baking state.
    pub fn set_state(&mut self, value: BakingState) {
        match value {
            BakingState::Off => {
                self.pin_ctrl.set_low();
                self.start_time = None;
                self.baking_duration = None;
            }
            BakingState::On { time_sec } => {
                self.pin_ctrl.set_high();
                self.start_time = Some(Instant::now());
                self.baking_duration = Some(Duration::from_secs(time_sec));
            }
        }
        self.current_state = value;
    }

    /// Update the baking state based on the elapsed time.
    ///
    /// If the baking is on, this will check if the time has elapsed and turn it off if needed.
    /// Otherwise, it will update the remaining time.
    ///
    /// If the baking is off, this does nothing.
    pub fn update(&mut self) {
        if let BakingState::On { time_sec: _ } = self.current_state {
            if let Some(start) = self.start_time {
                let elapsed = Instant::now() - start;

                let initial_duration = self.baking_duration.unwrap_or(Duration::from_secs(0));

                if elapsed >= initial_duration {
                    // Baking time is over
                    self.set_state(BakingState::Off);
                } else {
                    // Still baking, update time
                    self.current_state = BakingState::On {
                        time_sec: (initial_duration - elapsed).as_secs(),
                    }
                }
            } else {
                // This should not happen, something is wrong, turn off baking
                defmt::error!(
                    "Baking controller claims to be on but has no start time set! This should not happen. Turning off."
                );
                self.set_state(BakingState::Off);
            }
        }
    }
}

/// Commands for the baking task.
pub enum BakingCommand {
    /// Set the baking state.
    SetState(BakingState),
    /// Get the baking state.
    GetState,
}

/// The task for the baking controller.
///
/// If this task dies, the broadcast will stop feeding the watchdog and thus the system will reset.
#[embassy_executor::task(pool_size = 1)]
pub async fn baking_task(mut baking: Baking) {
    loop {
        let command = BAKING_SIGNAL.wait().await;
        match command {
            BakingCommand::SetState(state) => {
                baking.set_state(state);
            }
            BakingCommand::GetState => {
                BAKING_STATE_SIGNAL.signal(baking.get_state());
            }
        }
    }
}

/// Query the current baking state.
pub async fn baking_get() -> BakingState {
    BAKING_SIGNAL.signal(BakingCommand::GetState);
    BAKING_STATE_SIGNAL.wait().await
}

/// Set the baking state.
pub fn baking_set(state: BakingState) {
    BAKING_SIGNAL.signal(BakingCommand::SetState(state));
}
