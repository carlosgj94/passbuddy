pub mod action_completed;
pub mod entry_options;
pub mod new_entry_form;
pub mod new_group_form;
pub mod select_entry;
pub mod select_group;
pub mod text_entry_form;
use ratatui::{Frame, widgets::ListState};

use crate::{app::ScreenAction, keepass::KeePassDb};

pub trait Screen {
    fn new() -> Self;
    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, kpdb: &KeePassDb);
    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction;
    fn on_tick(&mut self) -> ScreenAction {
        ScreenAction::None
    }
}
