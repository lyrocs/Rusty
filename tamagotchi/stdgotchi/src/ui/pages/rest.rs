//! Rest Page
//!
//! Displays team Rustymon resting with HP regeneration over time

use crate::display::Sh8601Driver;
use crate::game::Rustymon;
use crate::ui::page::Page;
use crate::ui::sprite::AnimatedSprite;
use crate::assets::battle::load_enemy_sprites_embedded;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};

/// HP regeneration rate: 3 HP every 2 seconds
const HP_REGEN_INTERVAL: Duration = Duration::from_secs(2);
const HP_REGEN_AMOUNT: u32 = 3;

/// Rest entity with sprite and HP tracking
struct RestEntity {
    sprite: AnimatedSprite,
    rustymon: Rustymon,
    initial_hp: u32,
}

impl RestEntity {
    fn new(rustymon: Rustymon, position: (i32, i32)) -> Result<Self, Box<dyn Error>> {
        // Load idle sprite
        let (idle, _, _, _) = load_enemy_sprites_embedded(rustymon.species_id)
            .ok_or("Failed to load rustymon sprites")?;

        let frame_delay = Duration::from_millis(100);
        let mut sprite = AnimatedSprite::new(&idle, position, frame_delay, None)?;
        sprite.set_flip_horizontal(true);
        sprite.set_center_positioned(true);

        Ok(Self {
            initial_hp: rustymon.current_hp,
            sprite,
            rustymon,
        })
    }

    fn update(&mut self) {
        self.sprite.update();
    }

    fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        self.sprite.draw(display)?;
        Ok(())
    }
}

/// Rest Page - Team Rustymon resting with HP regeneration
pub struct RestPage {
    background_color: Rgb888,
    entities: Vec<RestEntity>,
    last_regen: Instant,
    start_time: Instant,
}

impl RestPage {
    /// Create a new rest page with team rustymon
    pub fn new(rustymon_list: Vec<Rustymon>) -> Result<Self, Box<dyn Error>> {
        let mut entities = Vec::new();

        // Position rustymon vertically centered
        let positions = [
            (100, 120),  // Top
            (100, 220),  // Middle
            (100, 320),  // Bottom
        ];

        for (i, rustymon) in rustymon_list.iter().enumerate() {
            if i >= 3 {
                break;
            }

            let entity = RestEntity::new(rustymon.clone(), positions[i])?;
            entities.push(entity);

            log::info!("Rest entity {}: {} with HP {}/{}", i, rustymon.name, rustymon.current_hp, rustymon.max_hp);
        }

        Ok(Self {
            background_color: Rgb888::new(25, 35, 45),
            entities,
            last_regen: Instant::now(),
            start_time: Instant::now(),
        })
    }

    /// Process HP regeneration
    fn process_regen(&mut self) {
        if self.last_regen.elapsed() >= HP_REGEN_INTERVAL {
            self.last_regen = Instant::now();

            for entity in &mut self.entities {
                if entity.rustymon.current_hp < entity.rustymon.max_hp {
                    let regen = HP_REGEN_AMOUNT.min(entity.rustymon.max_hp - entity.rustymon.current_hp);
                    entity.rustymon.current_hp += regen;
                    log::debug!("{} regenerated {} HP ({}/{})",
                        entity.rustymon.name, regen, entity.rustymon.current_hp, entity.rustymon.max_hp);
                }
            }
        }
    }

    /// Check if all HP is restored
    fn is_hp_full(&self) -> bool {
        self.entities.iter().all(|e| e.rustymon.current_hp >= e.rustymon.max_hp)
    }

    /// Draw HP bar with current/max HP
    fn draw_hp_bar(
        display: &mut Sh8601Driver,
        label: &str,
        current_hp: u32,
        max_hp: u32,
        x: i32,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        // Label
        let label_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new(label, Point::new(x, y), label_style).draw(display)?;

        // Bar dimensions
        let bar_width = 150;
        let bar_height = 12;
        let bar_y = y + 5;

        // Background bar (dark)
        Rectangle::new(
            Point::new(x, bar_y),
            Size::new(bar_width as u32, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // HP bar (colored)
        let hp_percentage = (current_hp as f32 / max_hp as f32) * 100.0;
        let hp_color = if hp_percentage > 60.0 {
            Rgb888::new(0, 200, 0) // Green
        } else if hp_percentage > 30.0 {
            Rgb888::new(200, 200, 0) // Yellow
        } else {
            Rgb888::new(200, 0, 0) // Red
        };

        let filled_width = ((current_hp as f32 / max_hp as f32) * bar_width as f32) as u32;
        if filled_width > 0 {
            Rectangle::new(
                Point::new(x, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;
        }

        // HP text on bar
        use core::fmt::Write;
        let mut hp_text = heapless::String::<32>::new();
        write!(hp_text, "{}/{}", current_hp, max_hp).ok();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let text_x = x + (bar_width / 2) - (hp_text.len() as i32 * 3);
        Text::new(&hp_text, Point::new(text_x, bar_y + 9), text_style).draw(display)?;

        Ok(())
    }

    /// Get updated rustymon with HP regeneration
    pub fn get_updated_rustymon(&self) -> Vec<Rustymon> {
        self.entities.iter().map(|e| e.rustymon.clone()).collect()
    }

    /// Check if user touched the continue area (only active when HP is full)
    pub fn handle_touch(&self, x: i32, y: i32) -> bool {
        // Only allow continue when HP is fully restored
        if !self.is_hp_full() {
            return false;
        }

        // Continue button area at bottom center
        let button_x = 100;
        let button_y = 420;
        let button_w = 168;
        let button_h = 40;

        x >= button_x && x <= button_x + button_w && y >= button_y && y <= button_y + button_h
    }
}

impl Page for RestPage {
    fn update(&mut self) -> bool {
        // Update sprites
        for entity in &mut self.entities {
            entity.update();
        }

        // Process HP regeneration
        self.process_regen();

        // Always keep page active - user must manually continue
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 200, 255));
        Text::new("💤 Resting 💤", Point::new(100, 30), title_style).draw(display)?;

        // Draw each rustymon with their HP
        for (i, entity) in self.entities.iter().enumerate() {
            let base_y = 60 + (i as i32 * 100);

            // Draw sprite
            entity.draw(display)?;

            // Draw name
            let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            Text::new(&entity.rustymon.name, Point::new(180, base_y + 20), name_style).draw(display)?;

            // Draw HP bar
            Self::draw_hp_bar(
                display,
                "HP:",
                entity.rustymon.current_hp,
                entity.rustymon.max_hp,
                180,
                base_y + 30,
            )?;

            // Show regen rate
            let info_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 150, 150));
            Text::new("+3 HP / 2s", Point::new(180, base_y + 55), info_style).draw(display)?;
        }

        // Draw continue button at bottom
        let button_x = 100;
        let button_y = 420;
        let button_w = 168;
        let button_h = 40;

        let hp_full = self.is_hp_full();
        let button_color = if hp_full {
            Rgb888::new(0, 150, 0) // Green when ready
        } else {
            Rgb888::new(100, 100, 100) // Gray while regenerating
        };

        Rectangle::new(
            Point::new(button_x, button_y),
            Size::new(button_w, button_h),
        )
        .into_styled(PrimitiveStyle::with_fill(button_color))
        .draw(display)?;

        let button_text = if hp_full {
            "Continue"
        } else {
            "Regenerating..."
        };
        let button_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let text_x = button_x + (button_w as i32 / 2) - (button_text.len() as i32 * 5);
        Text::new(button_text, Point::new(text_x, button_y + 25), button_style).draw(display)?;

        display.flush()?;
        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No-op
    }

    fn needs_full_redraw(&self) -> bool {
        true // Always redraw for animations
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
