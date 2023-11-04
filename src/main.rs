#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _; // global logger
// use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _;
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_pio::Ws2812;

use adafruit_macropad as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    pio::PIOExt,
    watchdog::Watchdog,
    Timer,
};

const RED: RGB8 = RGB8::new(255, 0, 0);
// const GREEN: RGB8 = RGB8::new(0, 255, 0);
// const BLUE: RGB8 = RGB8::new(0, 0, 255);
// const WHITE: RGB8 = RGB8::new(255, 255, 255);

/// Map the LED index to the 2D grid coordinates
const LED_MAP: [&'static [(u8, u8)]; 10] = [
    &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 2), (3, 0), (3, 1), (3, 2)], // 0
    &[(0, 1), (1, 0), (1, 1), (2, 1), (3, 0), (3, 1), (3, 2)],                         // 1
    &[(0, 0), (0, 1), (0, 2), (1, 2), (2, 0), (2, 1), (3, 0), (3, 1), (3, 2)],         // 2
    &[(0, 0), (0, 1), (0, 2), (1, 1), (1, 2), (2, 2), (3, 0), (3, 1), (3, 2)],         // 3
    &[(0, 0), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1), (2, 2), (3, 2)],                 // 4
    &[(0, 0), (0, 1), (0, 2), (1, 0), (2, 1), (2, 2), (3, 0), (3, 1), (3, 2)],         // 5
    &[(0, 0), (0, 1), (0, 2), (1, 0), (2, 0), (2, 1), (2, 2), (3, 0), (3, 1), (3, 2)], // 6
    &[(0, 0), (0, 1), (0, 2), (1, 2), (2, 2), (3, 2)],                                 // 7
    &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1), (2, 2), (3, 0), (3, 1), (3, 2)], // 8
    &[(0, 0), (0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1), (2, 2), (3, 2)]          // 9
];

type LedStates = [RGB8; 12];

/// Get the LED states for a given digit
fn get_led_states(digit: u8) -> LedStates {
    let mut led_states = [RGB8::default(); 12];

    for &(x, y) in LED_MAP[digit as usize] {
        let index = map_coord_to_index(x, y);
        led_states[index as usize] = RED;
    }

    led_states
}

/// Map the 2D grid coordinates to the LED index
fn map_coord_to_index(x: u8, y: u8) -> u8 {
    x * 3 + y
}

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    // Setup PIO
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    // Setup Neopixel RGB LED
    let mut ws = Ws2812::new(
        pins.neopixel.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    loop {
        for digit in 0..=9 {
            let led_states = get_led_states(digit as u8);
            ws.write(led_states.iter().copied()).unwrap();
            delay.delay_ms(500);
        }
    }
}
