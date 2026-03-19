//! Sprite and animation subsystem.

pub mod spr;

pub use spr::{Sprite, SprError};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use std::time::Instant;

/// Manages sprite animation timing automatically.
///
/// Load a sprite, call `play()`, then `tick()` each frame.
/// The player advances frames based on each frame's delay.
pub struct AnimationPlayer {
    sprite: Sprite,
    last_advance: Instant,
    playing: bool,
    looping: bool,
}

impl AnimationPlayer {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            last_advance: Instant::now(),
            playing: false,
            looping: true,
        }
    }

    /// Start playing the animation.
    pub fn play(&mut self) {
        self.playing = true;
        self.last_advance = Instant::now();
    }

    /// Pause the animation.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Reset to frame 0.
    pub fn reset(&mut self) {
        let _ = self.sprite.seek_frame(0);
        self.last_advance = Instant::now();
    }

    /// Set whether the animation loops.
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Whether the animation is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Call once per frame. Advances the sprite if enough time has elapsed.
    pub fn tick(&mut self) {
        if !self.playing {
            return;
        }
        let delay = self.sprite.current_delay_ms() as u128;
        if self.last_advance.elapsed().as_millis() >= delay {
            if self.sprite.next_frame().is_err() {
                if self.looping {
                    let _ = self.sprite.seek_frame(0);
                } else {
                    self.playing = false;
                }
            }
            self.last_advance = Instant::now();
        }
    }

    /// Draw the current frame to a DrawTarget.
    pub fn draw<D: DrawTarget<Color = Rgb888>>(&self, display: &mut D, x: i32, y: i32) {
        self.sprite.draw(display, x, y);
    }

    /// Draw with background color for transparent pixels.
    pub fn draw_with_bg<D: DrawTarget<Color = Rgb888>>(
        &self,
        display: &mut D,
        x: i32,
        y: i32,
        bg: Rgb888,
    ) {
        self.sprite.draw_with_bg(display, x, y, bg);
    }

    /// Access the underlying sprite.
    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    /// Access the underlying sprite mutably.
    pub fn sprite_mut(&mut self) -> &mut Sprite {
        &mut self.sprite
    }
}
