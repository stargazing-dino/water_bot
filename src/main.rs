#![no_std]
#![no_main]

use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_time::rate::Extensions;

// The macro for our start-up function
use rp_pico::entry;

// GPIO traits
use embedded_hal::digital::v2::OutputPin;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Pull in any important traits
use rp_pico::hal::prelude::*;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use rp_pico::hal::pac;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;

// The graphics library we'll use to draw to the screen
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Line,
    text::{Text, TextStyleBuilder},
};

use epd_waveshare::{
    color::*, epd2in13_v2::Display2in13 as EPDisplay, epd2in13_v2::Epd2in13 as EPD,
    epd2in13_v2::HEIGHT, epd2in13_v2::WIDTH, graphics::DisplayRotation, prelude::*,
};

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then blinks the LED in an
/// infinite loop.
#[entry]
fn entry() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // SPI declaration
    let _spi_sclk = pins.gpio10.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio11.into_mode::<hal::gpio::FunctionSpi>();
    let spi = hal::spi::Spi::<_, _, 8>::new(pac.SPI1);

    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        // you can put cookie (increase the speed) in it but I don't recommend it.
        4_000_000u32.Hz().into(),
        &embedded_hal::spi::MODE_0,
    );
    // End of SPI declaration

    // Start the rest of pins needed to communicate with the screen
    let mut cs = pins.gpio9.into_push_pull_output(); // CS
    cs.set_high().unwrap();
    let busy = pins.gpio13.into_pull_up_input(); // BUSY
    let dc = pins.gpio8.into_push_pull_output(); // DC
    let rst = pins.gpio12.into_push_pull_output(); // RST

    // Start the EPD struct
    let mut epd = EPD::new(
        &mut spi,   // SPI
        cs,         // CS
        busy,       // BUSY
        dc,         // DC
        rst,        // RST
        &mut delay, // DELAY
    )
    .unwrap();
    // Start the display buffer
    let mut display = EPDisplay::default();

    epd.wake_up(&mut spi, &mut delay).unwrap();
    // epd.set_background_color(epd_waveshare::color::Color::Black);
    // epd.clear_frame(&mut spi).unwrap();
    // epd.set_background_color(epd_waveshare::color::Color::White);
    epd.clear_frame(&mut spi, &mut delay).unwrap();

    display.clear_buffer(Color::Black);

    // Start the fun
    let style_text = TextStyleBuilder::new()
        // .text_color(BinaryColor::Off)
        // .background_color(BinaryColor::On)
        .build();
    let style_forms = PrimitiveStyleBuilder::new()
        .stroke_width(10)
        .stroke_color(BinaryColor::Off)
        .fill_color(BinaryColor::Off)
        .build();

    for i in 0..(WIDTH / 10) as i32 {
        Line::new(Point::new(i * 10, 0), Point::new(i * 10, HEIGHT as i32))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
            .draw(&mut display)
            .unwrap();
    }
    for i in 0..(HEIGHT / 10) as i32 {
        Line::new(Point::new(0, i * 10), Point::new(WIDTH as i32, i * 10))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::Off, 1))
            .draw(&mut display)
            .unwrap();
    }

    display.set_rotation(DisplayRotation::Rotate270);

    // If you want to show an image, you can use BMP or TGA
    // let tga = Tga::from_slice(include_bytes!(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/RustMX.tga"
    // )))
    // .unwrap();

    // let image: Image<Tga, BinaryColor> = Image::new(&tga, Point::new(50, 100));
    // image.draw(&mut display).unwrap();

    Text::new("RustMX", Point::new(50, 50), style_text)
        .draw(&mut display)
        .unwrap();

    epd.update_frame(&mut spi, display.buffer(), &mut delay)
        .unwrap();
    epd.display_frame(&mut spi, &mut delay).unwrap();
    delay.delay_ms(1000);

    // Set the LED to be an output
    let mut led_pin = pins.led.into_push_pull_output();

    // Blink the LED at 1 Hz
    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(1000);
        led_pin.set_low().unwrap();
        delay.delay_ms(1000);
    }
}
