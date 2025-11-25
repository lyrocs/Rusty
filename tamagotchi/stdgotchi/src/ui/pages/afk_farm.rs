//! AFK Farm Page
//!
//! Passive farming mode where heroes gain EXP over time without taking damage
//! EXP gain rate is based on hero stats vs enemy stats

use crate::display::Sh8601Driver;
use crate::game::{Enemy, GameData, Rustymon};
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

/// EXP update interval: gain EXP every 3 seconds
const EXP_GAIN_INTERVAL: Duration = Duration::from_secs(3);

/// AFK farming entity with animated sprite
struct AfkEntity {
    sprite: AnimatedSprite,
    rustymon: Rustymon,
    initial_exp: u32,
}

impl AfkEntity {
    fn new(rustymon: Rustymon, position: (i32, i32), flip: bool) -> Result<Self, Box<dyn Error>> {
        // Load idle sprite
        let (idle, _, _, _) = load_enemy_sprites_embedded(rustymon.species_id)
            .ok_or("Failed to load rustymon sprites")?;

        let frame_delay = Duration::from_millis(100);
        let mut sprite = AnimatedSprite::new(&idle, position, frame_delay, None)?;
        sprite.set_flip_horizontal(flip);
        sprite.set_center_positioned(true);

        Ok(Self {
            initial_exp: rustymon.exp,
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

/// Enemy display entity (no stats needed, just for show)
struct AfkEnemyEntity {
    sprite: AnimatedSprite,
    name: String,
}

impl AfkEnemyEntity {
    fn new(enemy_id: u32, name: String, position: (i32, i32)) -> Result<Self, Box<dyn Error>> {
        let (idle, _, _, _) = load_enemy_sprites_embedded(enemy_id)
            .ok_or("Failed to load enemy sprites")?;

        let frame_delay = Duration::from_millis(100);
        let mut sprite = AnimatedSprite::new(&idle, position, frame_delay, None)?;
        sprite.set_center_positioned(true);

        Ok(Self {
            sprite,
            name,
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

/// AFK Farm Page - Passive EXP farming
pub struct AfkFarmPage {
    background_color: Rgb888,
    hero_entities: Vec<AfkEntity>,
    enemy_entities: Vec<AfkEnemyEntity>,
    last_exp_gain: Instant,
    start_time: Instant,
    exp_per_cycle: u32,
    total_exp_gained: u32,
    game_data: GameData,
}

impl AfkFarmPage {
    /// Create a new AFK farm page
    /// EXP per cycle is calculated based on hero stats vs enemy difficulty
    pub fn new(
        heroes: Vec<Rustymon>,
        enemy_ids: &[u32],
        game_data: GameData,
    ) -> Result<Self, Box<dyn Error>> {
        let mut hero_entities = Vec::new();
        let mut enemy_entities = Vec::new();

        // Hero positions (right side, facing left)
        let hero_positions = [
            (200, 120),  // Top
            (200, 220),  // Middle
            (200, 320),  // Bottom
        ];

        // Enemy positions (left side, facing right)
        let enemy_positions = [
            (60, 120),   // Top
            (60, 220),   // Middle
            (60, 320),   // Bottom
        ];

        // Setup heroes
        for (i, hero) in heroes.iter().enumerate() {
            if i >= 3 {
                break;
            }
            let entity = AfkEntity::new(hero.clone(), hero_positions[i], true)?;
            hero_entities.push(entity);
            log::info!("AFK farm hero {}: {} (Level {})", i, hero.name, hero.level);
        }

        // Setup enemies (for display only)
        let enemy_count = hero_entities.len().min(enemy_ids.len());
        for i in 0..enemy_count {
            let enemy_id = enemy_ids[i % enemy_ids.len()];
            if let Some(enemy_data) = game_data.get_enemy(enemy_id) {
                let entity = AfkEnemyEntity::new(enemy_id, enemy_data.name.clone(), enemy_positions[i])?;
                enemy_entities.push(entity);
                log::info!("AFK farm enemy {}: {}", i, enemy_data.name);
            }
        }

        // Calculate EXP per cycle based on hero and enemy stats
        let exp_per_cycle = Self::calculate_exp_per_cycle(&heroes, enemy_ids, &game_data);
        log::info!("💰 AFK farming EXP per cycle ({} seconds): {} EXP",
            EXP_GAIN_INTERVAL.as_secs(), exp_per_cycle);

        Ok(Self {
            background_color: Rgb888::new(15, 25, 35),
            hero_entities,
            enemy_entities,
            last_exp_gain: Instant::now(),
            start_time: Instant::now(),
            exp_per_cycle,
            total_exp_gained: 0,
            game_data,
        })
    }

    /// Calculate EXP gain per cycle based on hero stats vs enemy difficulty
    /// Formula: (Average hero level × Enemy level × Enemy base_exp) / 50 per cycle
    fn calculate_exp_per_cycle(heroes: &[Rustymon], enemy_ids: &[u32], game_data: &GameData) -> u32 {
        if heroes.is_empty() || enemy_ids.is_empty() {
            return 5; // Minimum EXP
        }

        // Calculate average hero power (level + stats)
        let mut total_hero_power = 0;
        for hero in heroes {
            let power = hero.level + (hero.atk / 10) + (hero.max_hp / 50);
            total_hero_power += power;
        }
        let avg_hero_power = total_hero_power / heroes.len() as u32;

        // Calculate average enemy difficulty
        let mut total_enemy_value = 0;
        for &enemy_id in enemy_ids.iter().take(heroes.len()) {
            if let Some(enemy_data) = game_data.get_enemy(enemy_id) {
                let enemy_value = enemy_data.level + (enemy_data.base_exp / 10) as u32;
                total_enemy_value += enemy_value;
            }
        }
        let avg_enemy_value = if total_enemy_value > 0 {
            total_enemy_value / enemy_ids.len().min(heroes.len()) as u32
        } else {
            10
        };

        // EXP per cycle = (hero_power × enemy_value) / 15
        // This gives reasonable scaling (level 1 vs level 1 enemy = ~2 EXP per 3 seconds)
        let exp_per_cycle = ((avg_hero_power * avg_enemy_value) / 15).max(5);

        log::info!("📊 AFK EXP calculation: hero_power={}, enemy_value={}, exp_per_cycle={}",
            avg_hero_power, avg_enemy_value, exp_per_cycle);

        exp_per_cycle
    }

    /// Process EXP gain
    fn process_exp_gain(&mut self) {
        if self.last_exp_gain.elapsed() >= EXP_GAIN_INTERVAL {
            self.last_exp_gain = Instant::now();

            // Give EXP to all heroes
            for entity in &mut self.hero_entities {
                let old_level = entity.rustymon.level;
                entity.rustymon.exp += self.exp_per_cycle;

                // Check for level up
                let exp_for_next = self.game_data.get_exp_for_level(entity.rustymon.level);
                if entity.rustymon.exp >= exp_for_next {
                    entity.rustymon.level += 1;
                    entity.rustymon.recalculate_stats();
                    log::info!("🎉 {} leveled up {} → {}!",
                        entity.rustymon.name, old_level, entity.rustymon.level);
                }

                log::debug!("{} gained {} EXP (total: {})",
                    entity.rustymon.name, self.exp_per_cycle, entity.rustymon.exp);
            }

            self.total_exp_gained += self.exp_per_cycle * self.hero_entities.len() as u32;
        }
    }

    /// Draw EXP bar
    fn draw_exp_bar(
        display: &mut Sh8601Driver,
        current_exp: u32,
        exp_gained: u32,
        level: u32,
        game_data: &GameData,
        x: i32,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        // Bar dimensions
        let bar_width = 150;
        let bar_height = 8;

        // Calculate exp to next level
        let exp_for_current_level_start = if level > 1 {
            game_data.get_exp_for_level(level - 1)
        } else {
            0
        };
        let exp_for_next_level = game_data.get_exp_for_level(level);

        let exp_needed = if exp_for_next_level > exp_for_current_level_start {
            exp_for_next_level - exp_for_current_level_start
        } else {
            1
        };

        let exp_progress = if current_exp >= exp_for_current_level_start {
            (current_exp - exp_for_current_level_start).min(exp_needed)
        } else {
            0
        };

        // Background bar
        Rectangle::new(
            Point::new(x, y),
            Size::new(bar_width as u32, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // Progress bar (green)
        let filled_width = if exp_needed > 0 {
            ((exp_progress as f32 / exp_needed as f32) * bar_width as f32) as u32
        } else {
            0
        };

        if filled_width > 0 {
            Rectangle::new(
                Point::new(x, y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 200, 100)))
            .draw(display)?;
        }

        // EXP text
        use core::fmt::Write;
        let mut exp_text = heapless::String::<32>::new();
        write!(exp_text, "+{}", exp_gained).ok();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(150, 255, 150));
        Text::new(&exp_text, Point::new(x + bar_width + 5, y + 7), text_style).draw(display)?;

        Ok(())
    }

    /// Get updated rustymon with EXP gains
    pub fn get_updated_rustymon(&self) -> Vec<Rustymon> {
        self.hero_entities.iter().map(|e| e.rustymon.clone()).collect()
    }

    /// Check if user touched the stop button
    pub fn handle_touch(&self, x: i32, y: i32) -> bool {
        // Stop button area at bottom center
        let button_x = 100;
        let button_y = 410;
        let button_w = 168;
        let button_h = 40;

        x >= button_x && x <= button_x + button_w && y >= button_y && y <= button_y + button_h
    }

    /// Get farming statistics
    pub fn get_stats(&self) -> (u32, u64) {
        let elapsed_secs = self.start_time.elapsed().as_secs();
        (self.total_exp_gained, elapsed_secs)
    }
}

impl Page for AfkFarmPage {
    fn update(&mut self) -> bool {
        // Update sprites
        for entity in &mut self.hero_entities {
            entity.update();
        }
        for entity in &mut self.enemy_entities {
            entity.update();
        }

        // Process EXP gain
        self.process_exp_gain();

        // Always keep page active - user must manually stop
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Clear background
        display.clear(self.background_color)?;

        // Title
        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 200, 100));
        Text::new("⚔️ AFK FARMING ⚔️", Point::new(65, 25), title_style).draw(display)?;

        // Stats display
        let (total_exp, elapsed) = self.get_stats();
        let exp_per_min = if elapsed > 0 {
            (total_exp as f32 / elapsed as f32 * 60.0) as u32
        } else {
            0
        };

        use core::fmt::Write;
        let mut stats_text = heapless::String::<48>::new();
        write!(stats_text, "Total: {} EXP | {}/min", total_exp, exp_per_min).ok();
        let stats_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
        Text::new(&stats_text, Point::new(80, 45), stats_style).draw(display)?;

        // Draw entities and EXP bars
        for (i, entity) in self.hero_entities.iter().enumerate() {
            let base_y = 70 + (i as i32 * 100);

            // Draw sprite
            entity.draw(display)?;

            // Draw name and level
            let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
            let mut name_text = heapless::String::<24>::new();
            write!(name_text, "{} Lv{}", entity.rustymon.name, entity.rustymon.level).ok();
            Text::new(&name_text, Point::new(180, base_y + 20), name_style).draw(display)?;

            // Draw EXP bar
            let exp_gained = entity.rustymon.exp.saturating_sub(entity.initial_exp);
            Self::draw_exp_bar(
                display,
                entity.rustymon.exp,
                exp_gained,
                entity.rustymon.level,
                &self.game_data,
                180,
                base_y + 35,
            )?;
        }

        // Draw enemies (just for visual)
        for entity in &self.enemy_entities {
            entity.draw(display)?;
        }

        // Draw stop button
        let button_x = 100;
        let button_y = 410;
        let button_w = 168;
        let button_h = 40;

        Rectangle::new(
            Point::new(button_x, button_y),
            Size::new(button_w, button_h),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 50, 50)))
        .draw(display)?;

        let button_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("Stop Farming", Point::new(button_x + 20, button_y + 25), button_style).draw(display)?;

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
