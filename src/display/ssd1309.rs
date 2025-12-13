use core::convert::Infallible;

use embedded_graphics::{
    geometry::Size,
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, OriginDimensions, Pixel},
};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

const WIDTH: usize = 128;
const HEIGHT: usize = 64;
const PAGES: usize = HEIGHT / 8;
const BUFFER_SIZE: usize = WIDTH * PAGES;

pub trait DisplayInterface {
    type Error;

    fn send_commands(&mut self, cmds: &[u8]) -> Result<(), Self::Error>;
    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}

pub enum InterfaceError<SpiError, PinError> {
    Spi(SpiError),
    Pin(PinError),
}

pub struct SpiInterface<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI, DC> SpiInterface<SPI, DC> {
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }

    pub fn release(self) -> (SPI, DC) {
        (self.spi, self.dc)
    }
}

impl<SPI, DC> DisplayInterface for SpiInterface<SPI, DC>
where
    SPI: SpiDevice<u8>,
    DC: OutputPin,
{
    type Error = InterfaceError<SPI::Error, DC::Error>;

    fn send_commands(&mut self, cmds: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(InterfaceError::Pin)?;
        self.spi.write(cmds).map_err(InterfaceError::Spi)
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_high().map_err(InterfaceError::Pin)?;
        self.spi.write(data).map_err(InterfaceError::Spi)
    }
}

#[derive(Clone, Copy)]
pub enum Rotation {
    Rotate0,
    Rotate180,
}

#[derive(Clone, Copy)]
pub enum VccMode {
    /// Uses `0xAD 0x8A` in the init sequence.
    External,
    /// Uses `0xAD 0x8B` in the init sequence.
    Internal,
}

pub struct Config {
    pub rotation: Rotation,
    pub vcc_mode: VccMode,
    pub column_offset: u8,
    pub contrast: u8,
    pub inverted: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rotation: Rotation::Rotate0,
            vcc_mode: VccMode::Internal,
            column_offset: 0,
            contrast: 0x7F,
            inverted: false,
        }
    }
}

pub enum Error<InterfaceError, ResetError> {
    Interface(InterfaceError),
    Reset(ResetError),
}

pub struct Ssd1309<IF, RST> {
    iface: IF,
    reset: Option<RST>,
    buffer: [u8; BUFFER_SIZE],
    config: Config,
}

impl<IF, RST> Ssd1309<IF, RST> {
    pub fn new(iface: IF, reset: Option<RST>) -> Self {
        Self {
            iface,
            reset,
            buffer: [0; BUFFER_SIZE],
            config: Config::default(),
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn release(self) -> (IF, Option<RST>) {
        (self.iface, self.reset)
    }

    pub fn flush(&mut self) -> Result<(), Error<IF::Error, RST::Error>>
    where
        IF: DisplayInterface,
        RST: OutputPin,
    {
        let col = self.config.column_offset;
        let buffer = &self.buffer;
        let iface = &mut self.iface;

        for page in 0..PAGES {
            let page_cmd = 0xB0u8 | page as u8;
            let col_low = col & 0x0F;
            let col_high = 0x10 | (col >> 4);

            iface
                .send_commands(&[page_cmd, col_low, col_high])
                .map_err(Error::Interface)?;

            let start = page * WIDTH;
            let end = start + WIDTH;
            iface
                .send_data(&buffer[start..end])
                .map_err(Error::Interface)?;
        }

        Ok(())
    }

    pub fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), Error<IF::Error, RST::Error>>
    where
        IF: DisplayInterface,
        RST: OutputPin,
    {
        if let Some(reset) = self.reset.as_mut() {
            reset.set_low().map_err(Error::Reset)?;
            delay.delay_ms(10);
            reset.set_high().map_err(Error::Reset)?;
            delay.delay_ms(10);
        }

        let (seg_remap, com_scan) = match self.config.rotation {
            Rotation::Rotate0 => (0xA1, 0xC8),
            Rotation::Rotate180 => (0xA0, 0xC0),
        };

        let dc_dc = match self.config.vcc_mode {
            VccMode::External => 0x8A,
            VccMode::Internal => 0x8B,
        };

        let invert = if self.config.inverted { 0xA7 } else { 0xA6 };

        self.send_commands(&[
            0xAE, // Display off
            0xD5,
            0x80, // Clock divide ratio
            0xA8,
            0x3F, // Multiplex ratio (1/64)
            0xD3,
            0x00, // Display offset
            0x40, // Display start line
            0xAD,
            dc_dc,     // DC-DC control
            seg_remap, // Segment remap
            com_scan,  // COM scan direction
            0xDA,
            0x12, // COM pins config
            0x81,
            self.config.contrast, // Contrast
            0xD9,
            0x22, // Pre-charge period
            0xDB,
            0x20,   // VCOMH deselect level
            0xA4,   // Resume to RAM content display
            invert, // Normal/invert display
            0x20,
            0x02, // Page addressing mode
            0x2E, // Deactivate scroll
            0xAF, // Display on
        ])?;

        self.clear_buffer(BinaryColor::Off);
        self.flush()?;

        Ok(())
    }

    pub fn clear_buffer(&mut self, color: BinaryColor) {
        self.buffer.fill(if color.is_on() { 0xFF } else { 0x00 });
    }

    fn send_commands(&mut self, cmds: &[u8]) -> Result<(), Error<IF::Error, RST::Error>>
    where
        IF: DisplayInterface,
        RST: OutputPin,
    {
        self.iface.send_commands(cmds).map_err(Error::Interface)
    }
}

impl<IF, RST> OriginDimensions for Ssd1309<IF, RST> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

impl<IF, RST> DrawTarget for Ssd1309<IF, RST> {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            if point.x < 0 || point.y < 0 {
                continue;
            }

            let x = point.x as usize;
            let y = point.y as usize;

            if x >= WIDTH || y >= HEIGHT {
                continue;
            }

            let page = y / 8;
            let bit = y % 8;
            let index = page * WIDTH + x;
            let mask = 1u8 << bit;

            if color.is_on() {
                self.buffer[index] |= mask;
            } else {
                self.buffer[index] &= !mask;
            }
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.clear_buffer(color);
        Ok(())
    }
}
