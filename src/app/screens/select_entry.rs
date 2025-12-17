use defmt::Format;
use heapless::Vec;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};

use crate::app::screens::Screen;
use crate::app::{ScreenAction, Screens};
use crate::keepass::KeePassDb;

pub const ITEMS: usize = 130; // Create entry + up to 128 entries + Back

#[derive(Debug, Format)]
pub struct SelectEntryScreen {
    group_id: Option<u32>,
    back_position: usize,
    initial_selection_applied: bool,
    last_rendered_selected: Option<usize>,
}

impl SelectEntryScreen {
    pub fn new(group_id: Option<u32>) -> Self {
        Self {
            group_id,
            back_position: 0,
            initial_selection_applied: false,
            last_rendered_selected: None,
        }
    }

    pub fn item_count(&self, keepass: &KeePassDb) -> usize {
        let mut count = 0usize;

        if let Some(group_id) = self.group_id {
            count = count.saturating_add(1); // Create entry

            for entry in keepass.entries.iter().filter_map(|entry| entry.as_ref()) {
                if entry.group_id != group_id {
                    continue;
                }
                // Reserve the last slot for "Back".
                if count + 1 >= ITEMS {
                    break;
                }

                count = count.saturating_add(1);
            }
        }

        // Back
        count = count.saturating_add(1);

        count.min(ITEMS)
    }
}

impl Screen for SelectEntryScreen {
    fn new() -> Self {
        Self {
            group_id: None,
            back_position: 0,
            initial_selection_applied: false,
            last_rendered_selected: None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, keepass: &KeePassDb) {
        let outer_block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(" Select entry ");

        let mut items: Vec<&str, ITEMS> = Vec::new();
        if let Some(group_id) = self.group_id {
            let _ = items.push("Create entry");

            for entry in keepass.entries.iter().filter_map(|entry| entry.as_ref()) {
                if entry.group_id != group_id {
                    continue;
                }
                // Reserve the last slot for "Back".
                if items.len() + 1 >= ITEMS {
                    break;
                }

                let title = &entry.title;
                let end = title.iter().position(|&b| b == 0).unwrap_or(title.len());
                let label = match core::str::from_utf8(&title[..end]) {
                    Ok("") => "<untitled>",
                    Ok(label) => label,
                    Err(_) => "<invalid utf8>",
                };

                let _ = items.push(label);
            }
        }
        let _ = items.push("Back");
        self.back_position = items.len().saturating_sub(1);

        if !self.initial_selection_applied {
            let initial = if items.len() > 1 { Some(1) } else { Some(0) };
            selected.select(initial);
            self.initial_selection_applied = true;
        }
        self.last_rendered_selected = selected.selected();

        let list = List::new(items)
            .block(outer_block)
            .style(Style::new())
            .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
            .highlight_symbol(">> ");
        frame.render_stateful_widget(list, frame.area(), selected);
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        let selected = self.last_rendered_selected.or(selected);
        if selected == Some(0) {
            return ScreenAction::Push(Screens::new_entry_form(selected.unwrap() as u32));
        }
        let Some(selected) = selected else {
            return ScreenAction::None;
        };

        if selected == self.back_position {
            return ScreenAction::Pop;
        }

        ScreenAction::None
    }
}
