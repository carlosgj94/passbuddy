pub mod new_entry_form;
pub mod new_group_form;
pub mod select_entry;
pub mod select_group;
use ratatui::{Frame, widgets::ListState};

use crate::{app::ScreenAction, keepass::KeePassDb};

pub trait Screen {
    fn new() -> Self;
    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, kpdb: &KeePassDb);
    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction;
}
