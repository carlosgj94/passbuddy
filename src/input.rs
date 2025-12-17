use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::pcnt::{Pcnt, channel};
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
        pcnt: peripherals::PCNT<'d>,
        clk_pin: peripherals::GPIO15<'d>,
        dt_pin: peripherals::GPIO17<'d>,
        sw_pin: peripherals::GPIO16<'d>,
    ) -> Self {
        let config = InputConfig::default().with_pull(Pull::Up);
        let clk = Input::new(clk_pin, config);
        let dt = Input::new(dt_pin, config);
        let sw = Input::new(sw_pin, config);

        Self {
            encoder: RotaryEncoder::new(pcnt, clk, dt),
            button: DebouncedButton::new(sw),
        }
    }

    pub fn poll(&mut self) -> InputEvent {
        InputEvent {
            delta: self.encoder.poll_delta(),
            pressed: self.button.poll_pressed(),
        }
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
    pcnt: Pcnt<'d>,
    last_count: i16,
}

impl<'d> RotaryEncoder<'d> {
    const FILTER_THRESHOLD_APB_CYCLES: u16 = 800;

    pub fn new(pcnt: peripherals::PCNT<'d>, clk: Input<'d>, dt: Input<'d>) -> Self {
        let pcnt = Pcnt::new(pcnt);

        let clk_signal = clk.peripheral_input();
        let dt_signal = dt.peripheral_input();

        {
            let u0 = &pcnt.unit0;
            u0.set_filter(Some(Self::FILTER_THRESHOLD_APB_CYCLES))
                .expect("pcnt filter threshold");
            u0.clear();

            let ch0 = &u0.channel0;
            ch0.set_ctrl_signal(dt_signal.clone());
            ch0.set_edge_signal(clk_signal.clone());
            ch0.set_ctrl_mode(channel::CtrlMode::Reverse, channel::CtrlMode::Keep);
            ch0.set_input_mode(channel::EdgeMode::Hold, channel::EdgeMode::Increment);

            u0.resume();
        }

        let last_count = pcnt.unit0.counter.get();

        Self { pcnt, last_count }
    }

    /// Returns the encoder delta (PCNT counts) since the last call.
    pub fn poll_delta(&mut self) -> i16 {
        let count = self.pcnt.unit0.counter.get();
        let delta = count.wrapping_sub(self.last_count);
        self.last_count = count;
        delta
    }
}
