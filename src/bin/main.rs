#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::{Mode as SpiMode, master::Config as SpiConfig, master::Spi};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, hmac::Hmac};
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::Orientation;
// use mousefood::prelude::Rgb565;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig, fonts};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List, ListState};
use ratatui::{Frame, Terminal};
use static_cell::StaticCell;
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

static SPI_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    // 1. Get the peripherals declared
    let mut _hmac = Hmac::new(peripherals.HMAC);
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(SpiMode::_3),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7);

    // 2. Let's initialize the display
    let cs = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let reset = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());

    let spi_buffer = SPI_BUFFER.init([0; 512]);
    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let interface = SpiInterface::new(spi_dev, dc, spi_buffer);

    let mut display = Builder::new(ST7789, interface)
        .display_size(240, 240)
        .reset_pin(reset)
        .init(&mut Delay::new())
        .unwrap();

    display
        .set_orientation(Orientation::default().rotate(mipidsi::options::Rotation::Deg90))
        .unwrap();

    // display.clear(Rgb565::BLACK).unwrap();

    let mut state = ListState::default();
    state.select_first();

    let backend = EmbeddedBackend::new(
        &mut display,
        EmbeddedBackendConfig {
            font_regular: fonts::MONO_9X18,
            font_bold: Some(fonts::MONO_9X18_BOLD),
            font_italic: None,
            ..Default::default()
        },
    );

    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| draw(frame, &mut state))
        .expect("to draw");

    // 3. Let's initialize the input devices

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}

fn draw(frame: &mut Frame, state: &mut ListState) {
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
