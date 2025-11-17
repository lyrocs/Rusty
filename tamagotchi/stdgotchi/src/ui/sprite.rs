//! Sprite System
//!
//! Reusable sprite components for backgrounds and animated entities.

use std::time::{Duration, Instant};
use crate::display::{GifPlayer, Sh8601Driver, StaticImage};

/// Global animation speed multiplier (higher = faster)
/// 1.0 = normal speed, 2.0 = 2x speed, 0.5 = half speed
/// Adjust this value to make all animations faster or slower
const ANIMATION_SPEED_MULTIPLIER: f32 = 2.0;

/// Static background sprite
pub struct Background {
    image: StaticImage,
    position: (i32, i32),
}

impl Background {
    /// Create a new background from GIF data
    pub fn new(gif_data: &[u8], position: (i32, i32)) -> Result<Self, Box<dyn std::error::Error>> {
        let image = StaticImage::new(gif_data)?;
        Ok(Self { image, position })
    }

    /// Draw the background
    pub fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn std::error::Error>> {
        self.image.render(display, self.position)
    }

    /// Draw a specific region of the background
    ///
    /// # Arguments
    /// * `display` - Display driver instance
    /// * `region` - (x, y, width, height) region to render in screen coordinates
    pub fn draw_region(
        &self,
        display: &mut Sh8601Driver,
        region: (i32, i32, u32, u32),
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.image.render_region(display, self.position, region)
    }
}

/// Animated sprite (monster, hero, etc.)
pub struct AnimatedSprite {
    player: GifPlayer,
    position: (i32, i32),
    current_frame: usize,
    frame_count: usize,
    last_frame_time: Instant,
    frame_delay: Duration,
    loops: Option<u32>, // None = infinite, Some(n) = loop n times
    current_loop: u32,
    flip_horizontal: bool, // Mirror the sprite horizontally
}

impl AnimatedSprite {
    /// Create a new animated sprite from GIF data
    ///
    /// # Arguments
    /// * `gif_data` - Raw GIF file bytes
    /// * `position` - (x, y) position on screen
    /// * `frame_delay` - Time between frames (e.g., Duration::from_millis(50))
    /// * `loops` - Number of loops (None for infinite)
    pub fn new(
        gif_data: &[u8],
        position: (i32, i32),
        frame_delay: Duration,
        loops: Option<u32>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let player = GifPlayer::new(gif_data)?;
        let frame_count = player.frame_count();

        Ok(Self {
            player,
            position,
            current_frame: 0,
            frame_count,
            last_frame_time: Instant::now(),
            frame_delay,
            loops,
            current_loop: 0,
            flip_horizontal: false,
        })
    }

    /// Create a new animated sprite with horizontal flip (for facing opposite direction)
    pub fn new_flipped(
        gif_data: &[u8],
        position: (i32, i32),
        frame_delay: Duration,
        loops: Option<u32>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sprite = Self::new(gif_data, position, frame_delay, loops)?;
        sprite.flip_horizontal = true;
        Ok(sprite)
    }

    /// Set horizontal flip state
    pub fn set_flip_horizontal(&mut self, flip: bool) {
        self.flip_horizontal = flip;
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        self.player.dimensions()
    }

    /// Get bounding box (x, y, width, height) for this sprite
    pub fn bounds(&self) -> (i32, i32, u32, u32) {
        let (width, height) = self.player.dimensions();
        (self.position.0, self.position.1, width as u32, height as u32)
    }

    /// Update animation (call once per frame)
    /// Returns true if animation is still playing, false if complete
    pub fn update(&mut self) -> bool {
        // Apply global animation speed multiplier
        // Higher multiplier = shorter delay = faster animation
        let effective_delay = self.frame_delay.div_f32(ANIMATION_SPEED_MULTIPLIER);

        // Check if enough time has passed for next frame
        if self.last_frame_time.elapsed() >= effective_delay {
            self.current_frame += 1;
            self.last_frame_time = Instant::now();

            // Check if we completed a loop
            if self.current_frame >= self.frame_count {
                self.current_frame = 0;
                self.current_loop += 1;

                // Check if we should stop
                if let Some(max_loops) = self.loops {
                    if self.current_loop >= max_loops {
                        return false; // Animation complete
                    }
                }
            }
        }

        true // Still playing
    }

    /// Draw the current frame
    pub fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn std::error::Error>> {
        self.player.render_frame_with_flip(display, self.current_frame, Some(self.position), self.flip_horizontal)
    }

    /// Reset animation to beginning
    pub fn reset_animation(&mut self) {
        self.current_frame = 0;
        self.current_loop = 0;
        self.last_frame_time = Instant::now();
    }

    /// Check if animation has completed all loops
    pub fn is_animation_complete(&self) -> bool {
        if let Some(max_loops) = self.loops {
            self.current_loop >= max_loops
        } else {
            false // Infinite loop never completes
        }
    }

    /// Get current frame index
    pub fn current_frame_index(&self) -> usize {
        self.current_frame % self.frame_count
    }

    /// Get total frame count
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }
}
