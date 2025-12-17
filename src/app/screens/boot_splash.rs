use defmt::Format;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{ListState, Paragraph};

use crate::app::ScreenAction;
use crate::app::screens::Screen;
use crate::keepass::KeePassDb;

pub const DEFAULT_TTL_FRAMES: u16 = 20;
const ANIM_STEP_FRAMES: u16 = 3;

const LOGO_FRAMES: [&str; 4] = [
    "#######\n\
##     ##\n\
##     ##\n\
###########\n\
##       ##\n\
###########",
    "#######\n\
##     ##\n\
##     ##\n\
###########\n\
##   #   ##\n\
###########",
    "#######\n\
##     ##\n\
##     ##\n\
###########\n\
##  ###  ##\n\
###########",
    "#######\n\
##     ##\n\
##     ##\n\
###########\n\
##  ###  ##\n\
###########",
];

#[derive(Debug, Format)]
pub struct BootSplashScreen {
    frame: u16,
    ttl_frames: u16,
}

impl BootSplashScreen {
    pub fn new() -> Self {
        Self::new_with_ttl(DEFAULT_TTL_FRAMES)
    }

    pub fn new_with_ttl(ttl_frames: u16) -> Self {
        Self {
            frame: 0,
            ttl_frames,
        }
    }

    fn logo_frame(&self) -> &'static str {
        let step = (self.frame / ANIM_STEP_FRAMES) as usize;
        let idx = step.min(LOGO_FRAMES.len().saturating_sub(1));
        LOGO_FRAMES[idx]
    }
}

impl Screen for BootSplashScreen {
    fn new() -> Self {
        Self::new()
    }

    fn draw(&mut self, frame: &mut Frame, _: &mut ListState, _: &KeePassDb) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }

        let complete = (self.frame / ANIM_STEP_FRAMES) as usize >= LOGO_FRAMES.len() - 1;
        let flash_on = complete && ((self.frame / 4) % 2 == 0);
        let style = if flash_on {
            Style::new().bold().bg(Color::White).fg(Color::Black)
        } else {
            Style::new().bold().fg(Color::White)
        };

        let logo = Paragraph::new(self.logo_frame())
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(logo, area);
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
