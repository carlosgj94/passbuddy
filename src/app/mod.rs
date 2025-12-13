pub mod screens;
pub mod terminal;

use ratatui::widgets::ListState;

pub use screens::new_group_form::{
    ITEMS as NEW_GROUP_FORM_ITEMS, LABELS as NEW_GROUP_FORM_LABELS, draw as draw_new_group_form,
};
pub use screens::select_database::{ITEMS as MENU_ITEMS, LABELS as MENU_LABELS, draw as draw_menu};
pub use terminal::{init_terminal, init_terminal_with_flush};

pub fn initial_state() -> ListState {
    let mut state = ListState::default();
    state.select_first();
    state
}
