use defmt::Format;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::ScreenAction;
use crate::app::screens::Screen;

pub const ITEMS: usize = 3;
pub const LABELS: [&str; ITEMS] = ["Name", "Icon", "Back"];

#[derive(Debug, Format)]
pub struct NewGroupForm;

impl Screen for NewGroupForm {
    fn draw(&self, frame: &mut Frame, state: &mut ListState) {
        let outer_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" New Group ");

        let list = List::new(LABELS)
            .block(outer_block)
            .style(Style::new())
            .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), state);
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        match selected {
            Some(2) => ScreenAction::Pop,
            _ => ScreenAction::None,
        }
    }
}
