mod driver;
mod game;
mod ui;

use driver::{ColorMode, St7789pDriver, LCD_H_RES, LCD_V_RES};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::{PinDriver, Pull},
    peripherals::Peripherals,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::Hertz,
};
use game::{GameState, Screen};
use std::time::Instant;
use ui::render_screen;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = run() {
        log::error!("Fatal error: {:?}", e);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    // ── Display ──────────────────────────────────────────────────────────────
    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        pins.gpio1,
        pins.gpio2,
        None::<esp_idf_svc::hal::gpio::AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;
    let spi_device =
        SpiDeviceDriver::new(&spi_driver, Some(pins.gpio5), &SpiConfig::new().baudrate(Hertz(40_000_000)))?;

    let dc = PinDriver::output(pins.gpio3)?;
    let rst = PinDriver::output(pins.gpio4)?;
    let bl = PinDriver::output(pins.gpio6)?;

    let mut display = St7789pDriver::new(spi_device, dc, rst, LCD_H_RES, LCD_V_RES, ColorMode::Rgb888)?;
    display.set_backlight_pin(bl);
    display.initialize(ColorMode::Rgb888)?;
    display.backlight_on()?;
    display.clear(Rgb888::BLACK)?;
    display.flush()?;

    // ── BOOT button (GPIO9, active-low with internal pull-up) ─────────────
    let mut btn = PinDriver::input(pins.gpio9)?;
    btn.set_pull(Pull::Up)?;

    // ── Game state ────────────────────────────────────────────────────────
    let mut state = GameState::new();

    // ── Button tracking ───────────────────────────────────────────────────
    let mut btn_prev = false;
    let mut press_start: Option<Instant> = None;

    // ── Battle animation timing ───────────────────────────────────────────
    let mut last_line_tick = Instant::now();
    const LINE_DELAY_MS: u128 = 550;

    log::info!("Rustymon started!");

    loop {
        let btn_cur = btn.is_low();

        // Rising edge → record press start
        if btn_cur && !btn_prev {
            press_start = Some(Instant::now());
        }

        // Falling edge → handle release
        if !btn_cur && btn_prev {
            if let Some(start) = press_start.take() {
                let held_ms = start.elapsed().as_millis();
                handle_button_release(&mut state, held_ms);
            }
        }

        btn_prev = btn_cur;

        // ── Advance battle log animation ──────────────────────────────────
        if state.screen == Screen::Battle && !state.battle_is_done() {
            if last_line_tick.elapsed().as_millis() >= LINE_DELAY_MS {
                state.advance_battle_line();
                last_line_tick = Instant::now();
            }
        }

        // ── Render ────────────────────────────────────────────────────────
        render_screen(&mut display, &state);
        display.flush()?;

        FreeRtos::delay_ms(50_u32);
    }
}

/// Dispatch a button release event.
/// < 500 ms → short press  (toggle cursor / no-op)
/// ≥ 500 ms → long press   (confirm / back)
fn handle_button_release(state: &mut GameState, held_ms: u128) {
    match state.screen {
        Screen::Overview => {
            if held_ms < 500 {
                state.toggle_cursor();
            } else {
                state.confirm_selection();
            }
        }
        Screen::Roster => {
            state.go_overview();
        }
        Screen::Battle => {
            if state.battle_is_done() {
                state.go_overview();
            }
            // Ignore presses while battle is still animating
        }
    }
}
