use defmt::Format;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::screens::Screen;
use crate::app::{ScreenAction, Screens};

pub const ITEMS: usize = 4;
pub const LABELS: [&str; ITEMS] = ["Personal", "Work", "Shared", "New group"];

#[derive(Debug, Format)]
pub struct SelectDatabaseScreen;

impl Screen for SelectDatabaseScreen {
    fn draw(&self, frame: &mut Frame, state: &mut ListState) {
        let outer_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" Select group ");

        let list = List::new(LABELS)
            .block(outer_block)
            .style(Style::new())
            .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
            .highlight_symbol(">> ");
        frame.render_stateful_widget(list, frame.area(), state);
    }
    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        match selected {
            Some(3) => ScreenAction::Push(Screens::new_group_form()),
            _ => ScreenAction::None,
        }
    }
}
