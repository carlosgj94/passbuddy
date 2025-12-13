use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

pub const ITEMS: usize = 3;
pub const LABELS: [&str; ITEMS] = ["Name", "Icon", "Back"];

pub fn draw(frame: &mut Frame, state: &mut ListState) {
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
