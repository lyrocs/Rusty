//! Input handling systems
//!
//! Button, touch, and gesture input processing.

use bevy_ecs::prelude::*;
use esp_idf_svc::hal::i2c::I2cDriver;
use log::warn;

use crate::display::Gesture;
use crate::ecs::resources::{AppMode, AppState, ButtonResource, GpioResource, TouchResource};

/// TCA9554 GPIO expander I2C address
const TCA9554_ADDRESS: u8 = 0x20;
/// TCA9554 input register (read pin states)
const REG_INPUT: u8 = 0x00;
/// Debounce threshold
const DEBOUNCE_THRESHOLD: u8 = 3;

/// System to handle button input
pub fn button_system<'d, T>(
    gpio_res: NonSendMut<GpioResource<'d, T>>,
    mut button_res: NonSendMut<ButtonResource>,
    mut i2c: NonSendMut<I2cDriver<'d>>,
    mut app_state: ResMut<AppState>,
) where
    T: esp_idf_svc::hal::gpio::Pin + esp_idf_svc::hal::gpio::InputPin,
{
    // Poll BOOT button (GPIO0, active low)
    let boot_pressed = gpio_res.boot_pin.is_low();

    if boot_pressed != button_res.boot_last_state {
        button_res.boot_debounce = button_res.boot_debounce.saturating_add(1);

        if button_res.boot_debounce >= DEBOUNCE_THRESHOLD {
            button_res.boot_last_state = boot_pressed;
            button_res.boot_debounce = 0;

            if boot_pressed {
                log::info!("BOOT button pressed!");
                app_state.current_mode = AppMode::ButtonFeedback;
            } else {
                log::info!("BOOT button released!");
                app_state.current_mode = AppMode::Welcome;
            }
            app_state.needs_redraw = true;
        }
    } else {
        button_res.boot_debounce = 0;
    }

    // Poll PWR button (EXIO4 via GPIO expander, active low)
    if let Ok(pwr_pressed) = read_pwr_button(&mut i2c) {
        if pwr_pressed != button_res.pwr_last_state {
            button_res.pwr_debounce = button_res.pwr_debounce.saturating_add(1);

            if button_res.pwr_debounce >= DEBOUNCE_THRESHOLD {
                button_res.pwr_last_state = pwr_pressed;
                button_res.pwr_debounce = 0;

                if pwr_pressed {
                    log::info!("PWR button pressed!");
                    app_state.current_mode = AppMode::ButtonFeedback;
                } else {
                    log::info!("PWR button released!");
                    app_state.current_mode = AppMode::Welcome;
                }
                app_state.needs_redraw = true;
            }
        } else {
            button_res.pwr_debounce = 0;
        }
    }
}

/// Read PWR button state from GPIO expander (EXIO4)
fn read_pwr_button(i2c: &mut I2cDriver) -> Result<bool, Box<dyn std::error::Error>> {
    let mut input_state = [0u8; 1];
    i2c.write_read(TCA9554_ADDRESS, &[REG_INPUT], &mut input_state, 1000)?;

    // EXIO4 is bit 4, active low
    let pin_high = (input_state[0] & 0b0001_0000) != 0;
    Ok(!pin_high) // Invert for active-low
}

/// System to handle touch and gesture input
pub fn touch_system(
    mut touch_res: NonSendMut<TouchResource>,
    mut i2c: NonSendMut<I2cDriver>,
    mut app_state: ResMut<AppState>,
) {
    // Check for touch
    match touch_res.touch.finger_number(&mut i2c) {
        Ok(count) => {
            let touch_detected = count > 0;

            if touch_detected && !touch_res.last_touch_active {
                // New touch detected
                if let Ok(touches) = touch_res.touch.get_touches(&mut i2c) {
                    if let Some(_point) = touches.first() {
                        log::info!("Touch detected");
                        app_state.current_mode = AppMode::Drawing;
                        app_state.needs_redraw = true;
                    }
                }

                touch_res.last_touch_active = true;
            } else if !touch_detected && touch_res.last_touch_active {
                // Touch released
                log::info!("Touch released");
                touch_res.last_touch_active = false;
            }

            // Check for gestures
            if let Ok(gesture) = touch_res.touch.read_gesture(&mut i2c) {
                match gesture {
                    Gesture::SwipeUp => {
                        log::info!("Gesture: Swipe Up");
                        app_state.current_mode = AppMode::Welcome;
                        app_state.needs_redraw = true;
                    }
                    Gesture::SwipeDown => {
                        log::info!("Gesture: Swipe Down - Playing GIF");
                        app_state.current_mode = AppMode::GifPlaying;
                        app_state.needs_redraw = true;
                    }
                    Gesture::SwipeLeft => {
                        log::info!("Gesture: Swipe Left");
                        app_state.needs_redraw = true;
                    }
                    Gesture::SwipeRight => {
                        log::info!("Gesture: Swipe Right");
                        app_state.needs_redraw = true;
                    }
                    Gesture::DoubleClick => {
                        log::info!("Gesture: Double Click - Reset");
                        app_state.current_mode = AppMode::Welcome;
                        app_state.needs_redraw = true;
                    }
                    Gesture::None => {}
                }
            }
        }
        Err(e) => {
            warn!("Failed to read touch: {:?}", e);
        }
    }
}
