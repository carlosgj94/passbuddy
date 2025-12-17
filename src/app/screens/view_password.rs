use defmt::Format;
use heapless::String;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::widgets::{Block, ListState, Paragraph, Wrap};

use crate::app::ScreenAction;
use crate::app::screens::Screen;
use crate::keepass::KeePassDb;

const MAX_TITLE_LEN: usize = 32;
const MAX_PASSWORD_LEN: usize = 64;

#[derive(Debug, Format)]
pub struct ViewPasswordScreen {
    entry_index: usize,
}

impl ViewPasswordScreen {
    pub fn new(entry_index: usize) -> Self {
        Self { entry_index }
    }

    fn bytes_to_string<const N: usize>(bytes: &[u8]) -> String<N> {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let Ok(text) = core::str::from_utf8(&bytes[..end]) else {
            let mut out: String<N> = String::new();
            let _ = out.push_str("<invalid>");
            return out;
        };

        if text.is_empty() {
            let mut out: String<N> = String::new();
            let _ = out.push_str("<empty>");
            return out;
        }

        let mut out: String<N> = String::new();
        for ch in text.chars() {
            if out.push(ch).is_err() {
                break;
            }
        }
        out
    }
}

impl Screen for ViewPasswordScreen {
    fn new() -> Self {
        Self::new(0)
    }

    fn draw(&mut self, frame: &mut Frame, _: &mut ListState, kpdb: &KeePassDb) {
        let entry = kpdb
            .entries
            .get(self.entry_index)
            .and_then(|entry| entry.as_ref());

        let (title, password) = match entry {
            Some(entry) => (
                Self::bytes_to_string::<MAX_TITLE_LEN>(&entry.title),
                Self::bytes_to_string::<MAX_PASSWORD_LEN>(&entry.password),
            ),
            None => {
                let mut title: String<MAX_TITLE_LEN> = String::new();
                let _ = title.push_str("Missing entry");
                let mut password: String<MAX_PASSWORD_LEN> = String::new();
                let _ = password.push_str("<missing>");
                (title, password)
            }
        };

        let mut title_padded: String<{ MAX_TITLE_LEN + 2 }> = String::new();
        let _ = title_padded.push(' ');
        let _ = title_padded.push_str(title.as_str());
        let _ = title_padded.push(' ');

        let block = Block::bordered()
            .border_style(Style::new().bold().green())
            .title(title_padded.as_str());
        let inner = block.inner(frame.area());
        frame.render_widget(block, frame.area());

        if inner.is_empty() {
            return;
        }

        let paragraph = Paragraph::new(password.as_str())
            .alignment(Alignment::Center)
            .style(Style::new().bold())
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    fn on_select(&mut self, _: Option<usize>) -> ScreenAction {
        ScreenAction::Pop
    }
}
