use defmt::Format;
use esp_hal::rng::Rng;
use heapless::{String, Vec};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, ListState, Paragraph};

use crate::app::ScreenAction;
use crate::app::screens::Screen;
use crate::keepass::KeePassDb;

const PIN_LEN: usize = 4;
const DIGIT_COUNT: usize = 10;
const DIGIT_LINE_CAP: usize = 32;
const BLINK_PERIOD_FRAMES: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DigitSpan {
    start: usize,
    width: usize,
}

#[derive(Debug, Format)]
pub struct PinEntryScreen {
    pin: String<PIN_LEN>,
    digit_order: [u8; DIGIT_COUNT],
    last_rendered_selected: Option<usize>,
}

impl PinEntryScreen {
    pub fn new() -> Self {
        Self {
            pin: String::new(),
            digit_order: Self::shuffled_digits(),
            last_rendered_selected: None,
        }
    }

    pub fn item_count(&self) -> usize {
        DIGIT_COUNT
    }

    fn shuffled_digits() -> [u8; DIGIT_COUNT] {
        let mut digits = [0u8; DIGIT_COUNT];
        for (idx, digit) in digits.iter_mut().enumerate() {
            *digit = idx as u8;
        }

        let rng = Rng::new();
        for i in (1..DIGIT_COUNT).rev() {
            let j = Self::random_index(&rng, (i as u8).saturating_add(1));
            digits.swap(i, j);
        }

        digits
    }

    fn random_index(rng: &Rng, upper_exclusive: u8) -> usize {
        if upper_exclusive <= 1 {
            return 0;
        }

        let m = upper_exclusive as u16;
        let zone = 256u16 - (256u16 % m);
        let mut raw = [0u8; 1];

        loop {
            rng.read(&mut raw);
            if (raw[0] as u16) < zone {
                return (raw[0] as u16 % m) as usize;
            }
        }
    }

    fn build_digit_line(&self) -> (String<DIGIT_LINE_CAP>, Vec<DigitSpan, DIGIT_COUNT>) {
        let mut line: String<DIGIT_LINE_CAP> = String::new();
        let mut spans: Vec<DigitSpan, DIGIT_COUNT> = Vec::new();
        let mut cursor: usize = 0;

        for (idx, digit) in self.digit_order.iter().enumerate() {
            if idx > 0 {
                if line.push(' ').is_err() {
                    break;
                }
                cursor = cursor.saturating_add(1);
            }

            if spans
                .push(DigitSpan {
                    start: cursor,
                    width: 1,
                })
                .is_err()
            {
                break;
            }

            let ch = char::from(b'0' + *digit);
            if line.push(ch).is_err() {
                break;
            }
            cursor = cursor.saturating_add(1);
        }

        (line, spans)
    }

    fn masked_display(&self, frame: &Frame) -> String<{ PIN_LEN + 1 }> {
        let mut out: String<{ PIN_LEN + 1 }> = String::new();
        for _ in 0..self.pin.len() {
            let _ = out.push('*');
        }

        if self.pin.len() < PIN_LEN && (frame.count() / BLINK_PERIOD_FRAMES) % 2 == 0 {
            let _ = out.push('_');
        }

        out
    }

    fn push_digit(&mut self, digit: u8) {
        if self.pin.len() >= PIN_LEN {
            return;
        }
        let ch = char::from(b'0' + digit);
        let _ = self.pin.push(ch);
    }
}

impl Screen for PinEntryScreen {
    fn new() -> Self {
        Self::new()
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, _: &KeePassDb) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(frame.area());

        let top_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" Enter PIN ");
        let top_inner = top_block.inner(chunks[0]);
        frame.render_widget(top_block, chunks[0]);

        let display_text = self.masked_display(frame);
        if top_inner.height > 0 && top_inner.width > 0 {
            let y = top_inner.y + top_inner.height / 2;
            let text_area = Rect {
                x: top_inner.x,
                y,
                width: top_inner.width,
                height: 1,
            };

            let paragraph = Paragraph::new(display_text.as_str())
                .alignment(Alignment::Center)
                .style(Style::new().bold());
            frame.render_widget(paragraph, text_area);
        }

        let bottom_block = Block::bordered().border_style(Style::new().bold().green());
        let bottom_inner = bottom_block.inner(chunks[1]);
        frame.render_widget(bottom_block, chunks[1]);

        if bottom_inner.is_empty() || bottom_inner.height == 0 {
            self.last_rendered_selected = Some(0);
            return;
        }

        let (digit_line, spans) = self.build_digit_line();
        if spans.is_empty() {
            self.last_rendered_selected = Some(0);
            return;
        }

        let selected_raw = selected
            .selected()
            .or(self.last_rendered_selected)
            .unwrap_or(0);
        let selected_idx = selected_raw.min(spans.len().saturating_sub(1));
        self.last_rendered_selected = Some(selected_idx);

        let paragraph = Paragraph::new(digit_line.as_str());
        frame.render_widget(paragraph, bottom_inner);

        if let Some(span) = spans.get(selected_idx) {
            let highlight = Rect {
                x: bottom_inner.x + span.start as u16,
                y: bottom_inner.y,
                width: span.width as u16,
                height: 1,
            };
            frame.buffer_mut().set_style(
                highlight,
                Style::new().bold().bg(Color::White).fg(Color::Black),
            );
        }
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        let selected = self.last_rendered_selected.or(selected);
        let Some(selected) = selected else {
            return ScreenAction::None;
        };

        if let Some(digit) = self.digit_order.get(selected).copied() {
            self.push_digit(digit);
        }

        if self.pin.len() >= PIN_LEN {
            ScreenAction::Pop
        } else {
            ScreenAction::None
        }
    }
}
