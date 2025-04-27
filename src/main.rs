#![no_std]
#![no_main]

mod graph;

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, iso_8859_15::FONT_5X7, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};
use embedded_hal::digital::v2::OutputPin;
use embedded_layout::{
    layout::linear::{spacing::DistributeFill, FixedMargin, LinearLayout},
    prelude::{horizontal, vertical, Align, Chain},
};
use embedded_plots::curve::PlotPoint;
use graph::MoisturePlot;

use heapless::Vec;
use rp_pico::{entry, hal};

use epd_waveshare::{
    color::*, epd2in13_v2::Display2in13 as EPDisplay, epd2in13_v2::Epd2in13 as EPD,
    graphics::DisplayRotation, prelude::*,
};

use rp_pico::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    spi,
    watchdog::Watchdog,
};

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Start all the peripherals
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Set up the watchdog driver - needed by the clock setup code
    let clocks = init_clocks_and_plls(
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
    let sio = Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // SPI declaration
    let _spi_sclk = pins.gpio10.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio11.into_mode::<hal::gpio::FunctionSpi>();
    let spi = spi::Spi::<_, _, 8>::new(pac.SPI1);
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        fugit::RateExtU32::Hz(16_000_000u32),
        &embedded_hal::spi::MODE_0,
    );

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

    epd.wake_up(&mut spi, &mut delay).unwrap();
    epd.clear_frame(&mut spi, &mut delay).unwrap();

    // Start the display buffer
    let mut display = EPDisplay::default();
    display.clear_buffer(Color::White);
    display.set_rotation(DisplayRotation::Rotate90);

    let display_area = display.bounding_box();

    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let height = display_area.size.height;
    let width = display_area.size.width;
    let half_height = height / 2;
    let half_width = width / 2;

    let circle = Circle::new(Point::zero(), 11).into_styled(thin_stroke);
    let device_name = Text::new(
        "joe mama",
        Point::zero(),
        MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
    );
    let date = Text::new(
        "Tue, 11",
        Point::zero(),
        MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
    );
    let status = Text::new(
        "Status:",
        Point::zero(),
        MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
    );
    let watering_can = Circle::new(Point::zero(), 11).into_styled(thin_stroke);
    let last_watering = Text::new(
        "last 12/23/2019",
        Point::zero(),
        MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
    );

    let header = LinearLayout::vertical(Chain::new(device_name).append(date))
        .with_alignment(horizontal::Left)
        .with_spacing(FixedMargin(4))
        .arrange();
    let watering_status = LinearLayout::horizontal(Chain::new(status).append(watering_can))
        .with_spacing(FixedMargin(8))
        .arrange();
    let watering_status_and_last =
        LinearLayout::vertical(Chain::new(watering_status).append(last_watering))
            .with_alignment(horizontal::Right)
            .with_spacing(FixedMargin(4))
            .arrange();
    let mut data: Vec<PlotPoint, 4> = Vec::new();

    let _ = data.push(PlotPoint { x: 0, y: 0 });
    let _ = data.push(PlotPoint { x: 1, y: 2 });
    let _ = data.push(PlotPoint { x: 2, y: 2 });
    let _ = data.push(PlotPoint { x: 3, y: 0 });

    let plot = MoisturePlot::new(&data, Point::zero(), Size::new(half_width, half_height));
    let left = LinearLayout::vertical(Chain::new(header).append(plot))
        .with_spacing(DistributeFill(height))
        .arrange();
    let right = LinearLayout::vertical(Chain::new(watering_status_and_last).append(circle))
        .with_spacing(DistributeFill(height))
        .arrange();

    LinearLayout::horizontal(Chain::new(left).append(right))
        .align_to(
            &display_area,
            horizontal::NoAlignment,
            vertical::NoAlignment,
        )
        .with_spacing(DistributeFill(width))
        .arrange()
        .draw(&mut display)
        .unwrap();

    // Update the display with what we have
    epd.update_and_display_frame(&mut spi, display.buffer(), &mut delay)
        .unwrap();

    let mut led_pin = pins.gpio25.into_push_pull_output();

    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}
