//! Rendering system
//!
//! Handles display updates based on app state.

use bevy_ecs::prelude::*;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};
use log::info;

use crate::ecs::resources::{AppMode, AppState, DisplayResource, PageResource};

/// System to render the display
pub fn render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut app_state: ResMut<AppState>,
    page_res: Option<NonSendMut<PageResource>>,
) {
    let display = &mut display_res.display;

    match app_state.current_mode {
        AppMode::Welcome => {
            // Only render full screen if redraw is needed
            if app_state.needs_redraw {
                draw_welcome_screen(display, app_state.fps).ok();
                app_state.needs_redraw = false;
            } else {
                // Just update FPS overlay
                draw_fps_overlay(display, app_state.fps).ok();
            }
        }
        AppMode::Drawing => {
            // Drawing mode - just update FPS
            draw_fps_overlay(display, app_state.fps).ok();
        }
        AppMode::GifPlaying => {
            // Page-based rendering
            if let Some(mut page_res) = page_res {
                // Update FPS in page if it's a BattlePage
                if let Some(battle_page) = page_res.page.as_any_mut().downcast_mut::<crate::ui::pages::BattlePage>() {
                    battle_page.set_fps(app_state.fps);
                }

                // Update page logic
                let page_active = page_res.page.update();

                // Check if page needs full redraw
                let full_redraw = page_res.page.needs_full_redraw();

                // Draw the page with appropriate redraw mode
                // Page handles its own clearing (full background on first frame, sprite zones on subsequent frames)
                if let Err(e) = page_res.page.draw(display, full_redraw) {
                    log::error!("Failed to draw page: {:?}", e);
                }

                // Reset needs_redraw flag if it was set
                if app_state.needs_redraw {
                    app_state.needs_redraw = false;
                }

                // Check if page is done
                if !page_active {
                    info!("Page completed");
                    app_state.current_mode = AppMode::Welcome;
                    app_state.needs_redraw = true;
                    // Note: PageResource will be removed by cleanup system
                }
            }
        }
        AppMode::ButtonFeedback => {
            if app_state.needs_redraw {
                display.clear(Rgb888::new(50, 0, 50)).ok();
                let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                Text::new("Button pressed", Point::new(10, 30), text_style)
                    .draw(display)
                    .ok();
                draw_fps_text(display, app_state.fps).ok();
                display.flush().ok();
                app_state.needs_redraw = false;
            } else {
                draw_fps_overlay(display, app_state.fps).ok();
            }
        }
    }
}

/// Draw FPS text overlay
fn draw_fps_text(
    display: &mut crate::display::Sh8601Driver,
    fps: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    use core::fmt::Write;
    let mut fps_str = heapless::String::<16>::new();
    write!(fps_str, "FPS: {:.1}", fps).ok();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
    Text::new(&fps_str, Point::new(10, 10), text_style).draw(display)?;
    Ok(())
}

/// Draw FPS overlay (small box with FPS counter)
fn draw_fps_overlay(
    display: &mut crate::display::Sh8601Driver,
    fps: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Draw semi-transparent background box for FPS
    Rectangle::new(Point::new(5, 2), Size::new(70, 15))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    draw_fps_text(display, fps)?;
    display.flush()?;
    Ok(())
}

/// Draw the initial welcome screen
fn draw_welcome_screen(
    display: &mut crate::display::Sh8601Driver,
    fps: f32,
) -> Result<(), Box<dyn std::error::Error>> {
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

    draw_fps_text(display, fps)?;
    display.flush()?;
    Ok(())
}
