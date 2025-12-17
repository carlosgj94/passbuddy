use defmt::Format;
use heapless::String;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, ListState, Paragraph};

use crate::app::ScreenAction;
use crate::app::screens::Screen;
use crate::keepass::KeePassDb;

pub const MAX_TEXT_LEN: usize = 64;
const KEYBOARD_LINE_CAP: usize = 128;
const BLINK_PERIOD_FRAMES: usize = 20;

const LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeyboardKey {
    Submit,
    Letter(u8),
    Space,
    Delete,
    Back,
}

#[derive(Debug, Format)]
pub struct TextEntryFormScreen {
    text: String<MAX_TEXT_LEN>,
    keyboard_scroll_x: u16,
    last_rendered_selected: Option<usize>,
}

impl TextEntryFormScreen {
    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.keyboard_scroll_x = 0;
    }

    pub fn set_text(&mut self, value: &str) {
        self.text.clear();
        self.keyboard_scroll_x = 0;
        for ch in value.chars() {
            let _ = self.text.push(ch);
        }
    }

    fn has_submit(&self) -> bool {
        !self.text.is_empty()
    }

    fn key_count(&self) -> usize {
        let submit = if self.has_submit() { 1 } else { 0 };
        submit + LETTERS.len() + 3
    }

    fn key_at(&self, index: usize) -> Option<KeyboardKey> {
        let mut idx = index;

        if self.has_submit() {
            if idx == 0 {
                return Some(KeyboardKey::Submit);
            }
            idx = idx.saturating_sub(1);
        }

        if idx < LETTERS.len() {
            return Some(KeyboardKey::Letter(idx as u8));
        }
        idx = idx.saturating_sub(LETTERS.len());

        match idx {
            0 => Some(KeyboardKey::Space),
            1 => Some(KeyboardKey::Delete),
            2 => Some(KeyboardKey::Back),
            _ => None,
        }
    }

    fn label_for_key(key: KeyboardKey) -> &'static str {
        match key {
            KeyboardKey::Submit => "Submit",
            KeyboardKey::Letter(i) => LETTERS[i as usize],
            KeyboardKey::Space => "<space>",
            KeyboardKey::Delete => "Del",
            KeyboardKey::Back => "Back",
        }
    }

    fn build_keyboard_line(
        &self,
        selected_idx: usize,
    ) -> (String<KEYBOARD_LINE_CAP>, usize, usize, usize) {
        let mut line: String<KEYBOARD_LINE_CAP> = String::new();
        let mut cursor: usize = 0;
        let mut selected_start: usize = 0;
        let mut selected_width: usize = 0;

        let count = self.key_count();
        for idx in 0..count {
            let Some(key) = self.key_at(idx) else {
                continue;
            };
            let label = Self::label_for_key(key);

            if idx > 0 {
                if line.push(' ').is_err() {
                    break;
                }
                cursor = cursor.saturating_add(1);
            }

            if idx == selected_idx {
                selected_start = cursor;
                selected_width = label.len();
            }

            if line.push_str(label).is_err() {
                break;
            }
            cursor = cursor.saturating_add(label.len());
        }

        (line, selected_start, selected_width, cursor)
    }
}

impl Screen for TextEntryFormScreen {
    fn new() -> Self {
        Self {
            text: String::new(),
            keyboard_scroll_x: 0,
            last_rendered_selected: None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, _: &KeePassDb) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(frame.area());

        let top_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" Write text ");
        let top_inner = top_block.inner(chunks[0]);
        frame.render_widget(top_block, chunks[0]);

        let blink_on = (frame.count() / BLINK_PERIOD_FRAMES) % 2 == 0;
        let display_text = if self.text.is_empty() {
            if blink_on { "_" } else { "" }
        } else {
            self.text.as_str()
        };

        if top_inner.height > 0 && top_inner.width > 0 {
            let y = top_inner.y + top_inner.height / 2;
            let text_area = Rect {
                x: top_inner.x,
                y,
                width: top_inner.width,
                height: 1,
            };

            let paragraph = Paragraph::new(display_text)
                .alignment(Alignment::Center)
                .style(Style::new().bold());
            frame.render_widget(paragraph, text_area);
        }

        let bottom_block = Block::bordered().border_style(Style::new().bold().green());
        let bottom_inner = bottom_block.inner(chunks[1]);
        frame.render_widget(bottom_block, chunks[1]);

        let key_count = self.key_count();
        if key_count == 0 || bottom_inner.is_empty() || bottom_inner.height == 0 {
            self.keyboard_scroll_x = 0;
            self.last_rendered_selected = Some(0);
            return;
        }

        let selected_raw = selected
            .selected()
            .or(self.last_rendered_selected)
            .unwrap_or(0);
        let selected_idx = selected_raw.min(key_count - 1);
        self.last_rendered_selected = Some(selected_idx);

        let (keyboard_line, selected_start, selected_width, total_width) =
            self.build_keyboard_line(selected_idx);

        let view_width = bottom_inner.width as usize;
        let mut scroll_x = self.keyboard_scroll_x as usize;
        if view_width == 0 {
            scroll_x = 0;
        } else {
            let max_scroll = total_width.saturating_sub(view_width);
            scroll_x = scroll_x.min(max_scroll);

            let visible_left = scroll_x;
            let visible_right = visible_left.saturating_add(view_width);
            let selected_right = selected_start.saturating_add(selected_width);

            if selected_start < visible_left {
                scroll_x = selected_start;
            } else if selected_right > visible_right {
                scroll_x = selected_right.saturating_sub(view_width);
            }

            let max_scroll = total_width.saturating_sub(view_width);
            scroll_x = scroll_x.min(max_scroll);
        }
        self.keyboard_scroll_x = scroll_x as u16;

        let paragraph = Paragraph::new(keyboard_line.as_str()).scroll((0, self.keyboard_scroll_x));
        frame.render_widget(paragraph, bottom_inner);

        let visible_left = scroll_x;
        let visible_right = visible_left.saturating_add(view_width);
        let selected_left = selected_start;
        let selected_right = selected_start.saturating_add(selected_width);

        let highlight_left = selected_left.max(visible_left);
        let highlight_right = selected_right.min(visible_right);
        if highlight_left < highlight_right && selected_width > 0 {
            let x = bottom_inner.x + (highlight_left.saturating_sub(visible_left) as u16);
            let width = (highlight_right - highlight_left) as u16;
            let highlight = Rect {
                x,
                y: bottom_inner.y,
                width,
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

        match self.key_at(selected) {
            Some(KeyboardKey::Submit) => ScreenAction::Pop,
            Some(KeyboardKey::Letter(i)) => {
                let ch = (b'A' + i) as char;
                let _ = self.text.push(ch);
                ScreenAction::None
            }
            Some(KeyboardKey::Space) => {
                let _ = self.text.push(' ');
                ScreenAction::None
            }
            Some(KeyboardKey::Delete) => {
                let _ = self.text.pop();
                ScreenAction::None
            }
            Some(KeyboardKey::Back) => ScreenAction::Pop,
            None => ScreenAction::None,
        }
    }
}
