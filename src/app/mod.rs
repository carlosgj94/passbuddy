pub mod screens;
pub mod terminal;

use esp_storage::FlashStorage;
use ratatui::Frame;
use ratatui::widgets::ListState;

use defmt::{Format, warn};
pub use screens::select_group::ITEMS as MENU_ITEMS;
pub use terminal::{init_terminal, init_terminal_with_flush};

use screens::Screen;

use crate::keepass::{Entry, Group, KeePassDb};

#[derive(Debug, Format)]
pub enum Screens {
    SelectGroup(screens::select_group::SelectGroupScreen),
    NewGroupForm(screens::new_group_form::NewGroupForm),
    SelectEntry(screens::select_entry::SelectEntryScreen),
    NewEntryForm(screens::new_entry_form::NewEntryFormScreen),
    TextEntryForm(screens::text_entry_form::TextEntryFormScreen),
}

impl Screens {
    pub fn select_group() -> Self {
        Self::SelectGroup(screens::select_group::SelectGroupScreen::new())
    }

    pub fn new_group_form() -> Self {
        Self::NewGroupForm(screens::new_group_form::NewGroupForm::new())
    }

    pub fn select_entry(group_id: u32) -> Self {
        Self::SelectEntry(screens::select_entry::SelectEntryScreen::new(Some(
            group_id,
        )))
    }

    pub fn new_entry_form(group_id: u32) -> Self {
        Self::NewEntryForm(screens::new_entry_form::NewEntryFormScreen::new(Some(
            group_id,
        )))
    }

    pub fn text_entry_form() -> Self {
        Self::TextEntryForm(screens::text_entry_form::TextEntryFormScreen::new())
    }
}

impl Screen for Screens {
    fn new() -> Self {
        Self::SelectGroup(screens::select_group::SelectGroupScreen::new())
    }

    fn draw(&mut self, frame: &mut Frame, selected: &mut ListState, keepass: &KeePassDb) {
        match self {
            Screens::SelectGroup(screen) => screen.draw(frame, selected, keepass),
            Screens::NewGroupForm(screen) => screen.draw(frame, selected, keepass),
            Screens::SelectEntry(screen) => screen.draw(frame, selected, keepass),
            Screens::NewEntryForm(screen) => screen.draw(frame, selected, keepass),
            Screens::TextEntryForm(screen) => screen.draw(frame, selected, keepass),
        }
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        match self {
            Screens::SelectGroup(screen) => screen.on_select(selected),
            Screens::NewGroupForm(screen) => screen.on_select(selected),
            Screens::SelectEntry(screen) => screen.on_select(selected),
            Screens::NewEntryForm(screen) => screen.on_select(selected),
            Screens::TextEntryForm(screen) => screen.on_select(selected),
        }
    }
}

#[derive(Debug)]
pub enum ScreenAction {
    None,
    Push(Screens),
    Pop,
    CreateGroup(Group),
    CreateEntry(Entry),
}

#[derive(Debug)]
pub struct AppState {
    pub screen_stack: [Option<Screens>; 8],
    pub selected: ListState,
    pub kpdb: Option<KeePassDb>,
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
            kpdb: None,
        }
    }
    pub fn with_kpdb(mut self, kpdb: KeePassDb) -> Self {
        self.kpdb = Some(kpdb);
        self
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

    pub fn kpdb(&self) -> Option<&KeePassDb> {
        self.kpdb.as_ref()
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
        let kpdb = self.kpdb.as_ref().unwrap();
        let screen = self.screen_stack[idx]
            .as_mut()
            .unwrap_or_else(|| unreachable!("current_screen_index returned empty slot"));

        // TODO: avoid cloning `selected`; `render_stateful_widget` updates offsets.
        screen.draw(frame, &mut self.selected.clone(), kpdb);
    }

    pub fn on_select(&mut self, storage: &mut FlashStorage) {
        let selected = self.selected();
        match self.get_current_screen_mut().on_select(selected) {
            ScreenAction::None => {}
            ScreenAction::Pop => self.pop_screen(),
            ScreenAction::Push(screen) => self.push_screen(screen),
            ScreenAction::CreateGroup(group) => {
                if let Some(kpdb) = self.kpdb.as_mut() {
                    if let Err(err) = kpdb.create_group(group, storage) {
                        warn!("create_group failed: {}", err);
                    }
                }
                self.pop_screen();
            }
            ScreenAction::CreateEntry(entry) => {
                if let Some(kpdb) = self.kpdb.as_mut() {
                    if let Err(err) = kpdb.create_entry(entry, storage) {
                        warn!("create_entry failed: {}", err);
                    }
                }
                self.pop_screen();
            }
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
