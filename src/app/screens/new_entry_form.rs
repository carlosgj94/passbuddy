use defmt::Format;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::ScreenAction;
use crate::app::screens::Screen;
use crate::keepass::KeePassDb;

pub const ITEMS: usize = 5;
pub const LABELS: [&str; ITEMS] = ["Title", "Username", "Password", "Create", "Back"];

#[derive(Debug, Format)]
pub struct NewEntryFormScreen {
    pub group_id: Option<u32>,
}

impl NewEntryFormScreen {
    pub fn new(group_id: Option<u32>) -> Self {
        Self { group_id }
    }
}

impl Screen for NewEntryFormScreen {
    fn new() -> Self {
        Self { group_id: None }
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
            Some(4) => ScreenAction::None,
            Some(5) => ScreenAction::Pop,
            _ => ScreenAction::None,
        }
    }
}
