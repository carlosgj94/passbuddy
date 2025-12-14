pub mod new_group_form;
pub mod select_database;
use ratatui::{Frame, widgets::ListState};

use crate::app::ScreenAction;

pub trait Screen {
    fn draw(&self, frame: &mut Frame, state: &mut ListState);
    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction;
}
