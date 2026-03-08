mod driver;
mod game;
mod ui;

use driver::{
    ColorMode, Cst816dDriver, Gesture, St7789pDriver, TouchPoint, CST816D_DEVICE_ADDRESS,
    LCD_H_RES, LCD_V_RES,
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::{PinDriver, Pull},
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::Hertz,
};
use game::{GameState, MenuCursor, Screen};
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

    // ── Display (SPI: SCK=GPIO1, MOSI=GPIO2, CS=GPIO5, DC=GPIO3, RST=GPIO4, BL=GPIO6) ──
    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        pins.gpio1,
        pins.gpio2,
        None::<esp_idf_svc::hal::gpio::AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;
    let spi_device = SpiDeviceDriver::new(
        &spi_driver,
        Some(pins.gpio5),
        &SpiConfig::new().baudrate(Hertz(40_000_000)),
    )?;

    let dc = PinDriver::output(pins.gpio3)?;
    let rst = PinDriver::output(pins.gpio4)?;
    let bl = PinDriver::output(pins.gpio6)?;

    let mut display =
        St7789pDriver::new(spi_device, dc, rst, LCD_H_RES, LCD_V_RES, ColorMode::Rgb888)?;
    display.set_backlight_pin(bl);
    display.initialize(ColorMode::Rgb888)?;
    display.backlight_on()?;
    display.clear(Rgb888::BLACK)?;
    display.flush()?;

    // ── Touch (I2C: SDA=GPIO7, SCL=GPIO8) ────────────────────────────────
    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));
    let mut i2c = I2cDriver::new(peripherals.i2c0, pins.gpio7, pins.gpio8, &i2c_config)?;

    let mut touch = Cst816dDriver::new(CST816D_DEVICE_ADDRESS);
    let touch_ok = touch.initialize(&mut i2c).map_err(|e| log::error!("Touch init failed: {:?}", e)).is_ok();
    if !touch_ok {
        log::warn!("Touch screen unavailable – falling back to BOOT button (GPIO9)");
    }

    // ── BOOT button fallback (GPIO9, active-low) ──────────────────────────
    let mut btn = PinDriver::input(pins.gpio9)?;
    btn.set_pull(Pull::Up)?;

    // ── Game state ────────────────────────────────────────────────────────
    let mut state = GameState::new();

    // ── Touch tracker ─────────────────────────────────────────────────────
    let mut touch_was_down = false;
    let mut touch_last_pos: Option<TouchPoint> = None;

    // ── BOOT button tracker ───────────────────────────────────────────────
    let mut btn_prev = false;
    let mut btn_press_start: Option<Instant> = None;

    // ── Battle animation timing ───────────────────────────────────────────
    let mut last_line_tick = Instant::now();
    const LINE_DELAY_MS: u128 = 550;

    log::info!("Rustymon started! Touch: {}", if touch_ok { "OK" } else { "fallback" });

    loop {
        // ── Touch input ───────────────────────────────────────────────────
        if touch_ok {
            match touch.get_touch_and_gesture(&mut i2c) {
                Ok((opt_point, gesture)) => {
                    let is_touching = opt_point.is_some();

                    if let Some(ref p) = opt_point {
                        touch_last_pos = Some(TouchPoint { x: p.x, y: p.y });
                    }

                    // Swipe / long-press gestures fire immediately
                    match gesture {
                        Gesture::SwipeLeft | Gesture::SwipeRight
                        | Gesture::SwipeUp | Gesture::SwipeDown
                        | Gesture::LongPress => {
                            handle_gesture(&mut state, gesture);
                        }
                        _ => {}
                    }

                    // Tap fires on finger-lift (falling edge)
                    if !is_touching && touch_was_down {
                        let fired_gesture = if gesture == Gesture::None {
                            Gesture::SingleClick
                        } else {
                            gesture
                        };
                        if matches!(fired_gesture, Gesture::SingleClick | Gesture::DoubleClick) {
                            if let Some(pos) = touch_last_pos {
                                handle_tap(&mut state, pos.x, pos.y);
                            }
                        }
                    }

                    touch_was_down = is_touching;
                }
                Err(e) => log::warn!("Touch read error: {:?}", e),
            }
        }

        // ── BOOT button fallback ──────────────────────────────────────────
        if !touch_ok {
            let btn_cur = btn.is_low();
            if btn_cur && !btn_prev {
                btn_press_start = Some(Instant::now());
            }
            if !btn_cur && btn_prev {
                if let Some(start) = btn_press_start.take() {
                    let held_ms = start.elapsed().as_millis();
                    handle_btn_release(&mut state, held_ms);
                }
            }
            btn_prev = btn_cur;
        }

        // ── Battle animation ──────────────────────────────────────────────
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

// ─── Touch gesture handler ────────────────────────────────────────────────────
// Swipes and long-press: navigate between screens or cycle the cursor.

fn handle_gesture(state: &mut GameState, gesture: Gesture) {
    match state.screen {
        Screen::Overview => match gesture {
            // Swipe horizontally → move cursor
            Gesture::SwipeLeft => state.cursor = MenuCursor::Roster,
            Gesture::SwipeRight => state.cursor = MenuCursor::Battle,
            // Swipe up / long-press → confirm selected option
            Gesture::SwipeUp | Gesture::LongPress => state.confirm_selection(),
            // Swipe down → go to roster
            Gesture::SwipeDown => state.go_roster(),
            _ => {}
        },
        Screen::Roster => {
            // Any swipe / long-press → back to overview
            state.go_overview();
        }
        Screen::Battle => {
            if state.battle_is_done() {
                state.go_overview();
            }
        }
    }
}

// ─── Touch tap handler ────────────────────────────────────────────────────────
// Maps screen-space coordinates to game actions.
//
// Overview button layout (240×284 display):
//   BATTLE  x: 14–110  y: 244–274
//   ROSTER  x: 130–226 y: 244–274

fn handle_tap(state: &mut GameState, x: u16, y: u16) {
    match state.screen {
        Screen::Overview => {
            if y >= 230 {
                // Bottom zone: direct button tap
                if x < 120 {
                    state.cursor = MenuCursor::Battle;
                    state.start_battle();
                } else {
                    state.cursor = MenuCursor::Roster;
                    state.go_roster();
                }
            } else {
                // Tap anywhere else → toggle highlighted button
                state.toggle_cursor();
            }
        }
        Screen::Roster => {
            state.go_overview();
        }
        Screen::Battle => {
            if state.battle_is_done() {
                state.go_overview();
            }
        }
    }
}

// ─── BOOT button fallback handler ─────────────────────────────────────────────
// Short press (<500 ms) = toggle cursor, long press = confirm.

fn handle_btn_release(state: &mut GameState, held_ms: u128) {
    match state.screen {
        Screen::Overview => {
            if held_ms < 500 {
                state.toggle_cursor();
            } else {
                state.confirm_selection();
            }
        }
        Screen::Roster => state.go_overview(),
        Screen::Battle => {
            if state.battle_is_done() {
                state.go_overview();
            }
        }
    }
}
