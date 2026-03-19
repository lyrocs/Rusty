//! Dedicated input polling thread.
//!
//! Runs on a separate thread polling touch (I2C) and buttons (GPIO) so no
//! input is missed during rendering. Based on stdgotchi's input_thread.rs
//! pattern with crossbeam channel.

use crossbeam_channel::Sender;
use std::thread;
use std::time::Duration;

use super::{InputEvent, SwipeDirection};
use crate::input::touch::{Cst816dDriver, CST816D_DEVICE_ADDRESS};

// Global I2C driver for touch access from the input thread.
// Using a raw pointer to avoid static_mut_refs warnings while maintaining
// the same pattern used in the existing stdgotchi/rustymon projects.
static TOUCH_I2C: std::sync::atomic::AtomicPtr<esp_idf_svc::hal::i2c::I2cDriver<'static>> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// Initialize the shared I2C driver for the input thread.
///
/// # Safety
/// Must be called exactly once before `spawn_input_thread`.
pub unsafe fn init_touch_i2c(i2c: &'static mut esp_idf_svc::hal::i2c::I2cDriver<'static>) {
    TOUCH_I2C.store(i2c as *mut _, std::sync::atomic::Ordering::Release);
}

/// Get mutable access to the shared I2C driver.
///
/// # Safety
/// Caller must ensure exclusive access (only the input thread uses this).
pub unsafe fn get_touch_i2c() -> Option<&'static mut esp_idf_svc::hal::i2c::I2cDriver<'static>> {
    let ptr = TOUCH_I2C.load(std::sync::atomic::Ordering::Acquire);
    if ptr.is_null() { None } else { Some(&mut *ptr) }
}

/// Spawn the input polling thread.
///
/// Polls buttons at 100Hz and touch at 20Hz (I2C throttled).
/// Sends high-level `InputEvent`s through the crossbeam channel.
pub fn spawn_input_thread(
    boot_pin: esp_idf_svc::hal::gpio::PinDriver<
        'static,
        esp_idf_svc::hal::gpio::Gpio9,
        esp_idf_svc::hal::gpio::Input,
    >,
    pwr_pin: esp_idf_svc::hal::gpio::PinDriver<
        'static,
        esp_idf_svc::hal::gpio::Gpio18,
        esp_idf_svc::hal::gpio::Input,
    >,
    sender: Sender<InputEvent>,
    swipe_threshold: i32,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("input".to_string())
        .stack_size(8192)
        .spawn(move || {
            log::info!("[INPUT] Input thread started");

            let touch = Cst816dDriver::new(CST816D_DEVICE_ADDRESS);

            // Button debounce state
            let mut last_boot_pressed = false;
            let mut last_pwr_pressed = false;
            let mut boot_debounce = 0u8;
            let mut pwr_debounce = 0u8;
            const DEBOUNCE_THRESHOLD: u8 = 2;

            // Touch/swipe state
            let mut last_touch_active = false;
            let mut swipe_sent = false;
            let mut touch_start_pos: (u16, u16) = (0, 0);
            let mut touch_current_pos: (u16, u16) = (0, 0);

            // I2C throttle
            let mut i2c_counter = 0u8;
            const I2C_POLL_INTERVAL: u8 = 5; // every 50ms

            loop {
                // ── BOOT button (GPIO9) ──
                let boot_pressed = boot_pin.is_low();
                if boot_pressed != last_boot_pressed {
                    boot_debounce = boot_debounce.saturating_add(1);
                    if boot_debounce >= DEBOUNCE_THRESHOLD {
                        last_boot_pressed = boot_pressed;
                        boot_debounce = 0;
                        let event = if boot_pressed {
                            InputEvent::BootPressed
                        } else {
                            InputEvent::BootReleased
                        };
                        let _ = sender.send(event);
                    }
                } else {
                    boot_debounce = 0;
                }

                // ── PWR button (GPIO18) ──
                let pwr_pressed = pwr_pin.is_low();
                if pwr_pressed != last_pwr_pressed {
                    pwr_debounce = pwr_debounce.saturating_add(1);
                    if pwr_debounce >= DEBOUNCE_THRESHOLD {
                        last_pwr_pressed = pwr_pressed;
                        pwr_debounce = 0;
                        let event = if pwr_pressed {
                            InputEvent::PowerPressed
                        } else {
                            InputEvent::PowerReleased
                        };
                        let _ = sender.send(event);
                    }
                } else {
                    pwr_debounce = 0;
                }

                // ── Touch (I2C, throttled) ──
                i2c_counter = i2c_counter.wrapping_add(1);
                if i2c_counter >= I2C_POLL_INTERVAL {
                    i2c_counter = 0;

                    if let Some(i2c) = unsafe { get_touch_i2c() } {
                        match touch.get_touch(i2c) {
                            Ok(Some(point)) => {
                                if !last_touch_active {
                                    // New touch
                                    swipe_sent = false;
                                    touch_start_pos = (point.x, point.y);
                                    touch_current_pos = touch_start_pos;
                                    let _ = sender.send(InputEvent::TouchDown {
                                        x: point.x,
                                        y: point.y,
                                    });
                                    last_touch_active = true;
                                } else {
                                    touch_current_pos = (point.x, point.y);

                                    // Software swipe detection
                                    if !swipe_sent {
                                        let dx = touch_current_pos.0 as i32
                                            - touch_start_pos.0 as i32;
                                        let dy = touch_current_pos.1 as i32
                                            - touch_start_pos.1 as i32;
                                        let abs_dx = dx.abs();
                                        let abs_dy = dy.abs();

                                        if abs_dx > swipe_threshold || abs_dy > swipe_threshold {
                                            let direction = if abs_dx > abs_dy {
                                                if dx > 0 {
                                                    SwipeDirection::Right
                                                } else {
                                                    SwipeDirection::Left
                                                }
                                            } else {
                                                if dy > 0 {
                                                    SwipeDirection::Down
                                                } else {
                                                    SwipeDirection::Up
                                                }
                                            };
                                            let _ = sender.send(InputEvent::Swipe(direction));
                                            swipe_sent = true;
                                        }
                                    }
                                }
                            }
                            Ok(None) if last_touch_active => {
                                // Touch released
                                if !swipe_sent {
                                    let _ = sender.send(InputEvent::Tap {
                                        x: touch_start_pos.0,
                                        y: touch_start_pos.1,
                                    });
                                }
                                let _ = sender.send(InputEvent::TouchUp);
                                last_touch_active = false;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                log::error!("[INPUT] Touch read error: {:?}", e);
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        })
        .expect("Failed to spawn input thread")
}
