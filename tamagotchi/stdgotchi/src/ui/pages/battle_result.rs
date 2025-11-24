//! Battle Result Page
//!
//! Displays battle results with EXP gains and HP regeneration

use crate::display::Sh8601Driver;
use crate::game::{GameData, Rustymon};
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

/// Result display entity with sprite and stats
struct ResultEntity {
    sprite: AnimatedSprite,
    rustymon: Rustymon,
    initial_hp: u32,
    initial_exp: u32,
    exp_gained: u32,
}

impl ResultEntity {
    fn new(
        rustymon: Rustymon,
        exp_gained: u32,
        position: (i32, i32),
    ) -> Result<Self, Box<dyn Error>> {
        // Load idle sprite
        let (idle, _, _, _) = load_enemy_sprites_embedded(rustymon.species_id)
            .ok_or("Failed to load rustymon sprites")?;

        let frame_delay = Duration::from_millis(100);
        let mut sprite = AnimatedSprite::new(&idle, position, frame_delay, None)?;
        sprite.set_flip_horizontal(true); // Face left like in battle
        sprite.set_center_positioned(true);

        log::info!("📊 Creating ResultEntity: {} - initial_exp={}, exp_gained={}, level={}",
            rustymon.name, rustymon.exp, exp_gained, rustymon.level);

        Ok(Self {
            initial_hp: rustymon.current_hp,
            initial_exp: rustymon.exp,
            sprite,
            rustymon,
            exp_gained,
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

/// Battle Result Page
pub struct BattleResultPage {
    background_color: Rgb888,
    entities: Vec<ResultEntity>,
    game_data: GameData,
    last_regen: Instant,
    start_time: Instant,
}

impl BattleResultPage {
    /// Create a new battle result page
    pub fn new(
        rustymon_list: Vec<Rustymon>,
        exp_rewards: Vec<u32>,
        game_data: GameData,
    ) -> Result<Self, Box<dyn Error>> {
        let mut entities = Vec::new();

        // Position rustymon vertically centered
        let positions = [
            (100, 120),  // Top
            (100, 220),  // Middle
            (100, 320),  // Bottom
        ];

        for (i, (rustymon, &exp_gained)) in rustymon_list.iter().zip(exp_rewards.iter()).enumerate() {
            if i >= 3 {
                break;
            }

            let entity = ResultEntity::new(rustymon.clone(), exp_gained, positions[i])?;
            entities.push(entity);

            log::info!("Result entity {}: {} gained {} exp", i, rustymon.name, exp_gained);
        }

        Ok(Self {
            background_color: Rgb888::new(20, 30, 40),
            entities,
            game_data,
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

    /// Draw EXP bar with base exp (yellow) and gained exp (green overlay)
    fn draw_exp_bar(
        display: &mut Sh8601Driver,
        label: &str,
        current_exp: u32,
        exp_gained: u32,
        level: u32,
        game_data: &GameData,
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

        // Calculate exp to next level
        // get_exp_for_level(N) returns the TOTAL exp needed to reach level N+1
        // For example: level 1 needs 548 total exp to become level 2
        // So for a level 1 rustymon:
        //   - Start of level 1: get_exp_for_level(0) = 0
        //   - End of level 1 / Start of level 2: get_exp_for_level(1) = 548
        let exp_for_current_level_start = if level > 1 {
            game_data.get_exp_for_level(level - 1)
        } else {
            0 // Level 1 starts at 0 EXP
        };
        let exp_for_next_level = game_data.get_exp_for_level(level);

        let exp_needed = if exp_for_next_level > exp_for_current_level_start {
            exp_for_next_level - exp_for_current_level_start
        } else {
            1 // Avoid division by zero
        };

        let exp_progress = if current_exp >= exp_for_current_level_start {
            (current_exp - exp_for_current_level_start).min(exp_needed)
        } else {
            0
        };

        // Base exp bar (yellow) - initial progress before battle
        let base_width = if exp_needed > 0 {
            ((exp_progress as f32 / exp_needed as f32) * bar_width as f32) as u32
        } else {
            0
        };

        // Gained exp overlay (green on top of yellow)
        let total_progress = (exp_progress + exp_gained).min(exp_needed);
        let total_width = if exp_needed > 0 {
            ((total_progress as f32 / exp_needed as f32) * bar_width as f32) as u32
        } else {
            0
        };

        // Debug logging on first draw
        static mut FIRST_EXP_DRAW: bool = true;
        unsafe {
            if FIRST_EXP_DRAW {
                log::info!("🔍 EXP Bar Debug:");
                log::info!("  Level: {}", level);
                log::info!("  Current EXP: {}", current_exp);
                log::info!("  EXP gained: {}", exp_gained);
                log::info!("  exp_for_current_level_start: {}", exp_for_current_level_start);
                log::info!("  exp_for_next_level: {}", exp_for_next_level);
                log::info!("  exp_needed: {}", exp_needed);
                log::info!("  exp_progress: {}", exp_progress);
                log::info!("  base_width (yellow): {} px", base_width);
                log::info!("  total_width (yellow+green): {} px", total_width);
                log::info!("  green_width: {} px", total_width.saturating_sub(base_width));
                FIRST_EXP_DRAW = false;
            }
        }

        // Background bar (dark)
        Rectangle::new(
            Point::new(x, bar_y),
            Size::new(bar_width as u32, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        if base_width > 0 {
            Rectangle::new(
                Point::new(x, bar_y),
                Size::new(base_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(200, 200, 0)))
            .draw(display)?;
        }

        if total_width > base_width {
            Rectangle::new(
                Point::new(x + base_width as i32, bar_y),
                Size::new(total_width - base_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 200, 0)))
            .draw(display)?;
        }

        // EXP text on bar
        use core::fmt::Write;
        let mut exp_text = heapless::String::<32>::new();
        write!(exp_text, "+{} EXP", exp_gained).ok();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(0, 255, 0));
        let text_x = x + (bar_width / 2) - (exp_text.len() as i32 * 3);
        Text::new(&exp_text, Point::new(text_x, bar_y + 9), text_style).draw(display)?;

        Ok(())
    }

    /// Get updated rustymon with HP regeneration and EXP gains
    pub fn get_updated_rustymon(&self) -> Vec<Rustymon> {
        self.entities.iter().map(|e| {
            let mut rustymon = e.rustymon.clone();
            // EXP is already added to rustymon in the entity
            rustymon.exp += e.exp_gained;
            // Level up if needed
            while rustymon.exp >= self.game_data.get_exp_for_level(rustymon.level + 1) {
                rustymon.level += 1;
                log::info!("{} leveled up to {}!", rustymon.name, rustymon.level);
            }
            rustymon
        }).collect()
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

impl Page for BattleResultPage {
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
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 215, 0));
        Text::new("VICTORY!", Point::new(130, 30), title_style).draw(display)?;

        // Draw each rustymon with their stats
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

            // Draw EXP bar
            Self::draw_exp_bar(
                display,
                "EXP:",
                entity.initial_exp,
                entity.exp_gained,
                entity.rustymon.level,
                &self.game_data,
                180,
                base_y + 55,
            )?;
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
