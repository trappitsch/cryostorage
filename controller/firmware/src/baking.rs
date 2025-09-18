//! Module to provide support and potential cancellation for baking the instrument.

use embassy_futures::select::{Either, select};
use embassy_rp::gpio::Output;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::{Duration, Instant, Timer};
use icd::BakingState;

pub static SET_BAKING_SIGNAL: Signal<CriticalSectionRawMutex, BakingState> = Signal::new();
pub static GET_BAKING_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static WATCH_BAKING: Watch<CriticalSectionRawMutex, BakingState, 2> = Watch::new();

// baking internal
static START_BAKING_SIGNAL: Signal<CriticalSectionRawMutex, StartBaking> = Signal::new();
static STOP_BAKING_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

enum StartBaking {
    Start(Duration),
}

#[derive(Default)]
pub struct BakingCtrl {
    state: CurrBakingState,
}

impl BakingCtrl {
    /// Get the current status of the Baking system
    pub fn get_status(&mut self) -> BakingState {
        self.state.update();
        self.state.into()
    }

    /// Turns the baking on or off
    pub fn control(&mut self, state: BakingState) {
        self.state.update(); // sets it to off if timer expired
        match state {
            BakingState::Off => {
                if let CurrBakingState::On { .. } = self.state {
                    STOP_BAKING_SIGNAL.signal(());
                    self.state = CurrBakingState::Off;
                }
            }
            BakingState::On { time_sec } => {
                if let CurrBakingState::Off = self.state {
                    let dur = Duration::from_secs(time_sec);
                    self.state = CurrBakingState::On {
                        left: dur,
                        set: dur,
                        started: Instant::now(),
                    };
                    STOP_BAKING_SIGNAL.reset();
                    START_BAKING_SIGNAL.signal(StartBaking::Start(dur));
                }
            }
        }
    }

    pub fn stop(&mut self) {}
}

/// Represents the current baking state with all times as an enum.
#[derive(Copy, Clone, Default, PartialEq)]
enum CurrBakingState {
    On {
        /// Time left in this timer
        left: Duration,
        /// Originally set time
        set: Duration,
        /// Instant the timer was started
        started: Instant,
    },
    #[default]
    Off,
}

impl CurrBakingState {
    /// Update the times in the current baking state if they are on
    fn update(&mut self) {
        if let CurrBakingState::On {
            ref mut left,
            set,
            started,
        } = *self
        {
            let elps = started.elapsed();
            if elps >= set {
                *self = CurrBakingState::Off;
            } else {
                *left = set - elps; // panics if < Duration::MIN
            }
        };
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn baking_main_task(mut baking_ctrl: BakingCtrl) {
    let watch_sender = WATCH_BAKING.sender();
    loop {
        match select(SET_BAKING_SIGNAL.wait(), GET_BAKING_SIGNAL.wait()).await {
            Either::First(state) => {
                baking_ctrl.control(state);
            }
            Either::Second(_) => {
                watch_sender.send(baking_ctrl.get_status());
            }
        }
    }
}

#[embassy_executor::task(pool_size = 1)]
pub async fn baking_ctrl_task(mut p_baking: Output<'static>) {
    loop {
        let StartBaking::Start(dur) = START_BAKING_SIGNAL.wait().await;

        p_baking.set_high();
        select(Timer::after(dur), STOP_BAKING_SIGNAL.wait()).await;
        p_baking.set_low();

        START_BAKING_SIGNAL.reset();
    }
}

impl From<CurrBakingState> for BakingState {
    fn from(value: CurrBakingState) -> Self {
        match value {
            CurrBakingState::Off => BakingState::Off,
            CurrBakingState::On { left, .. } => BakingState::On {
                time_sec: left.as_secs(),
            },
        }
    }
}
