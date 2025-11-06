//! Rendering system
//!
//! Handles display updates based on app state.

use bevy_ecs::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::Text,
};
use log::info;

use crate::display::GifPlayer;
use crate::ecs::resources::{AppMode, AppState, DisplayResource};

/// Embedded GIF animation data
const GIF_DATA: &[u8] = include_bytes!("../../assets/80.gif");

/// System to render the display
pub fn render_system(mut display_res: NonSendMut<DisplayResource>, mut app_state: ResMut<AppState>) {
    // Only render if redraw is needed
    if !app_state.needs_redraw {
        return;
    }

    let display = &mut display_res.display;

    match app_state.current_mode {
        AppMode::Welcome => {
            draw_welcome_screen(display).ok();
        }
        AppMode::Drawing => {
            // Drawing mode - keep current display
        }
        AppMode::GifPlaying => {
            play_gif_animation(display).ok();
            // After GIF, return to welcome
            app_state.current_mode = AppMode::Welcome;
            draw_welcome_screen(display).ok();
        }
        AppMode::ButtonFeedback => {
            display.clear(Rgb888::new(50, 0, 50)).ok();
            let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
            Text::new("Button pressed", Point::new(10, 30), text_style).draw(display).ok();
            display.flush().ok();
        }
    }

    app_state.needs_redraw = false;
}

/// Draw the initial welcome screen
fn draw_welcome_screen(display: &mut crate::display::Sh8601Driver) -> Result<(), Box<dyn std::error::Error>> {
    display.clear(Rgb888::BLACK)?;

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);
    Text::new("stdgotchi", Point::new(10, 30), text_style).draw(display)?;
    Text::new("ESP32-S3 AMOLED", Point::new(10, 50), text_style).draw(display)?;
    Text::new("Touch & Gestures!", Point::new(10, 70), text_style).draw(display)?;
    Text::new("Swipe down for GIF", Point::new(10, 90), text_style).draw(display)?;

    Circle::new(Point::new(50, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
        .draw(display)?;

    Circle::new(Point::new(100, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
        .draw(display)?;

    Circle::new(Point::new(150, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::MAGENTA))
        .draw(display)?;

    display.flush()?;
    Ok(())
}

/// Play the GIF animation at a fixed position
fn play_gif_animation(display: &mut crate::display::Sh8601Driver) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading GIF animation...");

    let mut player = GifPlayer::new(GIF_DATA)?;
    let (width, height) = player.dimensions();
    info!("GIF dimensions: {}x{}, frames: {}", width, height, player.frame_count());

    // Calculate centered position for the GIF
    let display_size = display.size();
    let pos_x = (display_size.width as i32 - width as i32) / 2;
    let pos_y = (display_size.height as i32 - height as i32) / 2;

    info!("Rendering GIF at position: ({}, {})", pos_x, pos_y);

    // Clear screen before animation
    display.clear(Rgb888::BLACK)?;

    // Play animation loop (3 complete loops) at fixed position
    let total_frames = player.frame_count() * 3;
    for _ in 0..total_frames {
        let delay = player.next_frame(display, Some((pos_x, pos_y)))?;
        display.flush()?;
        std::thread::sleep(delay);
    }

    info!("GIF animation completed");
    Ok(())
}
