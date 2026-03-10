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
use game::{
    CapturedMonster, CurrentScreen, Exp, Health, InputEvent, InputQueue,
    Level, MonName, PendingCapture, RosterEntities, RosterSlot, Screen, Stats,
};
use ui::{extract_render_data, render_screen};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

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

    // ── Shared raw touch queue (written by touch thread, read by main loop) ──
    let raw_touch_queue: Arc<Mutex<VecDeque<(Option<TouchPoint>, Gesture)>>> =
        Arc::new(Mutex::new(VecDeque::new()));

    // ── Dedicated touch thread ────────────────────────────────────────────
    // Polls the touch controller at 10 ms intervals so no tap is missed,
    // even when the main loop is busy with rendering.
    if touch_ok {
        let i2c_shared = Arc::new(Mutex::new(i2c));
        let queue_t = Arc::clone(&raw_touch_queue);
        let i2c_t   = Arc::clone(&i2c_shared);

        std::thread::Builder::new()
            .stack_size(4096)
            .spawn(move || {
                let touch_drv = Cst816dDriver::new(CST816D_DEVICE_ADDRESS);
                loop {
                    if let Ok(mut guard) = i2c_t.lock() {
                        if let Ok(frame) = touch_drv.get_touch_and_gesture(&mut *guard) {
                            if let Ok(mut q) = queue_t.lock() {
                                q.push_back(frame);
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            })?;

        // i2c_shared's only remaining owner is the thread's Arc clone.
        drop(i2c_shared);
    }
    // When touch_ok is false, `i2c` was never moved and is simply dropped at
    // the end of `run()`.  No warning: Rust sees it referenced in the true branch.

    // ── Touch state tracker (main-loop side) ──────────────────────────────
    let mut touch_was_down = false;
    let mut touch_last: Option<TouchPoint> = None;
    // Set to true when a scroll/swipe gesture fires during a touch sequence;
    // prevents the lift from also generating a TapAt on the Roster screen.
    let mut touch_gesture_fired = false;

    // ── ECS world + schedule ──────────────────────────────────────────────
    let mut world    = game::setup_world();
    let mut schedule = game::build_schedule();

    log::info!("Rustymon started! touch={}", touch_ok);

    loop {
        // ── 1. Gather input → InputEvent list ────────────────────────────
        let mut events: Vec<InputEvent> = Vec::new();
        let screen = world.resource::<CurrentScreen>().0.clone();

        if touch_ok {
            // Drain all raw frames produced by the touch thread.
            let frames: Vec<_> = {
                if let Ok(mut q) = raw_touch_queue.lock() {
                    q.drain(..).collect()
                } else {
                    Vec::new()
                }
            };

            for (opt_point, gesture) in frames {
                let is_down = opt_point.is_some();

                if let Some(ref p) = opt_point {
                    touch_last = Some(TouchPoint { x: p.x, y: p.y });
                }

                if matches!(screen, Screen::Battle) {
                    // Battle: fire TapAt on EVERY frame that has an active touch.
                    if is_down {
                        if let Some(ref p) = opt_point {
                            events.push(InputEvent::TapAt { x: p.x, y: p.y });
                        }
                    }
                } else if matches!(screen, Screen::Roster) {
                    // Roster: gestures scroll; TapAt only fires on a clean lift
                    // (no scroll gesture detected during this touch sequence).
                    if is_down && !touch_was_down {
                        touch_gesture_fired = false; // new touch – reset
                    }
                    if let Some(ev) = gesture_to_input(gesture) {
                        events.push(ev);
                        touch_gesture_fired = true; // swipe consumed this sequence
                    }
                    if !is_down && touch_was_down && !touch_gesture_fired
                        && matches!(gesture, Gesture::None | Gesture::SingleClick | Gesture::DoubleClick)
                    {
                        if let Some(pos) = touch_last {
                            events.push(InputEvent::TapAt { x: pos.x, y: pos.y });
                        }
                    }
                    if !is_down {
                        touch_gesture_fired = false; // ready for next sequence
                    }
                } else {
                    // Overview: gestures + tap-on-lift mapped to semantic events.
                    if let Some(ev) = gesture_to_input(gesture) {
                        events.push(ev);
                    }
                    if !is_down && touch_was_down
                        && matches!(gesture, Gesture::None | Gesture::SingleClick | Gesture::DoubleClick)
                    {
                        if let Some(pos) = touch_last {
                            if let Some(ev) = tap_to_input(pos.x, pos.y, &screen) {
                                events.push(ev);
                            }
                        }
                    }
                }

                touch_was_down = is_down;
            }
        } else {
            // BOOT button fallback (only when touch is unavailable).
            if let Some(btn_ev) = btn.poll() {
                events.push(match btn_ev {
                    ButtonEvent::ShortPress => InputEvent::ToggleCursor,
                    ButtonEvent::LongPress  => InputEvent::Confirm,
                });
            }
        }

        // ── 2. Push events into the ECS InputQueue resource ───────────────
        world.resource_mut::<InputQueue>().0.extend(events);

        // ── 3. Tick ECS schedule (navigation + tap battle update) ─────────
        schedule.run(&mut world);

        // ── 3b. Spawn captured monster if one is pending ──────────────────
        if let Some(cap) = world.resource_mut::<PendingCapture>().0.take() {
            spawn_captured(&mut world, cap);
        }

        // ── 4. Extract snapshot → render → flush ─────────────────────────
        let render_data = extract_render_data(&world);
        render_screen(&mut display, &render_data);
        display.flush()?;

        FreeRtos::delay_ms(50_u32);
    }
}

// ─── Capture helper ───────────────────────────────────────────────────────────

/// Spawn a newly captured monster entity and append it to RosterEntities.
fn spawn_captured(world: &mut bevy_ecs::world::World, cap: CapturedMonster) {
    let slot = world.resource::<RosterEntities>().0.len();
    let entity = world.spawn((
        MonName(cap.name),
        Level(cap.level),
        Stats { atk: cap.atk, def: cap.def },
        Health { hp: cap.hp, max_hp: cap.hp },
        Exp { current: 0, next: (cap.level as u32 + 1) * 100 },
        RosterSlot(slot),
    )).id();
    world.resource_mut::<RosterEntities>().0.push(entity);
}

// ─── Input translation helpers ────────────────────────────────────────────────

fn gesture_to_input(gesture: Gesture) -> Option<InputEvent> {
    match gesture {
        Gesture::SwipeLeft  => Some(InputEvent::CursorToRoster),
        Gesture::SwipeRight => Some(InputEvent::CursorToBattle),
        Gesture::SwipeUp | Gesture::LongPress => Some(InputEvent::Confirm),
        Gesture::SwipeDown  => Some(InputEvent::SelectRoster),
        _ => None,
    }
}

/// Map a tap position to a semantic InputEvent for non-battle screens.
///
/// Overview button layout (240 × 284 px):
///   BATTLE  x: 14–110  y: 244–274   → left half of bottom zone
///   ROSTER  x: 130–226 y: 244–274   → right half of bottom zone
fn tap_to_input(x: u16, y: u16, screen: &Screen) -> Option<InputEvent> {
    match screen {
        Screen::Overview => {
            if y >= 230 {
                if x < 120 { Some(InputEvent::SelectBattle) }
                else        { Some(InputEvent::SelectRoster) }
            } else {
                Some(InputEvent::ToggleCursor)
            }
        }
        Screen::Roster => Some(InputEvent::Back),
        Screen::Battle => None, // handled separately in the main loop
    }
}
