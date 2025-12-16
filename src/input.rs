use esp_hal::analog::adc::{Adc, AdcConfig, AdcPin, Attenuation};
use esp_hal::gpio::AnalogPin;
use esp_hal::gpio::Input;
use esp_hal::peripherals;
use ratatui::widgets::ListState;

pub struct Inputs<'d> {
    pot: Potentiometer<'d, peripherals::GPIO1<'d>>,
}

impl<'d> Inputs<'d> {
    pub fn new(
        adc1: peripherals::ADC1<'d>,
        pot_pin: peripherals::GPIO1<'d>,
        menu_items: usize,
    ) -> Self {
        Self {
            pot: Potentiometer::new_adc1(adc1, pot_pin, menu_items),
        }
    }

    /// Reads inputs and mutates UI state accordingly.
    ///
    /// Returns the latest potentiometer reading (raw ADC code).
    pub fn poll_menu(&mut self, state: &mut ListState) -> u16 {
        self.pot.poll_menu(state)
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

pub struct Potentiometer<'d, PIN>
where
    PIN: AnalogPin + esp_hal::analog::adc::AdcChannel,
{
    adc: Adc<'d, peripherals::ADC1<'d>, esp_hal::Blocking>,
    pin: AdcPin<PIN, peripherals::ADC1<'d>>,
    menu_items: usize,
    min_raw: Option<u16>,
    max_raw: Option<u16>,
    pending_idx: Option<usize>,
    stable_count: u8,
    stable_required: u8,
}

impl<'d, PIN> Potentiometer<'d, PIN>
where
    PIN: AnalogPin + esp_hal::analog::adc::AdcChannel,
{
    pub fn new_adc1(adc1: peripherals::ADC1<'d>, pot_pin: PIN, menu_items: usize) -> Self {
        let mut adc_config = AdcConfig::new();
        let pin = adc_config.enable_pin(pot_pin, Attenuation::_11dB);
        let adc = Adc::new(adc1, adc_config);

        Self {
            adc,
            pin,
            menu_items: menu_items.max(1),
            min_raw: None,
            max_raw: None,
            pending_idx: None,
            stable_count: 0,
            // 2 consecutive reads in the same bucket before committing selection.
            stable_required: 2,
        }
    }

    pub fn poll_menu(&mut self, state: &mut ListState) -> u16 {
        let raw: u16 = self.adc.read_blocking(&mut self.pin);

        self.min_raw = Some(self.min_raw.map_or(raw, |min| min.min(raw)));
        self.max_raw = Some(self.max_raw.map_or(raw, |max| max.max(raw)));

        let min = self.min_raw.unwrap_or(raw);
        let max = self.max_raw.unwrap_or(raw);

        let items = self.menu_items;
        let span = max.saturating_sub(min);
        let scaled = (raw.saturating_sub(min) as u32) * (items as u32);
        let mut idx = (scaled / (span as u32 + 1)) as usize;
        if idx >= items {
            idx = items - 1;
        }

        match self.pending_idx {
            Some(pending) if pending == idx => {
                self.stable_count = self.stable_count.saturating_add(1);
            }
            _ => {
                self.pending_idx = Some(idx);
                self.stable_count = 1;
            }
        }

        if self.stable_count >= self.stable_required && state.selected() != Some(idx) {
            state.select(Some(idx));
        }

        raw
    }
}
