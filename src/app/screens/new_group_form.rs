use defmt::Format;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::screens::Screen;
use crate::app::{ScreenAction, Screens};
use crate::keepass::{Group, KeePassDb};

pub const ITEMS: usize = 4;
pub const LABELS: [&str; ITEMS] = ["Name", "Icon", "Create", "Back"];

#[derive(Debug, Format)]
pub struct NewGroupForm;

impl Screen for NewGroupForm {
    fn new() -> Self {
        Self
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
            Some(0) => ScreenAction::Push(Screens::text_entry_form()),
            Some(1) => ScreenAction::Push(Screens::text_entry_form()),
            Some(2) => ScreenAction::CreateGroup(Group::random()),
            Some(3) => ScreenAction::Pop,
            _ => ScreenAction::None,
        }
    }
}
