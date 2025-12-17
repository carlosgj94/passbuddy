use defmt::Format;
use esp_hal::rng::Rng;
use heapless::String;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::screens::Screen;
use crate::app::screens::text_entry_form::MAX_TEXT_LEN;
use crate::app::{ScreenAction, Screens};
use crate::keepass::{Entry, KeePassDb};

pub const ITEMS: usize = 4;
pub const LABELS: [&str; ITEMS] = ["Title", "Username", "Create", "Back"];
const PASSWORD_LEN: usize = 24;
const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

#[derive(Clone, Copy, Debug, Format, Eq, PartialEq)]
enum EntryField {
    Title,
    Username,
}

#[derive(Debug, Format)]
pub struct NewEntryFormScreen {
    pub group_id: Option<u32>,
    title: String<MAX_TEXT_LEN>,
    username: String<MAX_TEXT_LEN>,
    pending_field: Option<EntryField>,
}

impl NewEntryFormScreen {
    pub fn new(group_id: Option<u32>) -> Self {
        Self {
            group_id,
            title: String::new(),
            username: String::new(),
            pending_field: None,
        }
    }

    pub fn apply_text_entry_submit(&mut self, value: &str) {
        match self.pending_field.take() {
            Some(EntryField::Title) => {
                self.title.clear();
                let _ = self.title.push_str(value);
            }
            Some(EntryField::Username) => {
                self.username.clear();
                let _ = self.username.push_str(value);
            }
            None => {}
        }
    }

    fn entry_from_form(&self) -> Entry {
        let mut entry = Entry::default_with_group_id(self.group_id.unwrap_or(0));
        Self::fill_random_password(&mut entry.password);

        if !self.title.is_empty() {
            entry.title.fill(0);
            let bytes = self.title.as_bytes();
            let len = bytes.len().min(entry.title.len());
            entry.title[..len].copy_from_slice(&bytes[..len]);
        }

        if !self.username.is_empty() {
            entry.username.fill(0);
            let bytes = self.username.as_bytes();
            let len = bytes.len().min(entry.username.len());
            entry.username[..len].copy_from_slice(&bytes[..len]);
        }

        entry
    }

    fn fill_random_password(dst: &mut [u8; 64]) {
        dst.fill(0);

        let rng = Rng::new();
        let charset = PASSWORD_CHARSET;
        let m = charset.len() as u16;
        let zone = 256u16 - (256u16 % m);

        let out_len = PASSWORD_LEN.min(dst.len().saturating_sub(1));
        for byte in dst.iter_mut().take(out_len) {
            let mut raw = [0u8; 1];
            loop {
                rng.read(&mut raw);
                if (raw[0] as u16) < zone {
                    break;
                }
            }
            *byte = charset[(raw[0] as usize) % charset.len()];
        }
    }
}

impl Screen for NewEntryFormScreen {
    fn new() -> Self {
        Self::new(None)
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, _: &KeePassDb) {
        let outer_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" New Entry ");

        let list = List::new(LABELS)
            .block(outer_block)
            .style(Style::new())
            .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), selected);
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        match selected {
            Some(0) => {
                self.pending_field = Some(EntryField::Title);
                ScreenAction::Push(Screens::text_entry_form(self.title.as_str()))
            }
            Some(1) => {
                self.pending_field = Some(EntryField::Username);
                ScreenAction::Push(Screens::text_entry_form(self.username.as_str()))
            }
            Some(2) => ScreenAction::CreateEntry(self.entry_from_form()),
            Some(3) => ScreenAction::Pop,
            _ => ScreenAction::None,
        }
    }
}
