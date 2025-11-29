//! Battle Page
//!
//! Simplified hero vs enemy battle system with auto-attack combat

use crate::assets::battle::{load_enemy_sprites_embedded};
use crate::display::Sh8601Driver;
use crate::game::{self, Enemy as GameEnemy, GameData, Hero, KillTracker, BattleState};
use crate::game::battle::{DamageResult, hero_attack_with_battle_state, enemy_attack_with_battle_state};
use crate::ui::page::Page;
use crate::ui::sprite::AnimatedSprite;
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};

/// Animation types for battle entities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Idle,
    Attack,
    Attacked,
    Death,
}

/// Entity role in battle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityRole {
    Hero,
    Enemy,
}

/// Battle entity with animation state
pub struct BattleEntity {
    idle_sprite: AnimatedSprite,
    attack_sprite: AnimatedSprite,
    attacked_sprite: AnimatedSprite,
    death_sprite: Option<AnimatedSprite>,
    current_animation: AnimationType,
    role: EntityRole,
    attack_interval: Duration,
    is_dead: bool,
    death_time: Option<Instant>,
}

impl BattleEntity {
    /// Create a new battle entity with flip support
    pub fn new_with_flip(
        role: EntityRole,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        death_data: Option<&[u8]>,
        position: (i32, i32),
        attack_interval: Duration,
        flip_horizontal: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let frame_delay = Duration::from_millis(100);

        let mut idle_sprite = AnimatedSprite::new(idle_data, position, frame_delay, None)?;
        idle_sprite.set_flip_horizontal(flip_horizontal);
        idle_sprite.set_center_positioned(true);

        let mut attack_sprite = AnimatedSprite::new(attack_data, position, frame_delay, Some(1))?;
        attack_sprite.set_flip_horizontal(flip_horizontal);
        attack_sprite.set_center_positioned(true);

        let mut attacked_sprite = AnimatedSprite::new(attacked_data, position, frame_delay, Some(1))?;
        attacked_sprite.set_flip_horizontal(flip_horizontal);
        attacked_sprite.set_center_positioned(true);

        let death_sprite = death_data
            .map(|data| {
                let mut sprite = AnimatedSprite::new(data, position, frame_delay, Some(1))?;
                sprite.set_flip_horizontal(flip_horizontal);
                sprite.set_center_positioned(true);
                Ok::<_, Box<dyn Error>>(sprite)
            })
            .transpose()?;

        Ok(Self {
            idle_sprite,
            attack_sprite,
            attacked_sprite,
            death_sprite,
            current_animation: AnimationType::Idle,
            role,
            attack_interval,
            is_dead: false,
            death_time: None,
        })
    }

    /// Get current sprite based on animation state
    fn current_sprite(&self) -> &AnimatedSprite {
        match self.current_animation {
            AnimationType::Idle => &self.idle_sprite,
            AnimationType::Attack => &self.attack_sprite,
            AnimationType::Attacked => &self.attacked_sprite,
            AnimationType::Death => self.death_sprite.as_ref().unwrap_or(&self.idle_sprite),
        }
    }

    /// Get mutable current sprite
    fn current_sprite_mut(&mut self) -> &mut AnimatedSprite {
        match self.current_animation {
            AnimationType::Idle => &mut self.idle_sprite,
            AnimationType::Attack => &mut self.attack_sprite,
            AnimationType::Attacked => &mut self.attacked_sprite,
            AnimationType::Death => self.death_sprite.as_mut().unwrap_or(&mut self.idle_sprite),
        }
    }

    /// Set animation type
    fn set_animation(&mut self, animation: AnimationType) {
        if self.current_animation != animation {
            self.current_animation = animation;
            self.current_sprite_mut().reset_animation();
        }
    }

    /// Check if current animation is complete
    fn is_animation_complete(&self) -> bool {
        self.current_sprite().is_animation_complete()
    }

    /// Update entity state
    fn update(&mut self) {
        self.current_sprite_mut().update();

        // Return to idle after attack/attacked animations complete
        if (self.current_animation == AnimationType::Attack ||
            self.current_animation == AnimationType::Attacked) &&
            self.is_animation_complete() {
            self.set_animation(AnimationType::Idle);
        }
    }

    /// Draw entity
    fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        self.current_sprite().draw(display)
    }

    /// Trigger attack animation
    fn start_attack(&mut self) {
        self.set_animation(AnimationType::Attack);
    }

    /// Trigger attacked animation
    fn start_attacked(&mut self) {
        self.set_animation(AnimationType::Attacked);
    }

    /// Trigger death animation
    fn die(&mut self) {
        self.is_dead = true;
        self.death_time = Some(Instant::now());
        self.set_animation(AnimationType::Death);
    }
}

/// Damage number floating animation
struct DamageNumber {
    damage: u32,
    position: Point,
    start_time: Instant,
    duration: Duration,
    is_critical: bool,
    is_miss: bool,
}

impl DamageNumber {
    fn new(damage: u32, position: Point, is_critical: bool, is_miss: bool) -> Self {
        Self {
            damage,
            position,
            start_time: Instant::now(),
            duration: Duration::from_millis(1500),
            is_critical,
            is_miss,
        }
    }

    fn is_expired(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let progress = (elapsed / self.duration.as_millis() as f32).min(1.0);

        // Float upward
        let y_offset = (progress * 30.0) as i32;
        let draw_pos = Point::new(self.position.x, self.position.y - y_offset);

        // Fade out
        let alpha = ((1.0 - progress) * 255.0) as u8;

        let (text, color) = if self.is_miss {
            ("MISS".to_string(), Rgb888::new(150, 150, 150))
        } else if self.is_critical {
            let mut text = heapless::String::<16>::new();
            write!(text, "{}!", self.damage).ok();
            (text.to_string(), Rgb888::new(255, 100, 0))
        } else {
            (self.damage.to_string(), Rgb888::new(255, 255, 255))
        };

        let style = if self.is_critical {
            MonoTextStyle::new(&FONT_10X20, color)
        } else {
            MonoTextStyle::new(&FONT_6X10, color)
        };

        Text::new(&text, draw_pos, style).draw(display)?;
        Ok(())
    }
}

/// Touch area for battle UI
struct TouchArea {
    bounds: (i32, i32, u32, u32),
    action: BattleAction,
}

/// Battle actions from touch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleAction {
    ToggleAuto,
}

/// Main battle page
pub struct BattlePage {
    background_color: Rgb888,
    hero_entity: Option<BattleEntity>,
    enemy_entity: Option<BattleEntity>,

    // Game state
    hero: Hero,
    game_enemy: Option<GameEnemy>,
    kill_tracker: KillTracker,
    game_data: GameData,

    // Enemy waves
    enemy_ids: Vec<u32>,
    current_enemy_index: usize,

    // Damage number animations
    damage_numbers: Vec<DamageNumber>,

    // Battle state (tracks attack timing)
    battle_state: BattleState,

    // Auto-battle mode
    auto_mode: bool,

    // Touch interaction
    touch_areas: Vec<TouchArea>,

    // First draw flag
    first_draw: bool,
}

impl BattlePage {
    /// Create a new battle page
    pub fn new(
        background_color: Rgb888,
        hero: Hero,
        kill_tracker: KillTracker,
        game_data: GameData,
    ) -> Self {
        log::info!("Creating battle page with hero: {} (Level {})", hero.name, hero.level);

        // Create touch areas for auto-mode button
        let touch_areas = vec![
            TouchArea {
                bounds: (10, 10, 80, 30), // Auto button in top-left
                action: BattleAction::ToggleAuto,
            },
        ];

        Self {
            background_color,
            hero_entity: None,
            enemy_entity: None,
            hero,
            game_enemy: None,
            kill_tracker,
            game_data,
            enemy_ids: Vec::new(),
            current_enemy_index: 0,
            damage_numbers: Vec::new(),
            battle_state: BattleState::default(),
            auto_mode: false,
            touch_areas,
            first_draw: true,
        }
    }

    /// Add enemy ID to the spawn pool
    pub fn add_enemy_id_to_pool(&mut self, enemy_id: u32) {
        self.enemy_ids.push(enemy_id);
    }

    /// Get kill tracker
    pub fn get_kill_tracker(&self) -> &KillTracker {
        &self.kill_tracker
    }

    /// Get updated hero
    pub fn get_hero(&self) -> &Hero {
        &self.hero
    }

    /// Check if hero died
    pub fn hero_died(&self) -> bool {
        self.hero.current_health <= 0
    }

    /// Toggle auto-battle mode
    pub fn toggle_auto(&mut self) {
        self.auto_mode = !self.auto_mode;
        log::info!("Auto-battle mode: {}", if self.auto_mode { "ON" } else { "OFF" });
    }

    /// Check if in auto-battle mode
    pub fn is_auto_mode(&self) -> bool {
        self.auto_mode
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<BattleAction> {
        for area in &self.touch_areas {
            let (ax, ay, w, h) = area.bounds;
            if x >= ax && x <= ax + w as i32 && y >= ay && y <= ay + h as i32 {
                return Some(area.action);
            }
        }
        None
    }

    /// Add hero battle entity (called after page creation)
    pub fn add_hero(
        &mut self,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        death_data: Option<&[u8]>,
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        // Hero attacks based on ASPD
        let attack_interval = Duration::from_secs_f32(1.0 / (self.hero.aspd as f32 / 100.0));

        let entity = BattleEntity::new_with_flip(
            EntityRole::Hero,
            idle_data,
            attack_data,
            attacked_data,
            death_data,
            position,
            attack_interval,
            false, // Don't flip hero
        )?;

        self.hero_entity = Some(entity);
        Ok(())
    }

    /// Add enemy battle entity
    pub fn add_enemy(
        &mut self,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        death_data: Option<&[u8]>,
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        // Enemy attacks at 1 attack/sec (aspd 100)
        let attack_interval = Duration::from_secs(1);

        let entity = BattleEntity::new_with_flip(
            EntityRole::Enemy,
            idle_data,
            attack_data,
            attacked_data,
            death_data,
            position,
            attack_interval,
            true, // Flip enemy to face left
        )?;

        self.enemy_entity = Some(entity);
        Ok(())
    }

    /// Process battle combat logic
    fn process_combat(&mut self) {
        // Skip if hero or enemy is dead
        if self.hero_died() {
            if let Some(ref mut hero_entity) = self.hero_entity {
                if !hero_entity.is_dead {
                    hero_entity.die();
                    log::info!("💀 {} has been defeated!", self.hero.name);
                }
            }
            return;
        }

        let enemy_alive = self.game_enemy.as_ref().map(|e| e.current_hp > 0).unwrap_or(false);
        if !enemy_alive {
            if let Some(ref mut enemy_entity) = self.enemy_entity {
                if !enemy_entity.is_dead {
                    enemy_entity.die();
                    if let Some(ref enemy) = self.game_enemy {
                        log::info!("💀 {} has been defeated!", enemy.name);
                    }
                }
            }
            return;
        }

        // Hero auto-attack
        if let (Some(ref mut hero_entity), Some(ref mut enemy)) =
            (self.hero_entity.as_mut(), self.game_enemy.as_mut()) {

            let time_since_last_attack = self.battle_state.hero_last_attack;
            let attack_cooldown = hero_entity.attack_interval.as_secs_f64();
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            if current_time - time_since_last_attack >= attack_cooldown {
                // Hero attacks
                hero_entity.start_attack();
                let result = hero_attack_with_battle_state(&self.hero, enemy, &self.battle_state);

                self.battle_state.hero_last_attack = current_time;

                // Spawn damage number
                if let Some(ref enemy_entity) = self.enemy_entity {
                    let sprite = enemy_entity.current_sprite();
                    let (x, y, w, h) = sprite.bounds();
                    let damage_pos = Point::new(x + w as i32 / 2, y);
                    self.damage_numbers.push(DamageNumber::new(
                        result.damage,
                        damage_pos,
                        result.is_critical,
                        result.is_miss,
                    ));
                }

                // Trigger enemy attacked animation
                if !result.is_miss {
                    if let Some(ref mut enemy_entity) = self.enemy_entity {
                        enemy_entity.start_attacked();
                    }
                }
            }
        }

        // Enemy auto-attack
        if let Some(ref enemy) = self.game_enemy {
            let time_since_last_attack = self.battle_state.enemy_last_attack;
            let attack_cooldown = self.enemy_entity.as_ref()
                .map(|e| e.attack_interval.as_secs_f64())
                .unwrap_or(2.0);
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            if current_time - time_since_last_attack >= attack_cooldown {
                // Enemy attacks
                if let Some(ref mut enemy_entity) = self.enemy_entity {
                    enemy_entity.start_attack();
                }

                let result = enemy_attack_with_battle_state(enemy, &mut self.hero, &self.battle_state);
                self.battle_state.enemy_last_attack = current_time;

                // Spawn damage number
                if let Some(ref hero_entity) = self.hero_entity {
                    let sprite = hero_entity.current_sprite();
                    let (x, y, w, h) = sprite.bounds();
                    let damage_pos = Point::new(x + w as i32 / 2, y);
                    self.damage_numbers.push(DamageNumber::new(
                        result.damage,
                        damage_pos,
                        result.is_critical,
                        result.is_miss,
                    ));
                }

                // Trigger hero attacked animation
                if !result.is_miss {
                    if let Some(ref mut hero_entity) = self.hero_entity {
                        hero_entity.start_attacked();
                    }
                }
            }
        }
    }

    /// Spawn next enemy from pool
    fn spawn_next_enemy(&mut self) -> Result<(), Box<dyn Error>> {
        if self.current_enemy_index >= self.enemy_ids.len() {
            log::info!("No more enemies to spawn");
            return Ok(());
        }

        let enemy_id = self.enemy_ids[self.current_enemy_index];
        self.current_enemy_index += 1;

        // Load enemy data
        if let Some(enemy_data) = self.game_data.get_enemy(enemy_id) {
            // Create enemy scaled to hero level
            let enemy = GameEnemy::from_data_scaled(
                enemy_data.id,
                enemy_data.name.clone(),
                enemy_data.level,
                enemy_data.hp,
                enemy_data.attack,
                enemy_data.defense,
                enemy_data.hit,
                enemy_data.flee,
                enemy_data.base_exp,
                enemy_data.get_element(),
                self.hero.level,
            );
            log::info!("Spawning enemy: {} (HP: {}/{})", enemy.name, enemy.current_hp, enemy.max_hp);

            // Load enemy sprites
            if let Some((idle, attack, attacked, death)) = load_enemy_sprites_embedded(enemy_id) {
                let position = (270, 220); // Enemy position on right side
                let death_ref = death.as_ref().map(|v| v.as_slice());
                self.add_enemy(&idle, &attack, &attacked, death_ref, position)?;
            }

            self.game_enemy = Some(enemy);
        }

        Ok(())
    }

    /// Check for enemy death and spawn next
    fn check_enemy_death(&mut self) {
        if let Some(ref enemy) = self.game_enemy {
            if enemy.current_hp == 0 {
                // Enemy died, record kill
                self.kill_tracker.record_kill(enemy.id, &enemy.name);
                log::info!("Enemy {} defeated! Total kills: {}",
                    enemy.name, self.kill_tracker.get_total_kills());

                // Clear current enemy
                self.game_enemy = None;
                self.enemy_entity = None;

                // Spawn next enemy after a delay
                // For now, spawn immediately
                let _ = self.spawn_next_enemy();
            }
        }
    }

    /// Draw HP bar
    fn draw_hp_bar(
        display: &mut Sh8601Driver,
        current_hp: i32,
        max_hp: i32,
        x: i32,
        y: i32,
        width: u32,
    ) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        let bar_height = 8;

        // Background
        Rectangle::new(
            Point::new(x, y),
            Size::new(width, bar_height),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
        .draw(display)?;

        // HP bar
        let hp_percentage = (current_hp as f32 / max_hp as f32).max(0.0).min(1.0);
        let filled_width = (hp_percentage * width as f32) as u32;

        let hp_color = if hp_percentage > 0.6 {
            Rgb888::new(0, 200, 0)
        } else if hp_percentage > 0.3 {
            Rgb888::new(200, 200, 0)
        } else {
            Rgb888::new(200, 0, 0)
        };

        if filled_width > 0 {
            Rectangle::new(
                Point::new(x, y),
                Size::new(filled_width, bar_height),
            )
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;
        }

        // HP text
        let mut hp_text = heapless::String::<32>::new();
        write!(hp_text, "{}/{}", current_hp.max(0), max_hp).ok();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new(&hp_text, Point::new(x + 2, y + 7), text_style).draw(display)?;

        Ok(())
    }
}

impl Page for BattlePage {
    fn update(&mut self) -> bool {
        // Spawn first enemy on first update
        if self.first_draw && self.game_enemy.is_none() && !self.enemy_ids.is_empty() {
            let _ = self.spawn_next_enemy();
            self.battle_state.start_battle();
        }

        // Update battle entities
        if let Some(ref mut hero) = self.hero_entity {
            hero.update();
        }
        if let Some(ref mut enemy) = self.enemy_entity {
            enemy.update();
        }

        // Process combat in auto mode
        if self.auto_mode {
            self.process_combat();
        }

        // Check for enemy death
        self.check_enemy_death();

        // Update damage numbers
        self.damage_numbers.retain(|d| !d.is_expired());

        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Clear background
        display.clear(self.background_color)?;

        // Draw hero entity
        if let Some(ref hero) = self.hero_entity {
            hero.draw(display)?;
        }

        // Draw enemy entity
        if let Some(ref enemy) = self.enemy_entity {
            enemy.draw(display)?;
        }

        // Draw hero info (top-left)
        let hero_info_y = 50;
        let name_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new(&self.hero.name, Point::new(10, hero_info_y), name_style).draw(display)?;

        let job_name = self.hero.job.get_name();
        let mut level_text = heapless::String::<32>::new();
        write!(level_text, "Lv{} {}", self.hero.level, job_name).ok();
        let level_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
        Text::new(&level_text, Point::new(10, hero_info_y + 15), level_style).draw(display)?;

        // Hero HP bar
        Self::draw_hp_bar(
            display,
            self.hero.current_health,
            self.hero.max_health,
            10,
            hero_info_y + 20,
            150,
        )?;

        // Draw enemy info (top-right)
        if let Some(ref enemy) = self.game_enemy {
            let enemy_info_x = 220;
            let enemy_info_y = 50;

            let enemy_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 100, 100));
            Text::new(&enemy.name, Point::new(enemy_info_x, enemy_info_y), enemy_style).draw(display)?;

            let mut enemy_level = heapless::String::<16>::new();
            write!(enemy_level, "Lv{}", enemy.level).ok();
            let enemy_level_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));
            Text::new(&enemy_level, Point::new(enemy_info_x, enemy_info_y + 15), enemy_level_style).draw(display)?;

            // Enemy HP bar
            Self::draw_hp_bar(
                display,
                enemy.current_hp as i32,
                enemy.max_hp as i32,
                enemy_info_x,
                enemy_info_y + 20,
                130,
            )?;
        }

        // Draw damage numbers
        for damage_num in &self.damage_numbers {
            damage_num.draw(display)?;
        }

        // Draw auto-mode button
        let auto_button_bounds = (10, 10, 80, 30);
        let auto_color = if self.auto_mode {
            Rgb888::new(0, 200, 0)
        } else {
            Rgb888::new(100, 100, 100)
        };

        Rectangle::new(
            Point::new(auto_button_bounds.0, auto_button_bounds.1),
            Size::new(auto_button_bounds.2, auto_button_bounds.3),
        )
        .into_styled(PrimitiveStyle::with_fill(auto_color))
        .draw(display)?;

        let auto_text = if self.auto_mode { "AUTO: ON" } else { "AUTO: OFF" };
        let auto_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new(auto_text, Point::new(auto_button_bounds.0 + 5, auto_button_bounds.1 + 20), auto_style).draw(display)?;

        self.first_draw = false;
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
