mod driver;
mod game;
mod ui;

use driver::{
    ButtonDriver, ButtonEvent, ColorMode, Cst816dDriver, Gesture, St7789pDriver, TouchPoint,
    CST816D_DEVICE_ADDRESS, LCD_H_RES, LCD_V_RES,
};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use esp_idf_svc::hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::Hertz,
    gpio::PinDriver,
};
use game::{CurrentScreen, InputEvent, InputQueue, Screen};
use ui::{extract_render_data, render_screen};

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

    let dc  = PinDriver::output(pins.gpio3)?;
    let rst = PinDriver::output(pins.gpio4)?;
    let bl  = PinDriver::output(pins.gpio6)?;

    let mut display =
        St7789pDriver::new(spi_device, dc, rst, LCD_H_RES, LCD_V_RES, ColorMode::Rgb888)?;
    display.set_backlight_pin(bl);
    display.initialize(ColorMode::Rgb888)?;
    display.backlight_on()?;
    display.clear(Rgb888::BLACK)?;
    display.flush()?;

    // ── Touch controller (I2C: SDA=GPIO7, SCL=GPIO8) ─────────────────────
    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));
    let mut i2c = I2cDriver::new(peripherals.i2c0, pins.gpio7, pins.gpio8, &i2c_config)?;

    let mut touch = Cst816dDriver::new(CST816D_DEVICE_ADDRESS);
    let touch_ok = touch
        .initialize(&mut i2c)
        .map_err(|e| log::error!("Touch init failed: {:?}", e))
        .is_ok();

    if !touch_ok {
        log::warn!("Touch unavailable – falling back to BOOT button (GPIO9)");
    }

    // ── BOOT button fallback (GPIO9, active-low) ──────────────────────────
    let mut btn = ButtonDriver::new(pins.gpio9)?;

    // ── Touch state tracker ───────────────────────────────────────────────
    let mut touch_was_down = false;
    let mut touch_last: Option<TouchPoint> = None;

    // ── ECS world + schedule ──────────────────────────────────────────────
    let mut world    = game::setup_world();
    let mut schedule = game::build_schedule();

    log::info!("Rustymon started! touch={}", touch_ok);

    loop {
        // ── 1. Gather input → InputEvent list ────────────────────────────
        let mut events: Vec<InputEvent> = Vec::new();

        if touch_ok {
            match touch.get_touch_and_gesture(&mut i2c) {
                Ok((opt_point, gesture)) => {
                    let is_touching = opt_point.is_some();

                    if let Some(ref p) = opt_point {
                        touch_last = Some(TouchPoint { x: p.x, y: p.y });
                    }

                    // Gestures fire immediately on detection
                    if let Some(ev) = gesture_to_input(gesture) {
                        events.push(ev);
                    }

                    // Tap fires on finger-lift
                    if !is_touching && touch_was_down {
                        let fired = if matches!(gesture, Gesture::None) {
                            Gesture::SingleClick
                        } else {
                            gesture
                        };
                        if matches!(fired, Gesture::SingleClick | Gesture::DoubleClick) {
                            if let Some(pos) = touch_last {
                                let screen = world.resource::<CurrentScreen>().0.clone();
                                if let Some(ev) = tap_to_input(pos.x, pos.y, &screen) {
                                    events.push(ev);
                                }
                            }
                        }
                    }

                    touch_was_down = is_touching;
                }
                Err(e) => log::warn!("Touch read error: {:?}", e),
            }
        }

        // BOOT button fallback (only active when touch is unavailable)
        if !touch_ok {
            if let Some(btn_ev) = btn.poll() {
                events.push(match btn_ev {
                    ButtonEvent::ShortPress => InputEvent::ToggleCursor,
                    ButtonEvent::LongPress  => InputEvent::Confirm,
                });
            }
        }

        // ── 2. Push events into the ECS InputQueue resource ───────────────
        world.resource_mut::<InputQueue>().0.extend(events);

        // ── 3. Tick ECS schedule (navigation + battle animation) ──────────
        schedule.run(&mut world);

        // ── 4. Extract snapshot → render → flush ─────────────────────────
        let render_data = extract_render_data(&world);
        render_screen(&mut display, &render_data);
        display.flush()?;

        FreeRtos::delay_ms(50_u32);
    }
}

// ─── Input translation helpers ────────────────────────────────────────────────

/// Map a CST816D gesture to a semantic `InputEvent`.
/// Returns `None` for gestures that are handled as taps (SingleClick / None)
/// or that have no gameplay meaning.
fn gesture_to_input(gesture: Gesture) -> Option<InputEvent> {
    match gesture {
        Gesture::SwipeLeft  => Some(InputEvent::CursorToRoster),
        Gesture::SwipeRight => Some(InputEvent::CursorToBattle),
        Gesture::SwipeUp | Gesture::LongPress => Some(InputEvent::Confirm),
        Gesture::SwipeDown  => Some(InputEvent::SelectRoster),
        _ => None,
    }
}

/// Map a tap position to a semantic `InputEvent` based on the current screen.
///
/// Overview button layout (240 × 284 px):
///   BATTLE  x: 14–110  y: 244–274   → left half of bottom zone
///   ROSTER  x: 130–226 y: 244–274   → right half of bottom zone
fn tap_to_input(x: u16, y: u16, screen: &Screen) -> Option<InputEvent> {
    match screen {
        Screen::Overview => {
            if y >= 230 {
                // Direct button tap
                if x < 120 { Some(InputEvent::SelectBattle) }
                else        { Some(InputEvent::SelectRoster) }
            } else {
                // Tap elsewhere on the overview → cycle cursor
                Some(InputEvent::ToggleCursor)
            }
        }
        Screen::Roster | Screen::Battle => Some(InputEvent::Back),
    }
}
