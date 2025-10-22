use bevy_ecs::prelude::*;
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb888,
    image::{GetPixel, Image},
    primitives::Rectangle,
};
use tinygif::Gif;

/// Resource for tracking GIF animation state
#[derive(Resource)]
pub struct GifResource {
    pub position: Point,          // Current GIF position
    pub previous_position: Point, // Previous position for cleanup
    pub frame_index: usize,       // Current frame index
    pub first_render: bool,       // Track if GIF has been rendered at least once
}

impl Default for GifResource {
    fn default() -> Self {
        Self {
            position: Point::new(160, 200), // Center of screen roughly
            previous_position: Point::new(160, 200),
            frame_index: 0,
            first_render: true, // Force initial render
        }
    }
}

/// Renders a GIF animation with optimized background restoration
///
/// # Parameters
/// - `display`: The display target to draw on
/// - `background`: The background image to restore when clearing frames
/// - `gif_data`: The GIF data bytes
/// - `gif_res`: Resource tracking GIF position and frame state
/// - `target_frame_index`: The frame to display (typically generation % total_frames)
/// - `gif_width`: Width of the GIF in pixels
/// - `gif_height`: Height of the GIF in pixels
///
/// # Returns
/// - `true` if rendering occurred, `false` if no rendering was needed
pub fn render_gif_optimized<D, I>(
    display: &mut D,
    background: &I,
    gif_data: &[u8],
    gif_res: &mut GifResource,
    target_frame_index: usize,
    gif_width: u32,
    gif_height: u32,
) -> bool
where
    D: DrawTarget<Color = Rgb888>,
    I: GetPixel<Color = Rgb888>,
{
    // Check if position or frame changed or if it's the first render
    let position_changed = gif_res.position != gif_res.previous_position;
    let frame_changed = gif_res.frame_index != target_frame_index;
    let needs_render = position_changed || frame_changed || gif_res.first_render;

    if !needs_render {
        return false;
    }

    // Step 1: Clear the old GIF position by restoring background (only if position changed)
    if position_changed {
        let old_gif_area =
            Rectangle::new(gif_res.previous_position, Size::new(gif_width, gif_height));

        for pixel in old_gif_area.points() {
            if let Some(color) = background.pixel(pixel) {
                embedded_graphics::Pixel(pixel, color).draw(display).ok();
            }
        }

        // Also restore background at new position when moving
        let new_gif_area = Rectangle::new(gif_res.position, Size::new(gif_width, gif_height));

        for pixel in new_gif_area.points() {
            if let Some(color) = background.pixel(pixel) {
                embedded_graphics::Pixel(pixel, color).draw(display).ok();
            }
        }
    } else if frame_changed {
        // Step 2: For frame changes only, restore background to clear previous frame
        let gif_area = Rectangle::new(gif_res.position, Size::new(gif_width, gif_height));

        for pixel in gif_area.points() {
            if let Some(color) = background.pixel(pixel) {
                embedded_graphics::Pixel(pixel, color).draw(display).ok();
            }
        }
    }

    // Step 3: Draw the target GIF frame at the current position
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");
    let mut current_index = 0;
    for frame in gif.frames() {
        if current_index == target_frame_index {
            Image::new(&frame, gif_res.position).draw(display).ok();
            break;
        }
        current_index += 1;
    }

    // Update the GIF state
    gif_res.frame_index = target_frame_index;
    gif_res.first_render = false;

    true
}
