pub mod screens;
pub mod terminal;

use ratatui::Frame;
use ratatui::widgets::ListState;

use defmt::Format;
pub use screens::select_database::{ITEMS as MENU_ITEMS, LABELS as MENU_LABELS};
pub use terminal::{init_terminal, init_terminal_with_flush};

use screens::Screen;

#[derive(Debug, Format)]
pub enum Screens {
    SelectGroup(screens::select_database::SelectDatabaseScreen),
    NewGroupForm(screens::new_group_form::NewGroupForm),
}

impl Screens {
    pub fn select_group() -> Self {
        Self::SelectGroup(screens::select_database::SelectDatabaseScreen)
    }

    pub fn new_group_form() -> Self {
        Self::NewGroupForm(screens::new_group_form::NewGroupForm)
    }
}

impl Screen for Screens {
    fn draw(&self, frame: &mut Frame, state: &mut ListState) {
        match self {
            Screens::SelectGroup(screen) => screen.draw(frame, state),
            Screens::NewGroupForm(screen) => screen.draw(frame, state),
        }
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        match self {
            Screens::SelectGroup(screen) => screen.on_select(selected),
            Screens::NewGroupForm(screen) => screen.on_select(selected),
        }
    }
}

#[derive(Debug)]
pub enum ScreenAction {
    None,
    Push(Screens),
    Pop,
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
        let mut screen_stack: [Option<Screens>; 8] = core::array::from_fn(|_| None);
        screen_stack[0] = Some(Screens::select_group());
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

    fn current_screen_index(&self) -> usize {
        for i in (0..self.screen_stack.len()).rev() {
            if self.screen_stack[i].is_some() {
                return i;
            }
        }

        unreachable!("screen stack must never be empty");
    }

    pub fn get_current_screen(&self) -> &Screens {
        let idx = self.current_screen_index();
        self.screen_stack[idx]
            .as_ref()
            .unwrap_or_else(|| unreachable!("current_screen_index returned empty slot"))
    }

    pub fn get_current_screen_mut(&mut self) -> &mut Screens {
        let idx = self.current_screen_index();
        self.screen_stack[idx]
            .as_mut()
            .unwrap_or_else(|| unreachable!("current_screen_index returned empty slot"))
    }

    pub fn draw_current_screen(&mut self, frame: &mut Frame) {
        let idx = self.current_screen_index();
        let screen = self.screen_stack[idx]
            .as_ref()
            .unwrap_or_else(|| unreachable!("current_screen_index returned empty slot"));
        screen.draw(frame, &mut self.selected);
    }

    pub fn on_select(&mut self) {
        let selected = self.selected();
        match self.get_current_screen_mut().on_select(selected) {
            ScreenAction::None => {}
            ScreenAction::Pop => self.pop_screen(),
            ScreenAction::Push(screen) => self.push_screen(screen),
        }
    }

    fn push_screen(&mut self, screen: Screens) {
        // Find the next empty slot
        for i in 0..self.screen_stack.len() {
            if self.screen_stack[i].is_none() {
                self.screen_stack[i] = Some(screen);
                return;
            }
        }
    }

    fn pop_screen(&mut self) {
        for i in (1..self.screen_stack.len()).rev() {
            if self.screen_stack[i].is_some() {
                self.screen_stack[i] = None;
                return;
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
