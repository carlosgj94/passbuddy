pub mod ssd1309;

use alloc::boxed::Box;
use embedded_graphics::{geometry::Dimensions, pixelcolor::BinaryColor, prelude::DrawTarget};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig, fonts};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};
use ratatui::{Frame, Terminal};

pub fn init_terminal<'a, D>(display: &'a mut D) -> Terminal<EmbeddedBackend<'a, D, BinaryColor>>
where
    D: DrawTarget<Color = BinaryColor> + Dimensions + 'static,
{
    init_terminal_with_flush(display, |_| {})
}

pub fn init_terminal_with_flush<'a, D>(
    display: &'a mut D,
    flush: impl FnMut(&mut D) + 'static,
) -> Terminal<EmbeddedBackend<'a, D, BinaryColor>>
where
    D: DrawTarget<Color = BinaryColor> + Dimensions + 'static,
{
    let backend = EmbeddedBackend::new(
        display,
        EmbeddedBackendConfig {
            flush_callback: Box::new(flush),
            font_regular: fonts::MONO_6X10_OPTIMIZED,
            font_bold: None,
            font_italic: None,
            ..Default::default()
        },
    );

    Terminal::new(backend).expect("terminal init")
}

pub fn initial_state() -> ListState {
    let mut state = ListState::default();
    state.select_first();
    state
}

pub fn draw_menu(frame: &mut Frame, state: &mut ListState) {
    let outer_block = Block::bordered()
        .border_style(Style::new().bold().green())
        .title(" Select Database ");

    let items = ["Personal", "Work", "Shared"];
    let list = List::new(items)
        .block(outer_block)
        .style(Style::new())
        .highlight_style(Style::new().bold().bg(Color::White).fg(Color::Black))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, frame.area(), state);
}
