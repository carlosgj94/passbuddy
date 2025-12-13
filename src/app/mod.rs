pub mod screens;
pub mod terminal;

use ratatui::widgets::ListState;

use defmt::Format;
pub use screens::new_group_form::draw as draw_new_group_form;
pub use screens::select_database::{
    ITEMS as MENU_ITEMS, LABELS as MENU_LABELS, draw as draw_group_menu,
};
pub use terminal::{init_terminal, init_terminal_with_flush};

#[derive(Debug, Format, PartialEq)]
pub enum Screens {
    SelectGroup,
    NewGroupForm,
}

pub fn initial_state() -> ListState {
    let mut state = ListState::default();
    state.select_first();
    state
}
