use alloc::boxed::Box;

use embedded_graphics::{geometry::Dimensions, pixelcolor::BinaryColor, prelude::DrawTarget};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig, fonts};
use ratatui::Terminal;

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
