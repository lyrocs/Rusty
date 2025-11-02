/// Debug page - displays hero attacking animation in a loop
///
/// This page is used for testing and debugging hero attack animations.

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};

use crate::core::GameState;
use crate::combat::HeroAnimation;
use super::super::helpers::draw_hero_gif_with_animation;

/// Draw the Debug page with hero attacking animation
pub fn draw_debug_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Draw hero attacking animation centered on screen
    // Screen is 368x448, so center position would be around 184x224
    // NOTE: No global clear - draw_hero_gif_with_animation will clear only its zone
    let center_position = Point::new(184, 280);

    // Always display the attacking animation
    // This function will clear only the GIF zone and draw the frame
    draw_hero_gif_with_animation(
        display,
        game_state,
        center_position,
        HeroAnimation::Attacking,
    )?;

    Ok(())
}
