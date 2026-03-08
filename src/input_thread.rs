//! Input polling thread for Core 0
//!
//! This module implements a dedicated input polling thread that runs on Core 0,
//! separate from the main game loop. This ensures input is never missed even when
//! rendering takes significant time on Core 1.

use crossbeam_channel::Sender;
use esp_idf_svc::hal::gpio::{InputPin, Pin, PinDriver};
use std::thread;
use std::time::{Duration, Instant};

use crate::display::Cst816dDriver;

/// Input events sent from Core 0 to Core 1
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Boot button pressed (GPIO9)
    BootPressed,
    /// Boot button released
    BootReleased,
    /// Power button pressed (GPIO18)
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
/// * `boot_pin` - GPIO9 boot button pin
/// * `pwr_pin` - GPIO18 power button pin
/// * `touch` - Touch controller driver (will be moved to the thread)
/// * `sender` - Channel sender for input events
///
/// # Returns
/// Handle to the spawned thread
///
/// # Note
/// This function accesses the shared static I2C driver for touch
pub fn spawn_input_thread<'d, T, P>(
    boot_pin: PinDriver<'d, T, esp_idf_svc::hal::gpio::Input>,
    pwr_pin: PinDriver<'d, P, esp_idf_svc::hal::gpio::Input>,
    touch: Cst816dDriver,
    sender: Sender<InputEvent>,
) -> InputThreadHandle
where
    T: Pin + InputPin,
    P: Pin + InputPin,
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
            let mut swipe_sent_this_touch = false; // Track if we sent a swipe during current touch
            let mut touch_start_time = Instant::now(); // When current touch started
            let mut touch_start_pos: (u16, u16) = (0, 0); // Touch start position for swipe detection
            let mut touch_current_pos: (u16, u16) = (0, 0); // Current touch position

            // Debounce counters
            let mut boot_debounce = 0u8;
            let mut pwr_debounce = 0u8;
            const DEBOUNCE_THRESHOLD: u8 = 2;

            // Throttle I2C access to reduce bus contention
            let mut i2c_poll_counter = 0u8;
            const I2C_POLL_INTERVAL: u8 = 5; // Poll I2C devices every 5th iteration (50ms)

            loop {
                // === BOOT BUTTON (GPIO9) - Always poll, no I2C needed ===
                let boot_pressed = boot_pin.is_low();

                if boot_pressed != last_boot_pressed {
                    boot_debounce = boot_debounce.saturating_add(1);

                    if boot_debounce >= DEBOUNCE_THRESHOLD {
                        last_boot_pressed = boot_pressed;
                        boot_debounce = 0;

                        let event = if boot_pressed {
                            log::info!("[INPUT] Boot button pressed");
                            InputEvent::BootPressed
                        } else {
                            log::info!("[INPUT] Boot button released");
                            InputEvent::BootReleased
                        };

                        if let Err(e) = sender.send(event) {
                            log::error!("[INPUT] Failed to send boot button event: {:?}", e);
                        }
                    }
                } else {
                    boot_debounce = 0;
                }

                // === PWR BUTTON (GPIO18) - Always poll, no I2C needed ===
                let pwr_pressed = pwr_pin.is_low();

                if pwr_pressed != last_pwr_pressed {
                    pwr_debounce = pwr_debounce.saturating_add(1);

                    if pwr_debounce >= DEBOUNCE_THRESHOLD {
                        last_pwr_pressed = pwr_pressed;
                        pwr_debounce = 0;

                        let event = if pwr_pressed {
                            log::info!("[INPUT] PWR button pressed");
                            InputEvent::PowerPressed
                        } else {
                            log::info!("[INPUT] PWR button released");
                            InputEvent::PowerReleased
                        };

                        if let Err(e) = sender.send(event) {
                            log::error!("[INPUT] Failed to send PWR button event: {:?}", e);
                        }
                    }
                } else {
                    pwr_debounce = 0;
                }

                // === I2C DEVICES - Throttled to reduce bus contention ===
                // Only poll I2C devices every 50ms instead of every 10ms
                i2c_poll_counter = i2c_poll_counter.wrapping_add(1);
                if i2c_poll_counter >= I2C_POLL_INTERVAL {
                    i2c_poll_counter = 0;

                    // === TOUCH CONTROLLER (CST816D) ===
                    // Get I2C from shared static
                    if let Some(i2c) = unsafe { crate::drivers::sd_cs_pin::get_shared_i2c() } {
                        match touch.get_touch(i2c) {
                            Ok(Some(point)) => {
                                // Touch is active
                                if !last_touch_active {
                                    // New touch detected - reset swipe tracking for this touch session
                                    swipe_sent_this_touch = false;
                                    touch_start_time = Instant::now();
                                    touch_start_pos = (point.x, point.y);
                                    touch_current_pos = touch_start_pos;
                                    // Don't send touch event yet - wait to see if it's a swipe
                                    log::info!("[INPUT] Touch started at ({}, {})", point.x, point.y);
                                    last_touch_active = true;
                                } else {
                                    // Touch is ongoing - update current position
                                    touch_current_pos = (point.x, point.y);

                                    // Check for swipe based on movement (software detection)
                                    if !swipe_sent_this_touch {
                                        let dx = touch_current_pos.0 as i32 - touch_start_pos.0 as i32;
                                        let dy = touch_current_pos.1 as i32 - touch_start_pos.1 as i32;
                                        let abs_dx = dx.abs();
                                        let abs_dy = dy.abs();

                                        // Swipe threshold: 50 pixels minimum movement
                                        const SWIPE_THRESHOLD: i32 = 50;

                                        // Check if movement is large enough and predominantly in one direction
                                        if abs_dx > SWIPE_THRESHOLD || abs_dy > SWIPE_THRESHOLD {
                                            let swipe_direction = if abs_dx > abs_dy {
                                                // Horizontal swipe
                                                if dx > 0 {
                                                    Some(SwipeDirection::Right)
                                                } else {
                                                    Some(SwipeDirection::Left)
                                                }
                                            } else {
                                                // Vertical swipe
                                                if dy > 0 {
                                                    Some(SwipeDirection::Down)
                                                } else {
                                                    Some(SwipeDirection::Up)
                                                }
                                            };

                                            if let Some(direction) = swipe_direction {
                                                log::info!("[INPUT] Swipe detected: {:?}", direction);
                                                let event = InputEvent::Swipe { direction };
                                                if let Err(e) = sender.send(event) {
                                                    log::error!("[INPUT] Failed to send swipe event: {:?}", e);
                                                }
                                                swipe_sent_this_touch = true;
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(None) if last_touch_active => {
                                // Touch released - only send touch event if no swipe was detected
                                if !swipe_sent_this_touch {
                                    // This was a tap, not a swipe - send touch event
                                    log::info!("[INPUT] Touch tap at ({}, {})", touch_start_pos.0, touch_start_pos.1);
                                    let event = InputEvent::Touch {
                                        x: touch_start_pos.0,
                                        y: touch_start_pos.1,
                                    };
                                    if let Err(e) = sender.send(event) {
                                        log::error!("[INPUT] Failed to send touch event: {:?}", e);
                                    }
                                } else {
                                    log::info!("[INPUT] Touch released after swipe");
                                }

                                if let Err(e) = sender.send(InputEvent::TouchRelease) {
                                    log::error!("[INPUT] Failed to send touch release: {:?}", e);
                                }
                                last_touch_active = false;
                            }
                            Ok(None) => {
                                // No touch, and wasn't touching before - normal idle state
                            }
                            Err(e) => {
                                log::error!("[INPUT] Touch read error: {:?}", e);
                            }
                        }
                    } else {
                        log::warn!("[INPUT] I2C not available for touch");
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
