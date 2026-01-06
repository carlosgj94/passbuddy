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
use crate::usb_hid_queue::try_queue_type_text;

#[derive(Debug, Format)]
pub enum Screens {
    SelectGroup(screens::select_group::SelectGroupScreen),
    NewGroupForm(screens::new_group_form::NewGroupForm),
    SelectEntry(screens::select_entry::SelectEntryScreen),
    NewEntryForm(screens::new_entry_form::NewEntryFormScreen),
    EntryOptions(screens::entry_options::EntryOptionsScreen),
    TextEntryForm(screens::text_entry_form::TextEntryFormScreen),
    ActionCompleted(screens::action_completed::ActionCompletedScreen),
    BootSplash(screens::boot_splash::BootSplashScreen),
    ViewPassword(screens::view_password::ViewPasswordScreen),
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

    pub fn entry_options(entry_index: usize) -> Self {
        Self::EntryOptions(screens::entry_options::EntryOptionsScreen::new(entry_index))
    }

    pub fn text_entry_form(initial_text: &str) -> Self {
        Self::TextEntryForm(
            screens::text_entry_form::TextEntryFormScreen::new_with_text(initial_text),
        )
    }

    pub fn action_completed(message: &str) -> Self {
        Self::ActionCompleted(screens::action_completed::ActionCompletedScreen::new(
            message,
        ))
    }

    pub fn boot_splash() -> Self {
        Self::BootSplash(screens::boot_splash::BootSplashScreen::new())
    }

    pub fn view_password(entry_index: usize) -> Self {
        Self::ViewPassword(screens::view_password::ViewPasswordScreen::new(entry_index))
    }

    pub fn item_count(&self, kpdb: &KeePassDb) -> usize {
        match self {
            Screens::SelectGroup(screen) => screen.item_count(kpdb),
            Screens::NewGroupForm(_) => screens::new_group_form::ITEMS,
            Screens::SelectEntry(screen) => screen.item_count(kpdb),
            Screens::NewEntryForm(_) => screens::new_entry_form::ITEMS,
            Screens::EntryOptions(screen) => screen.item_count(kpdb),
            Screens::TextEntryForm(screen) => screen.item_count(),
            Screens::ActionCompleted(_) => 0,
            Screens::BootSplash(_) => 0,
            Screens::ViewPassword(_) => 0,
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
            Screens::EntryOptions(screen) => screen.draw(frame, selected, keepass),
            Screens::TextEntryForm(screen) => screen.draw(frame, selected, keepass),
            Screens::ActionCompleted(screen) => screen.draw(frame, selected, keepass),
            Screens::BootSplash(screen) => screen.draw(frame, selected, keepass),
            Screens::ViewPassword(screen) => screen.draw(frame, selected, keepass),
        }
    }

    fn on_select(&mut self, selected: Option<usize>) -> ScreenAction {
        match self {
            Screens::SelectGroup(screen) => screen.on_select(selected),
            Screens::NewGroupForm(screen) => screen.on_select(selected),
            Screens::SelectEntry(screen) => screen.on_select(selected),
            Screens::NewEntryForm(screen) => screen.on_select(selected),
            Screens::EntryOptions(screen) => screen.on_select(selected),
            Screens::TextEntryForm(screen) => screen.on_select(selected),
            Screens::ActionCompleted(screen) => screen.on_select(selected),
            Screens::BootSplash(screen) => screen.on_select(selected),
            Screens::ViewPassword(screen) => screen.on_select(selected),
        }
    }

    fn on_tick(&mut self) -> ScreenAction {
        match self {
            Screens::SelectGroup(screen) => screen.on_tick(),
            Screens::NewGroupForm(screen) => screen.on_tick(),
            Screens::SelectEntry(screen) => screen.on_tick(),
            Screens::NewEntryForm(screen) => screen.on_tick(),
            Screens::EntryOptions(screen) => screen.on_tick(),
            Screens::TextEntryForm(screen) => screen.on_tick(),
            Screens::ActionCompleted(screen) => screen.on_tick(),
            Screens::BootSplash(screen) => screen.on_tick(),
            Screens::ViewPassword(screen) => screen.on_tick(),
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
    ToggleEntryAutotype(usize),
    TypeEntryPassword(usize),
    DeleteEntry(usize),
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
        screen_stack[1] = Some(Screens::boot_splash());
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
        let action = self.get_current_screen_mut().on_select(selected);
        self.handle_screen_action(action, storage);
    }

    pub fn on_tick(&mut self, storage: &mut FlashStorage) {
        let action = self.get_current_screen_mut().on_tick();
        self.handle_screen_action(action, storage);
    }

    fn handle_screen_action(&mut self, action: ScreenAction, storage: &mut FlashStorage) {
        match action {
            ScreenAction::None => {}
            ScreenAction::Pop => self.pop_screen(),
            ScreenAction::Push(screen) => self.push_screen(screen),
            ScreenAction::TextEntrySubmit(text) => {
                self.pop_screen();
                match self.get_current_screen_mut() {
                    Screens::NewGroupForm(screen) => screen.set_name(text.as_str()),
                    Screens::NewEntryForm(screen) => screen.apply_text_entry_submit(text.as_str()),
                    Screens::EntryOptions(screen) => {
                        let Some(field) = screen.take_pending_field() else {
                            return;
                        };
                        let entry_index = screen.entry_index();
                        if let Some(kpdb) = self.kpdb.as_mut() {
                            let Some(existing) = kpdb
                                .entries
                                .get(entry_index)
                                .and_then(|entry| entry.as_ref())
                            else {
                                return;
                            };
                            let mut entry = *existing;
                            match field {
                                screens::entry_options::EntryField::Title => {
                                    fill_fixed(&mut entry.title, text.as_str());
                                }
                                screens::entry_options::EntryField::Username => {
                                    fill_fixed(&mut entry.username, text.as_str());
                                }
                            }
                            if let Err(err) = kpdb.update_entry(entry_index, entry, storage) {
                                warn!("update_entry failed: {}", err);
                            }
                        }
                    }
                    _ => {}
                };
            }
            ScreenAction::ToggleEntryAutotype(entry_index) => {
                let on_entry_options =
                    matches!(self.get_current_screen(), Screens::EntryOptions(_));
                if let Some(kpdb) = self.kpdb.as_mut() {
                    let Some(existing) = kpdb
                        .entries
                        .get(entry_index)
                        .and_then(|entry| entry.as_ref())
                    else {
                        return;
                    };
                    let mut entry = *existing;
                    entry.autotype = !entry.autotype;
                    if let Err(err) = kpdb.update_entry(entry_index, entry, storage) {
                        warn!("update_entry failed: {}", err);
                    }

                    if on_entry_options {
                        if let Some(updated) = kpdb
                            .entries
                            .get(entry_index)
                            .and_then(|entry| entry.as_ref())
                        {
                            let autotype_row = if updated.autotype { 4 } else { 3 };
                            self.selected.select(Some(autotype_row));
                            *self.selected.offset_mut() = 0;
                        }
                    }
                }
                self.apply_navigation(0);
            }
            ScreenAction::CreateGroup(group) => {
                let mut success = false;
                if let Some(kpdb) = self.kpdb.as_mut() {
                    success = match kpdb.create_group(group, storage) {
                        Ok(_) => true,
                        Err(err) => {
                            warn!("create_group failed: {}", err);
                            false
                        }
                    };
                }
                self.pop_screen();
                if success {
                    self.push_screen(Screens::action_completed("Group created"));
                }
            }
            ScreenAction::CreateEntry(entry) => {
                let mut success = false;
                if let Some(kpdb) = self.kpdb.as_mut() {
                    success = match kpdb.create_entry(entry, storage) {
                        Ok(_) => true,
                        Err(err) => {
                            warn!("create_entry failed: {}", err);
                            false
                        }
                    };
                }
                self.pop_screen();
                if success {
                    self.push_screen(Screens::action_completed("Entry created"));
                }
            }
            ScreenAction::TypeEntryPassword(entry_index) => {
                if let Some(kpdb) = self.kpdb.as_mut() {
                    if let Some(entry) = kpdb.entries.get(entry_index).unwrap() {
                        let pass_bytes = entry.password;

                        // Convert the pass bytes to str
                        let pass_end = pass_bytes
                            .iter()
                            .position(|&b| b == 0)
                            .unwrap_or(pass_bytes.len());
                        let Ok(pass) = core::str::from_utf8(&pass_bytes[..pass_end]) else {
                            return;
                        };
                        try_queue_type_text(pass).unwrap_or(());
                    }
                }
            }
            ScreenAction::DeleteEntry(entry_index) => {
                let mut success = false;
                if let Some(kpdb) = self.kpdb.as_mut() {
                    success = match kpdb.delete_entry(entry_index, storage) {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                }

                self.pop_screen();
                if success {
                    self.push_screen(Screens::action_completed("Entry deleted"));
                }
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

fn fill_fixed(dst: &mut [u8], value: &str) {
    dst.fill(0);
    let bytes = value.as_bytes();
    let len = bytes.len().min(dst.len());
    dst[..len].copy_from_slice(&bytes[..len]);
}
