//! Input polling thread for Core 0
//!
//! This module implements a dedicated input polling thread that runs on Core 0,
//! separate from the main game loop. This ensures input is never missed even when
//! rendering takes significant time on Core 1.

use crossbeam_channel::Sender;
use esp_idf_svc::hal::gpio::{InputPin, Pin, PinDriver};
use std::thread;
use std::time::Duration;

use crate::display::{Ft3x68Driver, ft3x68::Gesture};

/// TCA9554 GPIO expander I2C address (for PWR button on EXIO4)
const TCA9554_ADDRESS: u8 = 0x20;
/// TCA9554 input register
const REG_INPUT: u8 = 0x00;

/// Input events sent from Core 0 to Core 1
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Boot button pressed (GPIO0)
    BootPressed,
    /// Boot button released
    BootReleased,
    /// Power button pressed (EXIO4)
    PowerPressed,
    /// Power button released
    PowerReleased,
    /// Touch detected at coordinates
    Touch { x: u16, y: u16 },
    /// Touch released
    TouchRelease,
    /// Swipe gesture detected
    Swipe { direction: SwipeDirection },
}

/// Swipe directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Input polling thread state
pub struct InputThreadHandle {
    pub thread_handle: Option<thread::JoinHandle<()>>,
}

/// Spawn the input polling thread on Core 0
///
/// # Arguments
/// * `boot_pin` - GPIO0 boot button pin
/// * `touch` - Touch controller driver (will be moved to the thread)
/// * `sender` - Channel sender for input events
///
/// # Returns
/// Handle to the spawned thread
///
/// # Note
/// This function accesses the shared static I2C driver for touch and PWR button
pub fn spawn_input_thread<'d, T>(
    boot_pin: PinDriver<'d, T, esp_idf_svc::hal::gpio::Input>,
    mut touch: Ft3x68Driver,
    sender: Sender<InputEvent>,
) -> InputThreadHandle
where
    T: Pin + InputPin,
    'd: 'static,
{
    log::info!("[INPUT] Spawning input polling thread on Core 0...");

    let thread_handle = thread::Builder::new()
        .name("input_core0".to_string())
        .stack_size(8192) // 8KB stack
        .spawn(move || {
            log::info!("[INPUT] Input thread running (CPU affinity best-effort)");

            // State tracking for edge detection
            let mut last_boot_pressed = false;
            let mut last_pwr_pressed = false;
            let mut last_touch_active = false;
            let mut last_gesture = Gesture::None;

            // Debounce counters
            let mut boot_debounce = 0u8;
            let mut pwr_debounce = 0u8;
            const DEBOUNCE_THRESHOLD: u8 = 2;

            // Throttle I2C access to reduce bus contention
            let mut i2c_poll_counter = 0u8;
            const I2C_POLL_INTERVAL: u8 = 5; // Poll I2C devices every 5th iteration (50ms)

            loop {
                // === BOOT BUTTON (GPIO0) - Always poll, no I2C needed ===
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

                        if let Err(e) = sender.send(event) {
                            log::error!("[INPUT] Failed to send boot button event: {:?}", e);
                        }
                    }
                } else {
                    boot_debounce = 0;
                }

                // === I2C DEVICES - Throttled to reduce bus contention ===
                // Only poll I2C devices every 50ms instead of every 10ms
                i2c_poll_counter = i2c_poll_counter.wrapping_add(1);
                if i2c_poll_counter >= I2C_POLL_INTERVAL {
                    i2c_poll_counter = 0;

                    // === POWER BUTTON (EXIO4 via I2C) ===
                    // Get I2C from shared static
                    if let Some(i2c) = unsafe { crate::drivers::sd_cs_pin::get_shared_i2c() } {
                        let mut buffer = [0u8; 1];
                        if i2c.write_read(TCA9554_ADDRESS, &[REG_INPUT], &mut buffer, 100).is_ok() {
                            let pin_high = (buffer[0] & 0b0001_0000) != 0;
                            let pwr_pressed = !pin_high; // Active low

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

                                    if let Err(e) = sender.send(event) {
                                        log::error!("[INPUT] Failed to send power button event: {:?}", e);
                                    }
                                }
                            } else {
                                pwr_debounce = 0;
                            }
                        }
                    }

                    // === TOUCH CONTROLLER ===
                    // Get I2C from shared static
                    if let Some(i2c) = unsafe { crate::drivers::sd_cs_pin::get_shared_i2c() } {
                        // Check for gestures first (only emit on edge: None -> Swipe)
                        if let Ok(gesture) = touch.read_gesture(i2c) {
                            // Only send event when gesture changes from None to a swipe
                            if gesture != last_gesture {
                                let swipe_direction = match gesture {
                                    Gesture::SwipeUp => Some(SwipeDirection::Up),
                                    Gesture::SwipeDown => Some(SwipeDirection::Down),
                                    Gesture::SwipeLeft => Some(SwipeDirection::Left),
                                    Gesture::SwipeRight => Some(SwipeDirection::Right),
                                    _ => None,
                                };

                                if let Some(direction) = swipe_direction {
                                    log::info!("[INPUT] Swipe detected: {:?}", direction);
                                    let event = InputEvent::Swipe { direction };
                                    if let Err(e) = sender.send(event) {
                                        log::error!("[INPUT] Failed to send swipe event: {:?}", e);
                                    }
                                }

                                last_gesture = gesture;
                            }
                        }

                        if let Ok(count) = touch.finger_number(i2c) {
                            let touch_active = count > 0;

                            if touch_active && !last_touch_active {
                                // New touch detected
                                if let Ok(touches) = touch.get_touches(i2c) {
                                    if let Some(point) = touches.first() {
                                        let event = InputEvent::Touch {
                                            x: point.x,
                                            y: point.y,
                                        };

                                        if let Err(e) = sender.send(event) {
                                            log::error!("[INPUT] Failed to send touch event: {:?}", e);
                                        }
                                    }
                                }
                                last_touch_active = true;
                            } else if !touch_active && last_touch_active {
                                // Touch released
                                if let Err(e) = sender.send(InputEvent::TouchRelease) {
                                    log::error!("[INPUT] Failed to send touch release: {:?}", e);
                                }
                                last_touch_active = false;
                                // Reset gesture state so next swipe in same direction can be detected
                                last_gesture = Gesture::None;
                            }
                        }
                    }
                }

                // Poll at 100Hz (10ms) - I2C devices only polled every 50ms
                thread::sleep(Duration::from_millis(10));
            }
        })
        .expect("Failed to spawn input thread");

    InputThreadHandle {
        thread_handle: Some(thread_handle),
    }
}
