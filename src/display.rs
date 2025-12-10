use embedded_graphics::{geometry::Dimensions, pixelcolor::Rgb565, prelude::DrawTarget};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig, fonts};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};
use ratatui::{Frame, Terminal};

pub fn init_terminal<'a, D>(display: &'a mut D) -> Terminal<EmbeddedBackend<'a, D, Rgb565>>
where
    D: DrawTarget<Color = Rgb565> + Dimensions + 'static,
{
    let backend = EmbeddedBackend::new(
        display,
        EmbeddedBackendConfig {
            font_regular: fonts::MONO_9X18,
            font_bold: Some(fonts::MONO_9X18_BOLD),
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
        .highlight_style(Style::new().bold().bg(Color::Green).italic())
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, frame.area(), state);
}
