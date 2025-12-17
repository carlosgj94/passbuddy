use defmt::Format;
use heapless::String;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{ListState, Paragraph};

use crate::app::ScreenAction;
use crate::app::screens::Screen;
use crate::keepass::KeePassDb;

pub const MAX_MESSAGE_LEN: usize = 32;
pub const DEFAULT_TTL_FRAMES: u16 = 20;

const ANIM_STEP_FRAMES: u16 = 2;

const CHECK_FRAMES: [&str; 4] = [
    "         \n         \n         \n         ",
    "        ##\n         \n         \n         ",
    "        ##\n       ## \n         \n         ",
    "        ##\n       ## \n ##   ##  \n  ####    ",
];

#[derive(Debug, Format)]
pub struct ActionCompletedScreen {
    message: String<MAX_MESSAGE_LEN>,
    frame: u16,
    ttl_frames: u16,
}

impl ActionCompletedScreen {
    pub fn new(message: &str) -> Self {
        Self::new_with_ttl(message, DEFAULT_TTL_FRAMES)
    }

    pub fn new_with_ttl(message: &str, ttl_frames: u16) -> Self {
        let mut stored = String::new();
        for ch in message.chars() {
            if stored.push(ch).is_err() {
                break;
            }
        }

        Self {
            message: stored,
            frame: 0,
            ttl_frames,
        }
    }

    fn art_frame(&self) -> &'static str {
        let step = (self.frame / ANIM_STEP_FRAMES) as usize;
        let idx = step.min(CHECK_FRAMES.len().saturating_sub(1));
        CHECK_FRAMES[idx]
    }
}

impl Screen for ActionCompletedScreen {
    fn new() -> Self {
        Self::new("Done")
    }

    fn draw(&mut self, frame: &mut Frame, _: &mut ListState, _: &KeePassDb) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let complete = (self.frame / ANIM_STEP_FRAMES) as usize >= CHECK_FRAMES.len() - 1;
        let flash_on = complete && ((self.frame / 4) % 2 == 0);
        let art_style = if flash_on {
            Style::new().bold().bg(Color::White).fg(Color::Black)
        } else {
            Style::new().bold().fg(Color::Green)
        };

        let art = Paragraph::new(self.art_frame())
            .alignment(Alignment::Center)
            .style(art_style);
        if !chunks[0].is_empty() {
            frame.render_widget(art, chunks[0]);
        }

        let message = Paragraph::new(self.message.as_str())
            .alignment(Alignment::Center)
            .style(Style::new().bold());
        if !chunks[1].is_empty() {
            frame.render_widget(message, chunks[1]);
        }
    }

    fn on_select(&mut self, _: Option<usize>) -> ScreenAction {
        ScreenAction::Pop
    }

    fn on_tick(&mut self) -> ScreenAction {
        self.frame = self.frame.saturating_add(1);
        if self.frame > self.ttl_frames {
            ScreenAction::Pop
        } else {
            ScreenAction::None
        }
    }
}
