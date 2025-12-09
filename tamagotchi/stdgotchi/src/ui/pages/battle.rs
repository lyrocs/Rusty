//! Battle Page (Stub)
//!
//! NOTE: This is a placeholder for Phase 1 migration.
//! Will be replaced with new real-time combat system in Phase 2.

use crate::assets::battle::load_enemy_sprites_embedded;
use crate::display::St7789pDriver;
use crate::ecs::resources::SdCardWrapper;
use crate::game::{Enemy as GameEnemy, GameData, KillTracker, BattleState};
use crate::game::battle::DamageResult;
use crate::ui::page::Page;
use crate::ui::sprite::{AnimatedSprite, Background};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::{FONT_9X15, FONT_10X20}},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};

/// Action returned from battle page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleAction {
    None,
    Victory,
    Defeat,
    Flee,
}

/// Simplified battle page - placeholder for new combat system
pub struct BattlePage {
    enemy: GameEnemy,
    hero_hp: u32,
    hero_max_hp: u32,
    hero_atk: u32,
    hero_def: u32,
    hero_spd: u32,
    kill_tracker: KillTracker,
    last_update: Instant,
    hero_atk_bar: f32,    // 0.0 to 1.0, auto-attacks when full
    enemy_atk_bar: f32,   // Enemy attack bar
    battle_state: BattleState,
    is_victory: bool,
    is_defeat: bool,
    exp_gained: u64,
    enemy_sprite: Option<AnimatedSprite>,
    dirty: bool,
}

impl BattlePage {
    pub fn new(
        enemy: GameEnemy,
        _game_data: &GameData,
        kill_tracker: KillTracker,
        _sd_card: Option<&mut SdCardWrapper>,
    ) -> Result<Self, Box<dyn Error>> {
        // Load enemy sprite
        let enemy_sprite = if let Some((idle, _, _, _)) = load_enemy_sprites_embedded(enemy.id) {
            let frame_delay = Duration::from_millis(100);
            AnimatedSprite::new(&idle, (280, 200), frame_delay, None).ok()
        } else {
            None
        };

        Ok(Self {
            exp_gained: enemy.exp_reward,
            enemy,
            hero_hp: 100,
            hero_max_hp: 100,
            hero_atk: 20,
            hero_def: 10,
            hero_spd: 50,
            kill_tracker,
            last_update: Instant::now(),
            hero_atk_bar: 0.0,
            enemy_atk_bar: 0.0,
            battle_state: BattleState::default(),
            is_victory: false,
            is_defeat: false,
            enemy_sprite,
            dirty: true,
        })
    }

    pub fn handle_touch(&mut self, _x: i32, _y: i32) -> BattleAction {
        // Battle is now automatic - touch does nothing
        BattleAction::None
    }

    /// Update combat state and return battle result
    pub fn update_combat(&mut self) -> BattleAction {
        if self.is_victory || self.is_defeat {
            return if self.is_victory { BattleAction::Victory } else { BattleAction::Defeat };
        }

        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32().min(0.1);
        self.last_update = now;

        // Update hero ATK bar based on speed
        let hero_attacks_per_sec = self.hero_spd as f32 / 100.0; // 50 spd = 0.5 attacks/sec
        self.hero_atk_bar += delta * hero_attacks_per_sec;

        // Hero auto-attack when bar is full
        if self.hero_atk_bar >= 1.0 {
            self.hero_atk_bar = 0.0;

            let damage = self.hero_atk.saturating_sub(self.enemy.def / 2).max(1);
            self.enemy.take_damage(damage);

            if !self.enemy.is_alive() {
                self.is_victory = true;
                self.kill_tracker.record_kill(self.enemy.id, &self.enemy.name);
                return BattleAction::Victory;
            }
        }

        // Update enemy ATK bar (enemies attack slower - use fixed speed of 30)
        let enemy_spd = 30u32;
        let enemy_attacks_per_sec = enemy_spd as f32 / 100.0;
        self.enemy_atk_bar += delta * enemy_attacks_per_sec;

        // Enemy auto-attack when bar is full
        if self.enemy_atk_bar >= 1.0 {
            self.enemy_atk_bar = 0.0;

            let enemy_damage = self.enemy.atk.saturating_sub(self.hero_def / 2).max(1);
            self.hero_hp = self.hero_hp.saturating_sub(enemy_damage);

            if self.hero_hp == 0 {
                self.is_defeat = true;
                return BattleAction::Defeat;
            }
        }

        BattleAction::None
    }

    pub fn hero_died(&self) -> bool {
        self.is_defeat
    }

    pub fn is_victory(&self) -> bool {
        self.is_victory
    }

    pub fn get_exp_gained(&self) -> u64 {
        self.exp_gained
    }

    pub fn get_kill_tracker(&self) -> &KillTracker {
        &self.kill_tracker
    }

    pub fn get_enemy(&self) -> &GameEnemy {
        &self.enemy
    }
}

impl Page for BattlePage {
    fn draw(&mut self, display: &mut St7789pDriver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Always clear screen for real-time combat updates
        let bg = Rectangle::new(Point::new(0, 0), Size::new(368, 448));
        display.fill_solid(&bg, Rgb888::new(40, 50, 60))?;

        let style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        let small_style = MonoTextStyle::new(&FONT_9X15, Rgb888::WHITE);

        // Draw battle header
        Text::new("BATTLE", Point::new(150, 30), style)
            .draw(display)?;

        // Draw hero stats (left side)
        Text::new("HERO", Point::new(30, 100), style)
            .draw(display)?;

        let hp_text = format!("HP: {}/{}", self.hero_hp, self.hero_max_hp);
        Text::new(&hp_text, Point::new(30, 130), small_style)
            .draw(display)?;

        // Draw HP bar for hero
        let hp_bar_bg = Rectangle::new(Point::new(30, 140), Size::new(100, 10));
        display.fill_solid(&hp_bar_bg, Rgb888::new(60, 60, 60))?;

        let hp_percent = (self.hero_hp as f32 / self.hero_max_hp as f32).min(1.0);
        let hp_bar_width = (100.0 * hp_percent) as u32;
        let hp_bar = Rectangle::new(Point::new(30, 140), Size::new(hp_bar_width, 10));
        display.fill_solid(&hp_bar, Rgb888::new(80, 200, 80))?;

        // Draw ATK bar for hero
        let atk_bar_bg = Rectangle::new(Point::new(30, 155), Size::new(100, 8));
        display.fill_solid(&atk_bar_bg, Rgb888::new(40, 40, 40))?;

        let atk_bar_width = (100.0 * self.hero_atk_bar) as u32;
        if atk_bar_width > 0 {
            let atk_bar = Rectangle::new(Point::new(30, 155), Size::new(atk_bar_width, 8));
            display.fill_solid(&atk_bar, Rgb888::new(255, 200, 100))?;
        }
        Text::new("ATK", Point::new(135, 162), small_style).draw(display)?;

        // Draw enemy info (right side)
        Text::new(&self.enemy.name, Point::new(220, 100), style)
            .draw(display)?;

        let enemy_hp_text = format!("HP: {}/{}", self.enemy.current_hp, self.enemy.max_hp);
        Text::new(&enemy_hp_text, Point::new(220, 130), small_style)
            .draw(display)?;

        // Draw HP bar for enemy
        let enemy_hp_bar_bg = Rectangle::new(Point::new(220, 140), Size::new(100, 10));
        display.fill_solid(&enemy_hp_bar_bg, Rgb888::new(60, 60, 60))?;

        let enemy_hp_percent = (self.enemy.current_hp as f32 / self.enemy.max_hp as f32).min(1.0);
        let enemy_hp_bar_width = (100.0 * enemy_hp_percent) as u32;
        let enemy_hp_bar = Rectangle::new(Point::new(220, 140), Size::new(enemy_hp_bar_width, 10));
        display.fill_solid(&enemy_hp_bar, Rgb888::new(200, 80, 80))?;

        // Draw ATK bar for enemy
        let enemy_atk_bar_bg = Rectangle::new(Point::new(220, 155), Size::new(100, 8));
        display.fill_solid(&enemy_atk_bar_bg, Rgb888::new(40, 40, 40))?;

        let enemy_atk_bar_width = (100.0 * self.enemy_atk_bar) as u32;
        if enemy_atk_bar_width > 0 {
            let enemy_atk_bar = Rectangle::new(Point::new(220, 155), Size::new(enemy_atk_bar_width, 8));
            display.fill_solid(&enemy_atk_bar, Rgb888::new(255, 150, 100))?;
        }
        Text::new("ATK", Point::new(325, 162), small_style).draw(display)?;

        // Draw enemy sprite if available
        if let Some(ref mut sprite) = self.enemy_sprite {
            sprite.draw(display)?;
        }

        // Draw battle status
        if self.is_victory {
            Text::new("VICTORY!", Point::new(130, 380), MonoTextStyle::new(&FONT_10X20, Rgb888::new(100, 255, 100)))
                .draw(display)?;
        } else if self.is_defeat {
            Text::new("DEFEAT", Point::new(140, 380), MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 100, 100)))
                .draw(display)?;
        } else {
            Text::new("Auto Battle", Point::new(130, 380), style)
                .draw(display)?;
        }

        display.flush()?;
        self.dirty = false;
        Ok(())
    }

    fn update(&mut self) -> bool {
        // Update enemy sprite animation
        if let Some(ref mut sprite) = self.enemy_sprite {
            sprite.update();
        }

        // Update combat (ATK bars and auto-attacks)
        self.update_combat();

        // Always need redraw for real-time combat
        true
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.dirty
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
