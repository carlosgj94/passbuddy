pub mod screens;
pub mod terminal;

use esp_storage::FlashStorage;
use heapless::String;
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

    pub fn text_entry_form(initial_text: &str) -> Self {
        Self::TextEntryForm(
            screens::text_entry_form::TextEntryFormScreen::new_with_text(initial_text),
        )
    }

    pub fn item_count(&self, kpdb: &KeePassDb) -> usize {
        match self {
            Screens::SelectGroup(screen) => screen.item_count(kpdb),
            Screens::NewGroupForm(_) => screens::new_group_form::ITEMS,
            Screens::SelectEntry(screen) => screen.item_count(kpdb),
            Screens::NewEntryForm(_) => screens::new_entry_form::ITEMS,
            Screens::TextEntryForm(screen) => screen.item_count(),
        }
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
    TextEntrySubmit(String<{ screens::text_entry_form::MAX_TEXT_LEN }>),
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

    /// Applies a rotary navigation delta to the current menu selection.
    ///
    /// The selection is clamped to the valid item range for the current screen.
    pub fn apply_navigation(&mut self, delta: i16) {
        let item_count = match self.kpdb.as_ref() {
            Some(kpdb) => self.get_current_screen().item_count(kpdb),
            None => 0,
        };

        if item_count == 0 {
            self.selected.select(None);
            return;
        }

        let current = self.selected.selected().unwrap_or(0) as u64;
        let current = current.min(i64::MAX as u64) as i64;
        let mut next = current.saturating_add(i64::from(delta));
        let max = item_count.saturating_sub(1) as i64;

        if next < 0 {
            next = 0;
        } else if next > max {
            next = max;
        }

        self.selected.select(Some(next as usize));
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

        screen.draw(frame, &mut self.selected, kpdb);
    }

    pub fn on_select(&mut self, storage: &mut FlashStorage) {
        // Ensure the selection is valid for the current screen.
        self.apply_navigation(0);
        let selected = self.selected();
        match self.get_current_screen_mut().on_select(selected) {
            ScreenAction::None => {}
            ScreenAction::Pop => self.pop_screen(),
            ScreenAction::Push(screen) => self.push_screen(screen),
            ScreenAction::TextEntrySubmit(text) => {
                self.pop_screen();
                match self.get_current_screen_mut() {
                    Screens::NewGroupForm(screen) => screen.set_name(text.as_str()),
                    Screens::NewEntryForm(screen) => screen.apply_text_entry_submit(text.as_str()),
                    _ => {}
                };
            }
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
                self.selected.select_first();
                *self.selected.offset_mut() = 0;
                return;
            }
        }
    }

    fn pop_screen(&mut self) {
        for i in (1..self.screen_stack.len()).rev() {
            if self.screen_stack[i].is_some() {
                self.screen_stack[i] = None;
                self.selected.select_first();
                *self.selected.offset_mut() = 0;
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
