//! Input handling systems
//!
//! Button input processing.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, ButtonResource, GpioResource};

/// Debounce threshold
const DEBOUNCE_THRESHOLD: u8 = 3;

/// System to handle button input
pub fn button_system<'d, T>(
    gpio_res: NonSendMut<GpioResource<'d, T>>,
    mut button_res: NonSendMut<ButtonResource>,
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
                log::info!("BOOT button pressed - Opening Menu");
                app_state.current_mode = AppMode::Menu;
                app_state.needs_redraw = true;
            }
        }
    } else {
        button_res.boot_debounce = 0;
    }

}
