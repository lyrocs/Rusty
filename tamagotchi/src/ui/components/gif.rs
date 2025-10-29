/// GIF rendering utilities
///
/// Provides functions for rendering animated GIFs for monsters and heroes.

use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb888,
    prelude::*,
};
use tinygif::Gif;

use crate::core::GameState;
use crate::combat::{MonsterAnimation, MonsterAttackedAnimation, get_monster_attacked_gif};

/// Draw monster GIF animation at center position
pub fn draw_monster_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = game_state.monster_animation.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.monster_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Draw monster idle GIF (0.gif) on map page
pub fn draw_map_monster_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Get idle animation GIF (0.gif) for the monster
    let gif_data = MonsterAnimation::Idle.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame from map animation frame counter
    // Count total frames first to wrap properly
    let total_frames = gif.frames().count();
    if total_frames == 0 {
        return Ok(());
    }

    let frame_index = game_state.map_monster_animation_frame % total_frames;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Draw hero GIF animation
pub fn draw_hero_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = game_state.hero_animation.gif_data();
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse hero GIF");

    // Get GIF dimensions
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to align by bottom
    // center_position is treated as the bottom-center anchor point
    // This ensures smooth transitions between different-sized animations (36.gif vs 84.gif)
    let top_left = Point::new(
        center_position.x - (gif_width / 2), // Center horizontally
        center_position.y - gif_height,      // Align by bottom
    );

    // Get current frame
    let frame_index = game_state.hero_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Draw monster attacked GIF animation (24.gif when hero attacks)
pub fn draw_monster_attacked_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    if game_state.monster_attacked_animation == MonsterAttackedAnimation::Normal {
        // No attacked animation, draw normal monster
        return draw_monster_gif(display, game_state, center_position, monster_name);
    }

    // Draw attacked animation (24.gif) for specific monster
    let gif_data = get_monster_attacked_gif(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse monster attacked GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.monster_attacked_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}
