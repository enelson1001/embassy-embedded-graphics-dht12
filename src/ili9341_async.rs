use log::*;

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::SpiDevice;

use embedded_graphics_core::{
    geometry::Dimensions,
    pixelcolor::raw::ToBytes,
    prelude::{OriginDimensions, PixelColor, Point, Size},
    primitives::Rectangle,
};

use crate::command;
use crate::framebuffer::Framebuffer;

/// Specify state of specific mode of operation
#[derive(Clone, Copy, PartialEq)]
pub enum ModeState {
    On,
    Off,
}

/// Display orientation.
///
/// This enum represents the different possible orientations supported in this display
///
/// # Variants
///
/// - Portrait
/// - Landscape
#[allow(unused)]
#[derive(Clone, Copy, PartialEq)]
pub enum Orientation {
    Potrait,
    Landscape,
}

/// Optional configuration structure to invert the color and screen orientation
pub struct Config {
    inverted_color: ModeState,
    orientation: Orientation,
    height: usize,
    width: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inverted_color: ModeState::On,
            orientation: Orientation::Landscape,
            height: 240,
            width: 320,
        }
    }
}

/// Ili9341 async display driver.
///
/// This struct provides an interface for controlling the Ili9341 display
/// using SPI communication.
///
/// # Type Parameters
///
/// - `SPI`: The SPI device used for communication with the display.
/// - `DC`: The data/command pin, used to switch between sending data and commands.
/// - `RST`: The reset pin, used to reset the display.
/// - `PO`: The power on pin, used to power on the display.
///
/// # Constraints
///
/// - `SPI`: Must implement the `SpiDevice` trait.
/// - `DC`, `RST`, `PO`: Must implement the `OutputPin` trait with `Error = Infallible`.
pub struct Ili9341<SPI, DC, RST, PO>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    PO: OutputPin,
{
    /// SPI device used for communication with the display.
    spi: SPI,
    /// Data/command pin, used to switch between sending data and commands.
    dc: DC,
    /// Reset pin, used to reset the display.
    rst: RST,
    /// Power on pin, used to power on the display.
    power: PO,
    /// Whether the colors are inverted (`true`) or not (`false`).
    inverted: ModeState,
    /// Orientation of the display.
    orientation: Orientation,
    /// Height of display
    pub height: usize,
    /// Width of display
    pub width: usize,
}

impl<SPI, DC, RST, PO> OriginDimensions for Ili9341<SPI, DC, RST, PO>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    PO: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

impl<SPI, DC, RST, PO> Ili9341<SPI, DC, RST, PO>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    PO: OutputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(spi_device: SPI, dc: DC, rst: RST, power: PO, config: Config) -> Self {
        Self {
            spi: spi_device,
            dc,
            rst,
            power,
            inverted: config.inverted_color,
            orientation: config.orientation,
            height: config.height,
            width: config.width,
        }
    }

    /// Runs commands to initialize the display in the default configuration for this library. In most use cases, this should
    /// be all that is needed to start and set-up the device.
    ///
    /// # Parameters
    ///
    /// - `delay`: A mutable reference to an implementation of the `DelayNs` trait.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the initialization succeeds, or an `Error` if it fails.
    pub async fn initialize<D>(&mut self, delay: &mut D) -> Result<(), Error>
    where
        D: DelayNs,
    {
        self.hardware_reset(delay).await?;
        self.software_reset(delay).await?;
        self.set_orientation().await?;
        self.set_pixel_format().await?;
        self.set_invert_mode().await?;
        self.set_sleep_mode(ModeState::Off, delay).await?;
        self.set_display_mode(ModeState::On, delay).await?;

        Ok(())
    }

    /// Hardware Reset
    ///
    /// # Errors
    ///
    /// Returns an error if setting any pin fails.
    async fn hardware_reset<D>(&mut self, delay: &mut D) -> Result<(), Error>
    where
        D: DelayNs,
    {
        //async fn hardware_reset(&mut self) -> Result {
        debug!("Hardware reset");
        debug!("Set RST low");
        // Do hardware reset by holding reset low for at least 10us

        self.rst.set_low().map_err(Error::from_digital)?;
        delay.delay_ms(1).await;

        debug!("Set RST high");
        // Set high for normal operation
        // Wait 5ms after reset before sending commands
        // and 120ms before sending Sleep Out
        self.rst.set_high().map_err(Error::from_digital)?;
        delay.delay_ms(5).await;

        debug!("Hardware reset / done");

        Ok(())
    }

    /// Software Reset
    ///
    /// # Errors
    ///
    /// Returns an error if any commands to the display fails
    async fn software_reset<D>(&mut self, delay: &mut D) -> Result<(), Error>
    where
        D: DelayNs,
    {
        debug!("Software reset");
        self.send_command(command::SOFTWARE_RESET, &[]).await?;
        delay.delay_ms(120).await;
        debug!("Software reset / done");

        Ok(())
    }

    /// Set display orientation
    ///
    /// # Errors
    ///
    /// Returns an error if any commands to the display fails
    async fn set_orientation(&mut self) -> Result<(), Error> {
        debug!("Set Orientation");

        match self.orientation {
            Orientation::Potrait => {
                self.send_command(command::MEMORY_ACCESS_CONTROL, &[0x68])
                    .await?
            } //0x86
            Orientation::Landscape => {
                self.send_command(command::MEMORY_ACCESS_CONTROL, &[0x08])
                    .await?
            } //0x08
        }

        debug!("Display orientation / done");

        Ok(())
    }

    /// Set pixel format
    /// 0x55 = 16 bits per pixels, 0x66 = 18 bits per pixel
    ///
    /// # Errors
    ///
    /// Returns an error if any commands to the display fails
    async fn set_pixel_format(&mut self) -> Result<(), Error> {
        debug!("Set Pixel Format");
        self.send_command(command::PIXEL_FORMAT_SET, &[0x55])
            .await?;

        debug!("Display pixel format / done");

        Ok(())
    }

    /// Set sleep mode
    ///
    /// # Errors
    ///
    /// Returns an error if any commands to the display fails
    async fn set_sleep_mode<D>(&mut self, mode: ModeState, delay: &mut D) -> Result<(), Error>
    where
        D: DelayNs,
    {
        match mode {
            ModeState::Off => {
                debug!("Set Sleep Off");
                self.send_command(command::SLEEP_MODE_OFF, &[]).await?;
                delay.delay_ms(150).await;
            }

            ModeState::On => {
                debug!("Set Sleep On");
                self.send_command(command::SLEEP_MODE_ON, &[]).await?;
                delay.delay_ms(50).await;
            }
        }

        debug!("Set Sleep Mode / done");

        Ok(())
    }

    /// Set display mode
    ///
    /// # Errors
    ///
    /// Returns an error if any commands to the display fails
    async fn set_display_mode<D>(&mut self, mode: ModeState, delay: &mut D) -> Result<(), Error>
    where
        D: DelayNs,
    {
        match mode {
            ModeState::Off => {
                debug!("Set Display Off");
                self.send_command(command::DISPLAY_OFF, &[]).await?;
                delay.delay_ms(100).await;
            }

            ModeState::On => {
                debug!("Set Display On");
                self.send_command(command::DISPLAY_ON, &[]).await?;
                delay.delay_ms(100).await;
            }
        }

        debug!("Set Display Mode / done");

        Ok(())
    }

    /// Invert pixel color on screen
    ///
    /// # Errors
    ///
    /// Returns an error if any commands to the display fails
    async fn set_invert_mode(&mut self) -> Result<(), Error> {
        match self.inverted {
            ModeState::Off => {
                debug!("Invert Off");
                self.send_command(command::INVERT_OFF, &[]).await?;
            }

            ModeState::On => {
                debug!("Invert On");
                self.send_command(command::INVERT_ON, &[]).await?;
            }
        }

        debug!("Set Display Mode / done");

        Ok(())
    }

    /// Turn on backlght
    ///
    /// # Errors
    ///
    /// Returns an error if setting any pin fails.
    pub fn turn_on_backlight(&mut self) -> Result<(), Error> {
        //async fn hardware_reset(&mut self) -> Result {
        debug!("Turn on backlight");
        self.power.set_high().map_err(Error::from_digital)?;
        debug!("Turn on backlight / done");

        Ok(())
    }

    /// Send command over SPI bus
    ///
    /// # Errors
    ///
    /// Returns an error if writing to SPI bus fails.
    async fn send_command(&mut self, command: u8, data: &[u8]) -> Result<(), Error> {
        //trace!("Set DC to low for transferring commands");
        self.dc.set_low().map_err(Error::from_digital)?;
        self.spi.write(&[command]).await?;

        if !data.is_empty() {
            self.dc.set_high().map_err(Error::from_digital)?;
            self.spi.write(data).await?;
        }

        Ok(())
    }

    /// Sends the data to the given area of the display's frame buffer.
    ///
    /// The `area` is in your logical display coordinates; e.g if you use
    /// if landscape the logical size is (320, 240) and the (0, 0) is the
    /// top-right corner of the un-rotated physical screen.
    ///
    /// You can fill the area using a smaller buffer by repeatedly calling
    /// this method and passing the same `area`. Sending more data than fits
    /// in the area will wrap around and overwrite the beginning of the area.
    ///
    /// # Panics
    ///
    /// If the area is empty or not completely contained within the display
    /// bounds.
    pub async fn write_pixels(&mut self, data: &[u8], area: Rectangle) -> Result<(), Error> {
        //info!("area bottom right = {:?}", area.bottom_right().unwrap());
        assert!(self.bounding_box().contains(area.top_left));
        assert!(self.bounding_box().contains(area.bottom_right().unwrap()));

        let area_x0: u16 = area.top_left.x.try_into().unwrap();
        let area_y0: u16 = area.top_left.y.try_into().unwrap();
        let area_x1: u16 = area.bottom_right().unwrap().x.try_into().unwrap();
        let area_y1: u16 = area.bottom_right().unwrap().y.try_into().unwrap();

        self.set_window(area_x0, area_y0, area_x1, area_y1).await?;
        self.send_command(command::MEMORY_WRITE, &[]).await?;
        self.dc.set_high().map_err(Error::from_digital)?;

        self.spi.write(data).await?;

        Ok(())
    }

    /// Set the window area where pixel data will be drawn on screen, represented by top-left corner (x0, y0)
    /// and bottom-right corner (x1, y1).
    async fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), Error> {
        //info!("x0 = {:?}  y0 = {:?}  x1 = {:?}  y1 = {:?}", x0, y0, x1, y1);

        self.send_command(
            command::COLUMN_ADDRESS_SET,
            &[
                (x0 >> 8) as u8,
                (x0 & 0xff) as u8,
                (x1 >> 8) as u8,
                (x1 & 0xff) as u8,
            ],
        )
        .await?;
        self.send_command(
            command::PAGE_ADDRESS_SET,
            &[
                (y0 >> 8) as u8,
                (y0 & 0xff) as u8,
                (y1 >> 8) as u8,
                (y1 & 0xff) as u8,
            ],
        )
        .await?;

        Ok(())
    }
}

/******************************************************************************************************
*                                  IMPLEMENT CUSTOM ERRORS
*****************************************************************************************************/

use embedded_hal::digital::Error as DigitalError;
use embedded_hal::digital::ErrorKind as DigitalErrorKind;
use embedded_hal::spi::Error as SpiError;
use embedded_hal::spi::ErrorKind as SpiErrorKind;

/// Error Types used within this driver
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An error in the underlying SPI bus
    Spi(SpiErrorKind),

    /// An error in the underlying digital system
    Digital(DigitalErrorKind),
}

impl<E> From<E> for Error
where
    E: SpiError,
{
    fn from(error: E) -> Self {
        Self::Spi(error.kind())
    }
}

impl Error {
    /// Convert a digital error to an error
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_digital<E>(error: E) -> Self
    where
        E: DigitalError,
    {
        Self::Digital(error.kind())
    }
}

/******************************************************************************************************
*                                  IMPLEMENT CUSTOM TRAIT
*****************************************************************************************************/
/// Convenience trait to hide details of the driver type.
///
/// Once the display driver is created, only the error type depends on the HAL
/// types used for the implementation. For the use cases where panic on error
/// is acceptable, we can ignore the type parameters.
#[allow(async_fn_in_trait)]
pub trait WritePixels {
    async fn write_pixels(&mut self, data: &[u8], area: Rectangle);

    /// Transfers the contents of the framebuffer to the display.
    async fn flush<C>(&mut self, fb: &Framebuffer<'_, C>, top_left: Point)
    where
        C: PixelColor + ToBytes,
    {
        self.write_pixels(fb.data(), Rectangle::new(top_left, fb.size()))
            .await
    }
}

impl<SPI, DC, RST, PO> WritePixels for Ili9341<SPI, DC, RST, PO>
where
    SPI: SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    PO: OutputPin,
{
    async fn write_pixels(&mut self, data: &[u8], area: Rectangle) {
        self.write_pixels(data, area)
            .await
            .unwrap_or_else(|_| panic!("write_pixels failed"))
    }
}
