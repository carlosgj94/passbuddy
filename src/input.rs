use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::peripherals;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputEvent {
    pub delta: i16,
    pub pressed: bool,
}

pub struct Inputs<'d> {
    encoder: RotaryEncoder<'d>,
    button: DebouncedButton<'d>,
}

impl<'d> Inputs<'d> {
    pub fn new(
        clk_pin: peripherals::GPIO17<'d>,
        dt_pin: peripherals::GPIO15<'d>,
        sw_pin: peripherals::GPIO16<'d>,
    ) -> Self {
        let config = InputConfig::default().with_pull(Pull::Up);
        let clk = Input::new(clk_pin, config);
        let dt = Input::new(dt_pin, config);
        let sw = Input::new(sw_pin, config);

        Self {
            encoder: RotaryEncoder::new(clk, dt),
            button: DebouncedButton::new(sw),
        }
    }

    pub fn poll(&mut self) -> InputEvent {
        InputEvent {
            delta: self.encoder.poll_delta(),
            pressed: self.button.poll_pressed(),
        }
    }

    pub fn poll_encoder_delta(&mut self) -> i16 {
        self.encoder.poll_delta()
    }

    pub fn poll_button_pressed(&mut self) -> bool {
        self.button.poll_pressed()
    }
}

/// Debounces an active-low momentary button and emits a single `true` per press.
///
/// Call `poll_pressed()` at a fixed cadence (e.g. your main loop tick). A press
/// is emitted on the first `low` read after the button has been released (`high`)
/// for `release_streak_required` consecutive polls.
pub struct DebouncedButton<'d> {
    pin: Input<'d>,
    armed: bool,
    high_streak: u8,
    release_streak_required: u8,
}

impl<'d> DebouncedButton<'d> {
    pub fn new(pin: Input<'d>) -> Self {
        Self {
            armed: !pin.is_low(),
            pin,
            high_streak: 0,
            // Default: 2 consecutive "released" reads before re-arming.
            release_streak_required: 2,
        }
    }

    pub fn with_release_streak_required(mut self, release_streak_required: u8) -> Self {
        self.release_streak_required = release_streak_required.max(1);
        self
    }

    /// Returns `true` once per physical press.
    pub fn poll_pressed(&mut self) -> bool {
        let low = self.pin.is_low();
        if low {
            self.high_streak = 0;
            if self.armed {
                self.armed = false;
                return true;
            }
            return false;
        }

        if !self.armed {
            self.high_streak = self.high_streak.saturating_add(1);
            if self.high_streak >= self.release_streak_required {
                self.armed = true;
            }
        }

        false
    }
}

pub struct RotaryEncoder<'d> {
    clk: Input<'d>,
    dt: Input<'d>,
    state: u8,
    accum: i8,
}

impl<'d> RotaryEncoder<'d> {
    const STEP_THRESHOLD: i8 = 4;
    const TRANSITION_TABLE: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];

    pub fn new(clk: Input<'d>, dt: Input<'d>) -> Self {
        let state = Self::pin_state(&clk, &dt);
        Self {
            clk,
            dt,
            state,
            accum: 0,
        }
    }

    /// Returns the encoder delta (PCNT counts) since the last call.
    pub fn poll_delta(&mut self) -> i16 {
        let pin_state = Self::pin_state(&self.clk, &self.dt);
        self.state = ((self.state << 2) | pin_state) & 0x0f;
        let delta = Self::TRANSITION_TABLE[self.state as usize];

        if delta != 0 {
            self.accum = self.accum.saturating_add(delta);
            if self.accum >= Self::STEP_THRESHOLD {
                self.accum = 0;
                return 1;
            }
            if self.accum <= -Self::STEP_THRESHOLD {
                self.accum = 0;
                return -1;
            }
        }

        0
    }

    fn pin_state(clk: &Input<'d>, dt: &Input<'d>) -> u8 {
        let mut state = 0u8;
        if clk.is_high() {
            state |= 0x01;
        }
        if dt.is_high() {
            state |= 0x02;
        }
        state
    }
}
