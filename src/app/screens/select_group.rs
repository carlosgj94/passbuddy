use defmt::Format;
use heapless::Vec;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::screens::Screen;
use crate::app::{ScreenAction, Screens};
use crate::keepass::KeePassDb;

pub const ITEMS: usize = 4;

#[derive(Debug, Format)]
pub struct SelectGroupScreen {
    new_group_position: Option<usize>,
}

impl Screen for SelectGroupScreen {
    fn new() -> Self {
        Self {
            new_group_position: None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, keepass: &KeePassDb) {
        let outer_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" Select group ");

        self.new_group_position = None;

        let num_groups = (keepass.header.num_groups as usize).min(ITEMS);
        let mut items: Vec<&str, ITEMS> = Vec::new();
        for i in 0..num_groups {
            let Some(group) = keepass.groups[i].as_ref() else {
                break;
            };

            let name = &group.name;
            let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
            let Ok(label) = core::str::from_utf8(&name[..end]) else {
                continue;
            };
            if label.is_empty() {
                continue;
            }

            let _ = items.push(label);
        }

        if items.len() < ITEMS {
            self.new_group_position = Some(items.len());
            let _ = items.push("New group");
        }

        let list = List::new(items)
            .block(outer_block)
            .style(Style::new())
            .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
            .highlight_symbol(">> ");
        frame.render_stateful_widget(list, frame.area(), selected);
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        if selected.is_some() && selected == self.new_group_position {
            ScreenAction::Push(Screens::new_group_form())
        } else {
            ScreenAction::Push(Screens::select_entry(1))
        }
    }
}
