/*
This works is based upon works by other namely the following.
    1. https://github.com/yuri91/ili9341-rs/tree/master
    2. https://github.com/inazarenko/ssd1331-async/tree/b3006ad955dfc83100527a56d7c06d775ac3bb69

These two projects provided me inspiration to create this demo project.

Below are the timings I saw when I ran the program on a M5Stack Core Basic

==============================================================================================
DMA_TX = 4096, SPI-Clk = 10Mhz, Framebuffer size = screen width x 14/screen height x 2 bytes
==============================================================================================
Display initialization = 377 milliseconds
Clear display = 127 milliseconds
Display Title: 12941 microseconds
rounded rectangle: 13729 microseconds
rounded rectangle: 13533 microseconds

The flash size used:
App/part. size:    125,168/4,128,768 bytes, 3.03%

*/

#![no_std]
#![no_main]

pub mod command;
pub mod framebuffer;
pub mod ili9341_async;

use core::fmt::Write;
use heapless::String;
use log::*;

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_8X13_BOLD, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use profont::PROFONT_24_POINT;

use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority, DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Io, Level, Output},
    i2c::I2c,
    peripherals::{I2C0, SPI2},
    prelude::*,
    spi::{
        master::{Spi, SpiDmaBus},
        FullDuplexMode, SpiMode,
    },
    timer::timg::TimerGroup,
    Async,
};

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Delay, Duration, Instant, Timer};

use static_cell::ConstStaticCell;
use static_cell::StaticCell;

use embedded_hal_bus::spi::ExclusiveDevice;

use crate::framebuffer::Framebuffer;
use crate::ili9341_async::{Config, Ili9341, WritePixels};

pub struct Dht12Reading {
    pub humidity: f32,
    pub temp_fahrenheit: f32,
}

/// Period to wait between DHT12 readings
const SAMPLING_PERIOD: Duration = Duration::from_secs(2);

/// A channel between read_dht12_task and render task
static CHANNEL: StaticCell<Channel<NoopRawMutex, Dht12Reading, 2>> = StaticCell::new();

/// Frame Buffer Size = display width x 1/4 Display height x number of bytes in pexel color
const FRAME_BUFFER_SIZE: usize = 320 * 60 * 2;

/// Create static pixel data buffer can be used by both sync and async
static PIXEL_DATA: ConstStaticCell<[u8; FRAME_BUFFER_SIZE]> =
    ConstStaticCell::new([0; FRAME_BUFFER_SIZE]);

#[embassy_executor::task]
async fn render_task(
    mut display: Ili9341<
        ExclusiveDevice<SpiDmaBus<'static, SPI2, FullDuplexMode, Async>, Output<'static>, Delay>,
        Output<'static>,
        Output<'static>,
        Output<'static>,
    >,
    receiver: Receiver<'static, NoopRawMutex, Dht12Reading, 2>,
) {
    let pixel_data = PIXEL_DATA.take();
    let mut fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(320, 60));

    // Clear the screen - break up screen into 4 rectangle shapes the size
    // 320 x 60 which is the same size as frame buffer
    Rectangle::new(Point::new(0, 0), Size::new(320, 60))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(&mut fb)
        .unwrap();

    let mut start = Instant::now();
    for i in 0..4 {
        display.flush(&fb, Point::new(0, i * 60)).await;
    }
    info!(
        "clear display: {} milliseconds",
        Instant::now().duration_since(start).as_millis()
    );

    display.turn_on_backlight().unwrap();

    // Create character styles
    let char_10x20_blue_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::BLUE)
        // Need this so previous text is erased
        .background_color(Rgb565::BLACK)
        .build();

    // Create a new title style
    let text_style = TextStyleBuilder::new()
        .baseline(Baseline::Bottom)
        .alignment(Alignment::Left)
        .build();

    // Create screen title
    start = Instant::now();
    fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(170, 20));
    Text::with_text_style(
        "DHT12 SENSOR DATA",
        Point::new(0, 19),
        char_10x20_blue_style,
        text_style,
    )
    .draw(&mut fb)
    .unwrap();
    display.flush(&fb, Point::new(75, 10)).await;

    warn!(
        "Display Title: {} microseconds",
        Instant::now().duration_since(start).as_micros()
    );

    // Temperature round rectangle style
    let style_rect_temperature = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::GREEN)
        .build();

    start = Instant::now();
    fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(200, 30));
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(0, 0), Size::new(200, 30)),
        Size::new(10, 10),
    )
    .into_styled(style_rect_temperature)
    .draw(&mut fb)
    .unwrap();

    display.flush(&fb, Point::new(60, 40)).await;
    warn!(
        "rounded rectangle: {} microseconds",
        Instant::now().duration_since(start).as_micros()
    );

    // Humidity round rectangle style
    let style_rect_humidity = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::YELLOW)
        .build();

    start = Instant::now();
    fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(200, 30));
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(0, 0), Size::new(200, 30)),
        Size::new(10, 10),
    )
    .into_styled(style_rect_humidity)
    .draw(&mut fb)
    .unwrap();

    display.flush(&fb, Point::new(60, 90)).await;
    warn!(
        "rounded rectangle: {} microseconds",
        Instant::now().duration_since(start).as_micros()
    );

    let temp_char_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13_BOLD)
        .text_color(Rgb565::BLACK)
        // Need this so previous text is erased
        .background_color(Rgb565::GREEN)
        .build();

    // Create temperature title
    // Using font 8x13: Temperature is 11 characters 11x8=88 wide by 13 high so framebuffer will be 88, 13
    fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(88, 13));
    Text::with_text_style(
        "Temperature",
        Point::new(0, 12),
        temp_char_style,
        text_style,
    )
    .draw(&mut fb)
    .unwrap();
    display.flush(&fb, Point::new(76, 50)).await;

    let humidity_char_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13_BOLD)
        .text_color(Rgb565::BLACK)
        // Need this so previous text is erased
        .background_color(Rgb565::YELLOW)
        .build();

    // Create Humidity title
    // Using font 8x13: Humidity is 8 characters 8x8=64 wide by 13 high so framebuffer will be 88, 13
    fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(64, 13));
    Text::with_text_style(
        "Humidity",
        Point::new(0, 12),
        humidity_char_style,
        text_style,
    )
    .draw(&mut fb)
    .unwrap();
    display.flush(&fb, Point::new(76, 100)).await;

    let time_style = MonoTextStyleBuilder::new()
        .font(&PROFONT_24_POINT)
        .text_color(Rgb565::RED)
        // Need this so previous text is erased
        .background_color(Rgb565::BLACK)
        .build();

    // Create time value
    fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(128, 25));
    Text::with_text_style("12:00 pm", Point::new(0, 24), time_style, text_style)
        .draw(&mut fb)
        .unwrap();
    display.flush(&fb, Point::new(96, 160)).await;

    let mut old_humidity_value: i8 = -30;
    let mut old_temperature_value: i8 = -100;
    let value_font_width = 8;

    loop {
        let dht12_reading = receiver.receive().await;
        let humidity: i8 = dht12_reading.humidity as i8;
        let temperature: i8 = dht12_reading.temp_fahrenheit as i8;

        info!(
            "HUMIDITY = {:?}   TEMPERATURE F = {:?}",
            humidity, temperature
        );

        // Update display if temperature value changed
        if temperature != old_temperature_value {
            old_temperature_value = temperature;

            let mut temperature_value_str = String::<8>::new();
            let _ = write!(temperature_value_str, "{temperature}F");

            // Count the number of characters in temperaturte value string to determine framebuffer width
            let temp_pixel_width =
                (temperature_value_str.chars().count() * value_font_width) as u32;

            // Create temperature value
            fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(temp_pixel_width, 13));
            Text::with_text_style(
                &temperature_value_str,
                Point::new(0, 12),
                temp_char_style,
                text_style,
            )
            .draw(&mut fb)
            .unwrap();
            display.flush(&fb, Point::new(220, 50)).await;
        }

        // Update display if humidity value changed
        if humidity != old_humidity_value {
            old_humidity_value = humidity;

            let mut humidity_value_str = String::<8>::new();
            let _ = write!(humidity_value_str, "{humidity}%");

            // Count the number of characters in humidity value string to determine framebuffer width
            let humidity_pixel_width =
                (humidity_value_str.chars().count() * value_font_width) as u32;

            // Create humidity value
            fb = Framebuffer::<Rgb565>::new(pixel_data, Size::new(humidity_pixel_width, 13));
            Text::with_text_style(
                &humidity_value_str,
                Point::new(0, 12),
                humidity_char_style,
                text_style,
            )
            .draw(&mut fb)
            .unwrap();
            display.flush(&fb, Point::new(220, 100)).await;
        }
    }
}

#[embassy_executor::task]
async fn read_dht12_task(
    mut i2c: I2c<'static, I2C0, Async>,
    sender: Sender<'static, NoopRawMutex, Dht12Reading, 2>,
) {
    loop {
        info!("DHT12 Read Loop");
        let mut data = [0u8; 5];
        i2c.write_read(0x5c, &[0x00], &mut data).await.unwrap();

        /*
        esp_println::println!(
            "DHT12  B0:{:02x?}  B1:{:02x?}  B2:{:02x?}  B3:{:02x?}  B4:{:02x?}",
            data[0],
            data[1],
            data[2],
            data[3],
            data[4]
        );
        */

        let humidity: f32 = data[0] as f32 + (data[1] as f32) * 0.1;
        let mut temp_celsius: f32 = (data[2] & 0x7F) as f32 + (data[3] as f32) * 0.1;

        if (data[3] & 0x80) != 0 {
            temp_celsius = temp_celsius * -1.0;
        }
        let temp_fahrenheit: f32 = ((temp_celsius * 9.0) / 5.0) + 32.0;

        sender
            .send(Dht12Reading {
                humidity,
                temp_fahrenheit,
            })
            .await;

        Timer::after(SAMPLING_PERIOD).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    esp_println::println!("Init!");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    info!("Creating I2C bus");
    let i2c0 = I2c::new_async(
        peripherals.I2C0,
        io.pins.gpio21, // SDA
        io.pins.gpio22, // SCL
        100.kHz(),
    );

    info!("Create additional PINs");
    let rst = Output::new(io.pins.gpio33, Level::Low);
    let dc = Output::new(io.pins.gpio27, Level::Low);
    let bcklt = Output::new(io.pins.gpio32, Level::Low);

    info!("Create SPI bus");
    let spi_bus = Spi::new(peripherals.SPI2, 10_000_u32.kHz(), SpiMode::Mode0)
        .with_sck(io.pins.gpio18)
        .with_mosi(io.pins.gpio23);

    info!("Wrap SPI bus in a SPI DMA");
    let dma = Dma::new(peripherals.DMA);
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(8, 4096); // was 32768,  8*2, 4092
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();
    let dma_channel = dma.spi2channel;

    let spi_dma: SpiDmaBus<'_, SPI2, FullDuplexMode, Async> = spi_bus
        .with_dma(dma_channel.configure_for_async(false, DmaPriority::Priority0))
        .with_buffers(dma_rx_buf, dma_tx_buf);

    let cs = Output::new(io.pins.gpio14, Level::Low);
    let spi_device = ExclusiveDevice::new(spi_dma, cs, Delay).unwrap();

    info!("Create display");
    let mut display = Ili9341::new(spi_device, dc, rst, bcklt, Config::default());

    let start = Instant::now();
    display.initialize(&mut Delay).await.unwrap();

    warn!(
        "Display intialization took  {} milliseconds",
        Instant::now().duration_since(start).as_millis()
    );

    // Create channel to communicate between both tasks
    let channel: &'static mut _ = CHANNEL.init(Channel::new());
    let receiver = channel.receiver();
    let sender = channel.sender();

    // Spawn our tasks
    spawner.spawn(render_task(display, receiver)).ok();
    spawner.spawn(read_dht12_task(i2c0, sender)).ok();

    loop {
        //warn!("Main Loop");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}
