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
use esp_storage::FlashStorage;
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::Orientation;
use passbuddy::keepass::KeePassDb;
use passbuddy::storage::layout::StorageLayout;
use passbuddy::storage::region::DataRegion;
use static_cell::StaticCell;
use {esp_backtrace as _, esp_println as _};

use passbuddy::display;

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
    info!("Declared peripherals");
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
    info!("Initializing display");
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

    let mut state = display::initial_state();
    let mut terminal = display::init_terminal(&mut display);

    // TODO: Move the drawing inside an embassy task
    info!("Drawing menu");
    terminal
        .draw(|frame| display::draw_menu(frame, &mut state))
        .expect("to draw");

    // 4. Let's initialize the storage
    info!("Initializing storage");
    let mut storage = FlashStorage::new(peripherals.FLASH);
    match StorageLayout::run_healthcheck(&mut storage) {
        Ok(_) => {
            info!("Storage found; good to read");
        }
        Err(_) => {
            // The decive needs to be writen
            info!("Storage not found; initializing");
            StorageLayout::bootstrap_storage_write(&mut storage)
                .expect("initial storage bootstraping error");
        }
    }
    let layout = StorageLayout::new(&mut storage);
    let magic_str = core::str::from_utf8(&layout.header.magic).unwrap_or("<invalid utf8>");
    info!("magic: {=str}", magic_str);

    // 5. Let's initialize the input devices
    //
    // 6. Get the key to decrypt the storage
    //
    // 7. Get the keepass groups
    let offset_to_regions = layout
        .get_offset_to_region(DataRegion::KeePassDb)
        .expect("to get offset");
    let kpdb = KeePassDb::<32, 1024>::new(&mut storage, offset_to_regions);

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        Timer::after(Duration::from_secs(1)).await;
        // do periodic work (or log sparingly)
    }
}
