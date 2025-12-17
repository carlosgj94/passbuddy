use defmt::Format;
use heapless::String;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::screens::Screen;
use crate::app::screens::text_entry_form::MAX_TEXT_LEN;
use crate::app::{ScreenAction, Screens};
use crate::keepass::{Group, KeePassDb};

pub const ITEMS: usize = 4;
pub const LABELS: [&str; ITEMS] = ["Name", "Icon", "Create", "Back"];

#[derive(Debug, Format)]
pub struct NewGroupForm {
    name: String<MAX_TEXT_LEN>,
}

impl NewGroupForm {
    pub fn set_name(&mut self, value: &str) {
        self.name.clear();
        let _ = self.name.push_str(value);
    }

    fn group_from_form(&self) -> Group {
        let mut group = Group::random();
        if self.name.is_empty() {
            return group;
        }

        group.name.fill(0);
        let bytes = self.name.as_bytes();
        let len = bytes.len().min(group.name.len());
        group.name[..len].copy_from_slice(&bytes[..len]);
        group
    }
}

impl Screen for NewGroupForm {
    fn new() -> Self {
        Self {
            name: String::new(),
        }
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, _: &KeePassDb) {
        let outer_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" New Group ");

        let list = List::new(LABELS)
            .block(outer_block)
            .style(Style::new())
            .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), selected);
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        match selected {
            Some(0) => ScreenAction::Push(Screens::text_entry_form(self.name.as_str())),
            Some(2) => ScreenAction::CreateGroup(self.group_from_form()),
            Some(3) => ScreenAction::Pop,
            _ => ScreenAction::None,
        }
    }
}
