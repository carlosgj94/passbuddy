use alloc::boxed::Box;
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_usb::class::hid::{HidReaderWriter, HidWriter, State};
use embassy_usb::{Builder, UsbDevice};
use esp_hal::otg_fs::Usb;
use esp_hal::otg_fs::asynch::{Config, Driver as OtgDriver};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use passbuddy::usb_hid_queue;
use passbuddy::usb_hid_queue::UsbHidCommand;

pub fn spawn(spawner: &Spawner, usb: Usb<'static>) {
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

    let hid_config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 60,
        max_packet_size: 8,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut usb_builder, usb_state, hid_config);
    let usb = usb_builder.build();
    let (_, writer) = hid.split();

    spawner.must_spawn(run_usb(usb));
    spawner.must_spawn(usb_writer(writer));
}

#[embassy_executor::task]
async fn run_usb(mut usb: UsbDevice<'static, OtgDriver<'static>>) -> ! {
    usb.run().await
}

#[embassy_executor::task]
async fn usb_writer(mut writer: HidWriter<'static, OtgDriver<'static>, 8>) {
    writer.ready().await;
    info!("USB HID writer ready");

    let release = KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0, 0, 0, 0, 0, 0],
    };
    let _ = writer.write_serialize(&release).await;

    loop {
        match usb_hid_queue::receive().await {
            UsbHidCommand::TypeText(text) => {
                writer.ready().await;
                type_text(&mut writer, text.as_str()).await;
            }
        }
    }
}

const MOD_LSHIFT: u8 = 0x02;

fn hid_key_for_char(ch: char) -> Option<(u8, u8)> {
    let byte = u8::try_from(ch).ok()?;
    match ch {
        'a'..='z' => Some((0, 0x04 + (byte - b'a'))),
        'A'..='Z' => Some((MOD_LSHIFT, 0x04 + (byte - b'A'))),
        '1'..='9' => Some((0, 0x1e + (byte - b'1'))),
        '0' => Some((0, 0x27)),
        '-' => Some((0, 0x2d)),
        '_' => Some((MOD_LSHIFT, 0x2d)),
        ' ' => Some((0, 0x2c)),
        _ => None,
    }
}

async fn type_text(writer: &mut HidWriter<'static, OtgDriver<'static>, 8>, text: &str) {
    for ch in text.chars() {
        let Some((modifier, keycode)) = hid_key_for_char(ch) else {
            warn!("USB HID: unsupported character");
            continue;
        };

        let press = KeyboardReport {
            modifier,
            reserved: 0,
            leds: 0,
            keycodes: [keycode, 0, 0, 0, 0, 0],
        };
        if let Err(e) = writer.write_serialize(&press).await {
            warn!("USB HID press failed: {:?}", e);
            return;
        }

        let release = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [0, 0, 0, 0, 0, 0],
        };
        if let Err(e) = writer.write_serialize(&release).await {
            warn!("USB HID release failed: {:?}", e);
            return;
        }

        Timer::after(Duration::from_millis(2)).await;
    }
}
