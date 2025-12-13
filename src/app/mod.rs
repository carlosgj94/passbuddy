pub mod screens;
pub mod terminal;

use ratatui::widgets::ListState;

use defmt::Format;
pub use screens::new_group_form::draw as draw_new_group_form;
pub use screens::select_database::{
    ITEMS as MENU_ITEMS, LABELS as MENU_LABELS, draw as draw_group_menu,
};
pub use terminal::{init_terminal, init_terminal_with_flush};

#[derive(Debug, Format, PartialEq, Clone, Copy)]
pub enum Screens {
    SelectGroup,
    NewGroupForm,
}

pub fn initial_state() -> ListState {
    let mut state = ListState::default();
    state.select_first();
    state
}

#[derive(Debug)]
pub struct AppState {
    pub screen_stack: [Option<Screens>; 8],
    pub selected: ListState,
}

impl AppState {
    pub fn new() -> Self {
        let mut screen_stack = [None; 8];
        screen_stack[0] = Some(Screens::SelectGroup);
        let mut selected = ListState::default();
        selected.select_first();

        Self {
            screen_stack,
            selected,
        }
    }

    pub fn select_next(&mut self) {
        self.selected.select_next();
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected.selected()
    }

    pub fn selected_mut(&mut self) -> &mut ListState {
        &mut self.selected
    }

    pub fn get_current_screen(&self) -> Option<Screens> {
        for i in (0..self.screen_stack.len()).rev() {
            if self.screen_stack[i].is_some() {
                return self.screen_stack[i];
            }
        }
        None
    }

    pub fn push_screen(&mut self, screen: Screens) {
        // Find the next empty slot
        for i in 0..self.screen_stack.len() {
            if self.screen_stack[i].is_none() {
                self.screen_stack[i] = Some(screen);
                return;
            }
        }
    }

    pub fn pop_screen(&mut self) {
        for i in (0..self.screen_stack.len()).rev() {
            if self.screen_stack[i].is_some() {
                self.screen_stack[i] = None;
                return;
            }
        }
    }
}
