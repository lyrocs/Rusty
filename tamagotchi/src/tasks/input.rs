use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use alloc::sync::Arc;
use bevy_ecs::prelude::*;
use log::{info, warn};

use super::channels::{INPUT_CHANNEL, InputEvent, Button, GestureType};
use crate::ecs::resources::{TouchResource, ButtonResource};

/// Input task - handles touch and button events
/// Polls at 100Hz (10ms) for responsive input while being more efficient than per-frame polling
#[embassy_executor::task]
pub async fn input_task(world: Arc<Mutex<CriticalSectionRawMutex, World>>) {
    info!("[INPUT] Input task started - polling at 100Hz");

    // Input polling at 100Hz (10ms per poll)
    // This is more responsive than 60 FPS (16.67ms) and much better than blocking per-frame
    let mut ticker = Ticker::every(Duration::from_millis(10));

    // State tracking
    let mut last_touch_active = false;
    let mut last_boot_pressed = false;
    let mut last_pwr_pressed = false;

    // Debounce counters (need 3 consecutive reads for state change)
    let mut boot_debounce = 0u8;
    let mut pwr_debounce = 0u8;
    const DEBOUNCE_THRESHOLD: u8 = 3;

    loop {
        ticker.next().await;

        // Lock the world to access input resources
        let mut world_guard = world.lock().await;

        // ===== Touch Handling =====
        if let Some(mut touch_res) = world_guard.get_non_send_resource_mut::<TouchResource>() {
            // Check if touch is detected
            match touch_res.touch.finger_number() {
                Ok(fingers) => {
                    let touch_detected = fingers > 0;

                    if touch_detected && !last_touch_active {
                        // New touch detected - read coordinates
                        if let Ok(touches) = touch_res.touch.get_touches() {
                            if let Some(point) = touches.first() {
                                let event = InputEvent::Touch {
                                    x: point.x,
                                    y: point.y,
                                };

                                // Send touch event (non-blocking)
                                if INPUT_CHANNEL.try_send(event).is_err() {
                                    warn!("[INPUT] Touch event dropped - channel full");
                                }

                                last_touch_active = true;
                            }
                        }

                        // Check for gesture
                        if let Ok(gesture) = touch_res.touch.read_gesture() {
                            use ft3x68_rs::Gesture;
                            let gesture_type = match gesture {
                                Gesture::SwipeUp => Some(GestureType::SwipeUp),
                                Gesture::SwipeDown => Some(GestureType::SwipeDown),
                                Gesture::SwipeLeft => Some(GestureType::SwipeLeft),
                                Gesture::SwipeRight => Some(GestureType::SwipeRight),
                                Gesture::DoubleClick => Some(GestureType::DoubleTap),
                                _ => None,
                            };

                            if let Some(g) = gesture_type {
                                if INPUT_CHANNEL.try_send(InputEvent::Gesture(g)).is_err() {
                                    warn!("[INPUT] Gesture event dropped - channel full");
                                }
                            }
                        }
                    } else if !touch_detected && last_touch_active {
                        // Touch released
                        if INPUT_CHANNEL.try_send(InputEvent::TouchRelease).is_err() {
                            warn!("[INPUT] Touch release event dropped - channel full");
                        }
                        last_touch_active = false;
                    }
                }
                Err(_) => {
                    // Touch read error - skip this frame
                }
            }
        }

        // ===== Button Handling (with debouncing) =====
        if let Some(mut button_res) = world_guard.get_non_send_resource_mut::<ButtonResource>() {
            // Boot button (GPIO0) - active low
            let boot_raw = button_res.boot_button.is_low();

            // Debounce boot button
            if boot_raw == last_boot_pressed {
                boot_debounce = 0; // Same state, reset counter
            } else {
                boot_debounce = boot_debounce.saturating_add(1);

                if boot_debounce >= DEBOUNCE_THRESHOLD {
                    // State change confirmed
                    last_boot_pressed = boot_raw;
                    boot_debounce = 0;

                    let event = if boot_raw {
                        InputEvent::ButtonPress(Button::Boot)
                    } else {
                        InputEvent::ButtonRelease(Button::Boot)
                    };

                    if INPUT_CHANNEL.try_send(event).is_err() {
                        warn!("[INPUT] Boot button event dropped - channel full");
                    } else {
                        info!("[INPUT] Boot button {}", if boot_raw { "pressed" } else { "released" });
                    }
                }
            }

            // Power button (EXIO4 via GPIO expander) - active low
            match button_res.gpio_expander.read_pin(4) {
                Ok(pin_high) => {
                    let pwr_raw = !pin_high; // Invert: active low

                    // Debounce power button
                    if pwr_raw == last_pwr_pressed {
                        pwr_debounce = 0;
                    } else {
                        pwr_debounce = pwr_debounce.saturating_add(1);

                        if pwr_debounce >= DEBOUNCE_THRESHOLD {
                            last_pwr_pressed = pwr_raw;
                            pwr_debounce = 0;

                            let event = if pwr_raw {
                                InputEvent::ButtonPress(Button::Power)
                            } else {
                                InputEvent::ButtonRelease(Button::Power)
                            };

                            if INPUT_CHANNEL.try_send(event).is_err() {
                                warn!("[INPUT] Power button event dropped - channel full");
                            } else {
                                info!("[INPUT] Power button {}", if pwr_raw { "pressed" } else { "released" });
                            }
                        }
                    }
                }
                Err(_) => {
                    // GPIO expander read error - skip
                }
            }
        }

        // Release the lock
        drop(world_guard);

        // Yield to other tasks
        // The ticker already provides timing, this just ensures we're cooperative
    }
}
