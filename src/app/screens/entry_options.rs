use defmt::Format;
use heapless::String;
use heapless::Vec;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::screens::Screen;
use crate::app::screens::text_entry_form::MAX_TEXT_LEN;
use crate::app::{ScreenAction, Screens};
use crate::keepass::KeePassDb;

pub const ITEMS: usize = 7;
const AUTOTYPE_LABEL_CAP: usize = 20;

#[derive(Clone, Copy, Debug, Format, Eq, PartialEq)]
pub enum EntryField {
    Title,
    Username,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EntryOption {
    TypePassword,
    ChangeName,
    ChangeUsername,
    ViewPassword,
    ToggleAutotype,
    Back,
    DeleteEntry,
}

#[derive(Debug, Format)]
pub struct EntryOptionsScreen {
    entry_index: usize,
    autotype: bool,
    title: String<MAX_TEXT_LEN>,
    username: String<MAX_TEXT_LEN>,
    autotype_label: String<AUTOTYPE_LABEL_CAP>,
    pending_field: Option<EntryField>,
    entry_present: bool,
}

impl EntryOptionsScreen {
    pub fn new(entry_index: usize) -> Self {
        Self {
            entry_index,
            autotype: false,
            title: String::new(),
            username: String::new(),
            autotype_label: String::new(),
            pending_field: None,
            entry_present: false,
        }
    }

    pub fn item_count(&self, kpdb: &KeePassDb) -> usize {
        let Some(entry) = kpdb
            .entries
            .get(self.entry_index)
            .and_then(|entry| entry.as_ref())
        else {
            return 1;
        };

        let mut count = 6usize; // name, username, view, autotype, back, delete
        if entry.autotype {
            count = count.saturating_add(1);
        }

        count.min(ITEMS)
    }

    pub fn entry_index(&self) -> usize {
        self.entry_index
    }

    pub fn take_pending_field(&mut self) -> Option<EntryField> {
        self.pending_field.take()
    }

    fn option_at(&self, index: usize) -> Option<EntryOption> {
        let mut idx = index;

        if self.autotype {
            if idx == 0 {
                return Some(EntryOption::TypePassword);
            }
            idx = idx.saturating_sub(1);
        }

        match idx {
            0 => Some(EntryOption::ChangeName),
            1 => Some(EntryOption::ChangeUsername),
            2 => Some(EntryOption::ViewPassword),
            3 => Some(EntryOption::ToggleAutotype),
            4 => Some(EntryOption::Back),
            5 => Some(EntryOption::DeleteEntry),
            _ => None,
        }
    }

    fn sync_text(dst: &mut String<MAX_TEXT_LEN>, src: &[u8]) {
        dst.clear();
        let end = src.iter().position(|&b| b == 0).unwrap_or(src.len());
        let Ok(text) = core::str::from_utf8(&src[..end]) else {
            return;
        };
        let _ = dst.push_str(text);
    }

    fn sync_from_entry(&mut self, kpdb: &KeePassDb) {
        let Some(entry) = kpdb
            .entries
            .get(self.entry_index)
            .and_then(|entry| entry.as_ref())
        else {
            self.entry_present = false;
            self.autotype = false;
            self.title.clear();
            self.username.clear();
            self.autotype_label.clear();
            return;
        };

        self.entry_present = true;
        self.autotype = entry.autotype;
        Self::sync_text(&mut self.title, &entry.title);
        Self::sync_text(&mut self.username, &entry.username);

        self.autotype_label.clear();
        let _ = self.autotype_label.push_str("Autotype: ");
        let _ = self
            .autotype_label
            .push_str(if self.autotype { "true" } else { "false" });
    }
}

impl Screen for EntryOptionsScreen {
    fn new() -> Self {
        Self::new(0)
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, kpdb: &KeePassDb) {
        self.sync_from_entry(kpdb);

        let mut title_padded: String<{ MAX_TEXT_LEN + 2 }> = String::new();
        if self.entry_present && !self.title.is_empty() {
            let _ = title_padded.push(' ');
            let _ = title_padded.push_str(self.title.as_str());
            let _ = title_padded.push(' ');
        } else {
            let _ = title_padded.push_str(" Entry ");
        }

        let outer_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(title_padded.as_str());

        let mut items: Vec<&str, ITEMS> = Vec::new();
        if self.entry_present {
            if self.autotype {
                let _ = items.push("Type password");
            }
            let _ = items.push("Change name");
            let _ = items.push("Change username");
            let _ = items.push("View password");
            let _ = items.push(self.autotype_label.as_str());
        }
        let _ = items.push("Back");

        if self.entry_present {
            let _ = items.push("Delete entry");
        }

        let list = List::new(items)
            .block(outer_block)
            .style(Style::new())
            .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), selected);
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        let Some(selected) = selected else {
            return ScreenAction::None;
        };

        if !self.entry_present {
            return ScreenAction::Pop;
        }

        match self.option_at(selected) {
            Some(EntryOption::TypePassword) => ScreenAction::TypeEntryPassword(self.entry_index),
            Some(EntryOption::ChangeName) => {
                self.pending_field = Some(EntryField::Title);
                ScreenAction::Push(Screens::text_entry_form(self.title.as_str()))
            }
            Some(EntryOption::ChangeUsername) => {
                self.pending_field = Some(EntryField::Username);
                ScreenAction::Push(Screens::text_entry_form(self.username.as_str()))
            }
            Some(EntryOption::ViewPassword) => {
                ScreenAction::Push(Screens::view_password(self.entry_index))
            }
            Some(EntryOption::ToggleAutotype) => {
                ScreenAction::ToggleEntryAutotype(self.entry_index)
            }
            Some(EntryOption::Back) => ScreenAction::Pop,
            Some(EntryOption::DeleteEntry) => ScreenAction::DeleteEntry(self.entry_index),
            None => ScreenAction::None,
        }
    }
}
