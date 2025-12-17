#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;

use alloc::boxed::Box;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_usb::class::hid::{HidReaderWriter, State};
use embassy_usb::driver::Driver;
use embassy_usb::{Builder, UsbDevice};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::otg_fs::Usb;
use esp_hal::otg_fs::asynch::{Config, Driver as OtgDriver};
use esp_hal::spi::{Mode as SpiMode, master::Config as SpiConfig, master::Spi};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, hmac::Hmac};
use esp_storage::FlashStorage;
use passbuddy::app::AppState;
use passbuddy::keepass::KeePassDb;
use passbuddy::storage::layout::StorageLayout;
use passbuddy::storage::region::DataRegion;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use {esp_backtrace as _, esp_println as _};

use passbuddy::input::Inputs;
use passbuddy::{app, display};

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
            .with_frequency(Rate::from_mhz(10))
            .with_mode(SpiMode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7);
    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);

    // 2. Let's initialize the display
    info!("Initializing display");
    let cs = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    let reset = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());

    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let interface = display::ssd1309::SpiInterface::new(spi_dev, dc);
    let mut display = display::ssd1309::Ssd1309::new(interface, Some(reset));

    display
        .init(&mut Delay::new())
        .unwrap_or_else(|_| panic!("SSD1309 init failed"));

    let app_state = AppState::new();
    let mut terminal = app::init_terminal_with_flush(&mut display, |display| {
        display
            .flush()
            .unwrap_or_else(|_| panic!("SSD1309 flush failed"));
    });

    // 4. Let's initialize the storage
    info!("Initializing storage");
    let mut storage = FlashStorage::new(peripherals.FLASH);

    // Debug helper: wipe the storage layout sector. If enabled, keep it *before*
    // `run_healthcheck()` so the layout gets bootstrapped again.
    StorageLayout::wipe_layout(&mut storage).unwrap();

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
    info!("Setting the inputs");
    let mut inputs = Inputs::new(
        peripherals.PCNT,
        peripherals.GPIO15,
        peripherals.GPIO17,
        peripherals.GPIO16,
    );
    //
    // 6. Get the key to decrypt the storage
    //
    // 7. Get the keepass groups
    let keepass_region = layout.region_handle(DataRegion::KeePassDb).unwrap();
    if !KeePassDb::check_if_exists(&mut storage, keepass_region).unwrap() {
        info!("Creating a new keepass");
        KeePassDb::initialize_db(&mut storage, keepass_region).unwrap();
    }

    info!("Indexing the keepass database");
    let kpdb = KeePassDb::new(&mut storage, keepass_region).unwrap();
    let mut app_state = app_state.with_kpdb(kpdb);

    // 8. Initialize the USB device
    // Creating the driver from the hal
    let ep_out_buffer = Box::leak(Box::new([0u8; 124]));
    let config = Config::default();
    let otg_driver = OtgDriver::new(usb, ep_out_buffer, config);

    let mut usb_config = embassy_usb::Config::new(0xa0de, 0xdafe);
    usb_config.manufacturer = Some("Passbuddy");
    usb_config.product = Some("Passbuddy USB HID");
    usb_config.serial_number = Some("1234567890");
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;

    let config_descriptor_buffer = Box::leak(Box::new([0; 256]));
    let bos_descriptor_buffer = Box::leak(Box::new([0; 256]));
    let msos_descriptor_buffer = Box::leak(Box::new([0; 256]));
    let control_buffer = Box::leak(Box::new([0; 64]));

    let usb_state = Box::leak(Box::new(State::new()));

    let mut usb_builder = Builder::new(
        otg_driver,
        usb_config,
        config_descriptor_buffer,
        bos_descriptor_buffer,
        msos_descriptor_buffer,
        control_buffer,
    );

    let usb_config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 60,
        max_packet_size: 8,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut usb_builder, usb_state, usb_config);
    let usb = usb_builder.build();

    // 9. Spawn the tasks
    spawner.must_spawn(run_usb(usb));

    info!("Starting the loop");
    loop {
        Timer::after(Duration::from_millis(50)).await;
        let before = app_state.selected();
        let input_event = inputs.poll();
        app_state.apply_navigation(input_event.delta);

        let action_pressed = input_event.pressed;

        if action_pressed {
            info!("Action button pressed");
            app_state.on_select(&mut storage);
        }

        if app_state.selected() != before || action_pressed {
            terminal
                .draw(|frame| app_state.draw_current_screen(frame))
                .expect("to draw");
        }

        // do periodic work (or log sparingly)
    }
}

#[embassy_executor::task]
async fn run_usb(mut usb: UsbDevice<'static, OtgDriver<'static>>) -> ! {
    usb.run().await
}
