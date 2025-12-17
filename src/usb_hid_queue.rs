use defmt::Format;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use heapless::String;

pub const USB_HID_TEXT_CAP: usize = 64;
pub const USB_HID_QUEUE_DEPTH: usize = 4;

#[derive(Clone, Debug, Format)]
pub enum UsbHidCommand {
    TypeText(String<USB_HID_TEXT_CAP>),
}

#[derive(Clone, Copy, Debug, Format, Eq, PartialEq)]
pub enum UsbHidQueueError {
    Full,
    TooLong,
}

static CHANNEL: Channel<CriticalSectionRawMutex, UsbHidCommand, USB_HID_QUEUE_DEPTH> =
    Channel::new();

pub fn try_queue(command: UsbHidCommand) -> Result<(), UsbHidQueueError> {
    CHANNEL
        .try_send(command)
        .map_err(|_| UsbHidQueueError::Full)
}

pub fn try_queue_type_text(text: &str) -> Result<(), UsbHidQueueError> {
    let mut buf: String<USB_HID_TEXT_CAP> = String::new();
    if buf.push_str(text).is_err() {
        return Err(UsbHidQueueError::TooLong);
    }

    try_queue(UsbHidCommand::TypeText(buf))
}

pub async fn receive() -> UsbHidCommand {
    CHANNEL.receive().await
}
