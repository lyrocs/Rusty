//! Rendering system
//!
//! Handles display updates based on app state.

use bevy_ecs::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};
use log::info;

use crate::display::GifPlayer;
use crate::ecs::resources::{AppMode, AppState, DisplayResource, GifAnimationState};

/// Embedded GIF animation data
const GIF_DATA: &[u8] = include_bytes!("../../assets/80.gif");
const MAP_GIF_DATA: &[u8] = include_bytes!("../../assets/images/map/1.gif");
const HORNET_GIF_DATA: &[u8] = include_bytes!("../../assets/images/hornet/38.gif");

/// System to render the display
pub fn render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut app_state: ResMut<AppState>,
    gif_animation: Option<NonSendMut<GifAnimationState>>,
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
            // Frame-by-frame animation rendering
            if let Some(mut anim) = gif_animation {
                // Check if it's time for next frame
                if anim.last_frame_time.elapsed() >= anim.frame_delay {
                    // Render one frame - clear screen first
                    display.clear(Rgb888::BLACK).ok();

                    // Extract positions and frame index before mutable borrows
                    let map_pos = anim.map_pos;
                    let hornet_pos = anim.hornet_pos;
                    let hornet_frame = anim.hornet_frame_index;

                    // Layer 1: Map background (static image)
                    anim.map_image.render(display, map_pos).ok();

                    // Layer 2: Hornet foreground (animated)
                    anim.hornet_player.render_frame(display, hornet_frame, Some(hornet_pos)).ok();

                    // Use system FPS for display
                    draw_fps_overlay(display, app_state.fps).ok();

                    display.flush().ok();

                    // Update animation state
                    anim.current_frame += 1;
                    anim.hornet_frame_index = (anim.hornet_frame_index + 1) % anim.hornet_frame_count;
                    anim.last_frame_time = std::time::Instant::now();

                    // Check if animation is complete
                    if anim.current_frame >= anim.total_frames {
                        info!("GIF animation completed ({} frames)", anim.total_frames);

                        // Return to welcome mode
                        app_state.current_mode = AppMode::Welcome;
                        app_state.needs_redraw = true;
                        // Note: GifAnimationState will be removed by cleanup system
                    }
                }
                // If not time for next frame yet, skip (non-blocking!)
            }
        }
        AppMode::ButtonFeedback => {
            if app_state.needs_redraw {
                display.clear(Rgb888::new(50, 0, 50)).ok();
                let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                Text::new("Button pressed", Point::new(10, 30), text_style).draw(display).ok();
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
fn draw_fps_text(display: &mut crate::display::Sh8601Driver, fps: f32) -> Result<(), Box<dyn std::error::Error>> {
    use core::fmt::Write;
    let mut fps_str = heapless::String::<16>::new();
    write!(fps_str, "FPS: {:.1}", fps).ok();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
    Text::new(&fps_str, Point::new(10, 10), text_style).draw(display)?;
    Ok(())
}

/// Draw FPS overlay (small box with FPS counter)
fn draw_fps_overlay(display: &mut crate::display::Sh8601Driver, fps: f32) -> Result<(), Box<dyn std::error::Error>> {
    // Draw semi-transparent background box for FPS
    Rectangle::new(Point::new(5, 2), Size::new(70, 15))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    draw_fps_text(display, fps)?;
    display.flush()?;
    Ok(())
}

/// Draw the initial welcome screen
fn draw_welcome_screen(display: &mut crate::display::Sh8601Driver, fps: f32) -> Result<(), Box<dyn std::error::Error>> {
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

/// Play multiple GIF animations (background + foreground)
fn play_multi_gif_animation(display: &mut crate::display::Sh8601Driver) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading multi-GIF animation...");

    // Load background map GIF
    let mut map_player = GifPlayer::new(MAP_GIF_DATA)?;
    let (map_width, map_height) = map_player.dimensions();
    info!("Map GIF: {}x{}, frames: {}", map_width, map_height, map_player.frame_count());

    // Load foreground hornet GIF
    let mut hornet_player = GifPlayer::new(HORNET_GIF_DATA)?;
    let (hornet_width, hornet_height) = hornet_player.dimensions();
    info!("Hornet GIF: {}x{}, frames: {}", hornet_width, hornet_height, hornet_player.frame_count());

    let display_size = display.size();

    // Background: render at top-left (0, 0)
    let map_pos = (0, 0);

    // Foreground hornet: centered
    let hornet_x = (display_size.width as i32 - hornet_width as i32) / 2;
    let hornet_y = (display_size.height as i32 - hornet_height as i32) / 2;
    let hornet_pos = (hornet_x, hornet_y);

    info!("Map at {:?}, Hornet at {:?}", map_pos, hornet_pos);

    // Synchronize frame counts
    let max_frames = map_player.frame_count().max(hornet_player.frame_count());
    let total_frames = max_frames * 3; // 3 loops

    let start_time = std::time::Instant::now();
    let mut frame_count = 0;

    for _ in 0..total_frames {
        // Clear and render background map
        display.clear(Rgb888::BLACK)?;
        let _map_delay = map_player.next_frame(display, Some(map_pos))?;

        // Overlay foreground hornet
        let hornet_delay = hornet_player.next_frame(display, Some(hornet_pos))?;

        // Draw FPS
        let elapsed = start_time.elapsed().as_secs_f32();
        let fps = if elapsed > 0.0 { frame_count as f32 / elapsed } else { 0.0 };
        draw_fps_overlay(display, fps).ok();

        display.flush()?;

        // Use hornet delay for frame timing
        std::thread::sleep(hornet_delay);
        frame_count += 1;
    }

    let total_time = start_time.elapsed();
    let avg_fps = total_frames as f32 / total_time.as_secs_f32();
    info!("GIF animation completed. Average FPS: {:.2}", avg_fps);

    Ok(())
}

/// Play the GIF animation at a fixed position (old single GIF)
#[allow(dead_code)]
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
