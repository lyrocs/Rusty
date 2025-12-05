//! Semi-Active Battle Page
//!
//! Turn-based battle with active skill usage. Hero attacks first with skills,
//! then enemy attacks, alternating until one side is defeated.

use crate::assets::battle::load_enemy_sprites_embedded;
use crate::display::Sh8601Driver;
use crate::game::{Enemy as GameEnemy, GameData, Hero, KillTracker, BattleState};
use crate::game::battle::{hero_attack_with_battle_state, enemy_attack_with_battle_state};
use crate::game::skill::{SkillData, SkillType, SkillTarget};
use crate::ui::page::Page;
use crate::ui::sprite::AnimatedSprite;
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, CornerRadii},
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

/// Battle turn state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleTurn {
    HeroTurn,           // Waiting for player input
    HeroAttacking,      // Playing hero attack animation
    EnemyHit,           // Playing enemy hit animation
    EnemyTurn,          // Brief pause before enemy attacks
    EnemyAttacking,     // Playing enemy attack animation
    HeroHit,            // Playing hero hit animation
    BattleEnded,        // Battle is over
}

/// Battle entity with animation state
pub struct BattleEntity {
    idle_sprite: AnimatedSprite,
    attack_sprite: AnimatedSprite,
    attacked_sprite: AnimatedSprite,
    death_sprite: Option<AnimatedSprite>,
    current_animation: AnimationType,
    role: EntityRole,
    is_dead: bool,
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
            is_dead: false,
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
    }

    /// Draw entity
    fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        self.current_sprite().draw(display)
    }

    /// Get bounding box (x, y, width, height) for the current sprite
    fn bounds(&self) -> (i32, i32, u32, u32) {
        self.current_sprite().bounds()
    }

    /// Start attack animation
    fn start_attack(&mut self) {
        self.set_animation(AnimationType::Attack);
    }

    /// Start attacked animation
    fn start_attacked(&mut self) {
        self.set_animation(AnimationType::Attacked);
    }

    /// Trigger death animation
    fn die(&mut self) {
        self.is_dead = true;
        self.set_animation(AnimationType::Death);
    }

    /// Return to idle animation
    fn return_to_idle(&mut self) {
        self.set_animation(AnimationType::Idle);
    }
}

/// Damage number floating animation
struct DamageNumber {
    value: i32,  // Negative for damage, positive for heal
    position: Point,
    start_time: Instant,
    duration: Duration,
    is_critical: bool,
    is_miss: bool,
    is_heal: bool,
}

impl DamageNumber {
    fn new_damage(damage: u32, position: Point, is_critical: bool, is_miss: bool) -> Self {
        Self {
            value: -(damage as i32),
            position,
            start_time: Instant::now(),
            duration: Duration::from_millis(1500),
            is_critical,
            is_miss,
            is_heal: false,
        }
    }

    fn new_heal(heal: u32, position: Point) -> Self {
        Self {
            value: heal as i32,
            position,
            start_time: Instant::now(),
            duration: Duration::from_millis(1500),
            is_critical: false,
            is_miss: false,
            is_heal: true,
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

        let (text, color) = if self.is_miss {
            ("MISS".to_string(), Rgb888::new(150, 150, 150))
        } else if self.is_heal {
            let mut text = heapless::String::<16>::new();
            write!(text, "+{}", self.value).ok();
            (text.to_string(), Rgb888::new(0, 255, 0)) // Green for heal
        } else if self.is_critical {
            let mut text = heapless::String::<16>::new();
            write!(text, "{}!", self.value.abs()).ok();
            (text.to_string(), Rgb888::new(255, 100, 0)) // Orange for crit
        } else {
            (self.value.abs().to_string(), Rgb888::new(255, 255, 255))
        };

        let style = if self.is_critical || self.is_heal {
            MonoTextStyle::new(&FONT_10X20, color)
        } else {
            MonoTextStyle::new(&FONT_6X10, color)
        };

        Text::new(&text, draw_pos, style).draw(display)?;
        Ok(())
    }
}

/// Skill button state
#[derive(Debug, Clone)]
pub struct SkillButton {
    pub skill_id: Option<u32>,
    pub skill_name: String,
    pub cooldown_remaining: f32,
    pub max_cooldown: f32,
    pub bounds: (i32, i32, u32, u32), // x, y, width, height
}

impl SkillButton {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            skill_id: None,
            skill_name: String::new(),
            cooldown_remaining: 0.0,
            max_cooldown: 0.0,
            bounds: (x, y, 110, 55),  // Bigger buttons: 110x55
        }
    }

    pub fn is_ready(&self) -> bool {
        self.skill_id.is_some() && self.cooldown_remaining <= 0.0
    }

    pub fn is_empty(&self) -> bool {
        self.skill_id.is_none()
    }

    pub fn set_skill(&mut self, skill_id: u32, skill_name: &str, cooldown: f32) {
        self.skill_id = Some(skill_id);
        self.skill_name = skill_name.to_string();
        self.max_cooldown = cooldown;
        self.cooldown_remaining = 0.0;
    }

    pub fn use_skill(&mut self) {
        self.cooldown_remaining = self.max_cooldown;
    }

    pub fn start_cooldown(&mut self) {
        self.cooldown_remaining = self.max_cooldown;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.cooldown_remaining > 0.0 {
            self.cooldown_remaining -= delta_time;
            if self.cooldown_remaining < 0.0 {
                self.cooldown_remaining = 0.0;
            }
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        let (bx, by, bw, bh) = self.bounds;
        x >= bx && x < bx + bw as i32 && y >= by && y < by + bh as i32
    }
}

/// Result of the battle
#[derive(Debug, Clone, PartialEq)]
pub enum BattleResult {
    Victory { exp_gained: u32, cards_dropped: Vec<crate::game::expedition::Card> },
    Defeat,
    Fled,
    InProgress,
}

/// Semi-Active Battle Page
pub struct SemiActiveBattlePage {
    background_color: Rgb888,
    hero_entity: Option<BattleEntity>,
    enemy_entity: Option<BattleEntity>,

    // Game state
    hero: Hero,
    game_enemy: Option<GameEnemy>,
    kill_tracker: KillTracker,
    game_data: GameData,
    enemy_id: u32,

    // Turn-based state
    turn: BattleTurn,
    turn_timer: Instant,
    animation_duration: Duration,
    hero_auto_attack_delay: Duration,  // Time before hero auto-attacks

    // Skill buttons (3 slots)
    skill_buttons: [SkillButton; 3],
    attack_button_bounds: (i32, i32, u32, u32),

    // Pending damage to apply after animation
    pending_hero_damage: Option<(u32, bool, bool)>, // damage, is_crit, is_miss
    pending_enemy_damage: Option<(u32, bool, bool)>,

    // Damage number animations
    damage_numbers: Vec<DamageNumber>,

    // Battle result
    result: BattleResult,

    // UI state
    first_draw: bool,
    needs_redraw: bool,
    last_update: Instant,
}

impl SemiActiveBattlePage {
    /// Create a new semi-active battle page
    pub fn new(
        background_color: Rgb888,
        hero: Hero,
        enemy_id: u32,
        kill_tracker: KillTracker,
        game_data: GameData,
    ) -> Self {
        log::info!("Creating semi-active battle page for enemy {}", enemy_id);

        // Create skill buttons at bottom of screen (spread across width, no attack button)
        // 3 buttons of 110px each with spacing: 10 + 110 + 10 + 110 + 10 + 110 + 10 = 370
        let skill_buttons = [
            SkillButton::new(10, 390),
            SkillButton::new(128, 390),
            SkillButton::new(246, 390),
        ];

        // Attack button removed - auto-attack mode
        let attack_button_bounds = (0, 0, 0, 0);  // Not used

        Self {
            background_color,
            hero_entity: None,
            enemy_entity: None,
            hero,
            game_enemy: None,
            kill_tracker,
            game_data,
            enemy_id,
            turn: BattleTurn::HeroTurn,
            turn_timer: Instant::now(),
            animation_duration: Duration::from_millis(500),
            hero_auto_attack_delay: Duration::from_millis(1500), // Auto-attack after 1.5 seconds
            skill_buttons,
            attack_button_bounds,
            pending_hero_damage: None,
            pending_enemy_damage: None,
            damage_numbers: Vec::new(),
            result: BattleResult::InProgress,
            first_draw: true,
            needs_redraw: true,
            last_update: Instant::now(),
        }
    }

    /// Initialize battle entities and skills
    pub fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        // Load hero skills into buttons
        for (i, slot) in self.hero.equipped_skill_slots.iter().enumerate() {
            if let Some(skill_id) = slot.skill_id {
                if let Some(skill_data) = self.game_data.get_skill(skill_id) {
                    self.skill_buttons[i].set_skill(
                        skill_id,
                        &skill_data.name,
                        skill_data.cooldown_seconds,
                    );
                }
            }
        }

        // Load enemy data and sprites
        if let Some(enemy_data) = self.game_data.get_enemy(self.enemy_id) {
            let enemy = GameEnemy::from_data(
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
            );
            log::info!("Loaded enemy: {} (HP: {})", enemy.name, enemy.max_hp);

            // Load enemy sprites
            if let Some((idle, attack, attacked, death)) = load_enemy_sprites_embedded(self.enemy_id) {
                let position = (270, 200);
                let death_ref = death.as_ref().map(|v| v.as_slice());
                self.add_enemy(&idle, &attack, &attacked, death_ref, position)?;
            }

            self.game_enemy = Some(enemy);
        }

        // Initialize hero skills for battle
        self.hero.initialize_battle_skills();

        Ok(())
    }

    /// Add hero battle entity
    pub fn add_hero(
        &mut self,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        death_data: Option<&[u8]>,
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        let entity = BattleEntity::new_with_flip(
            EntityRole::Hero,
            idle_data,
            attack_data,
            attacked_data,
            death_data,
            position,
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
        let entity = BattleEntity::new_with_flip(
            EntityRole::Enemy,
            idle_data,
            attack_data,
            attacked_data,
            death_data,
            position,
            true, // Flip enemy to face left
        )?;
        self.enemy_entity = Some(entity);
        Ok(())
    }

    /// Handle player using basic attack
    fn handle_basic_attack(&mut self) {
        if self.turn != BattleTurn::HeroTurn {
            return;
        }

        log::info!("Hero uses basic attack");

        // Calculate damage
        if let Some(ref mut enemy) = self.game_enemy {
            let battle_state = BattleState::default();
            let result = hero_attack_with_battle_state(&self.hero, enemy, &battle_state);
            self.pending_enemy_damage = Some((result.damage, result.is_critical, result.is_miss));
        }

        // Start attack animation
        if let Some(ref mut hero_entity) = self.hero_entity {
            hero_entity.start_attack();
        }

        self.turn = BattleTurn::HeroAttacking;
        self.turn_timer = Instant::now();
    }

    /// Handle player using a skill
    fn handle_skill_use(&mut self, skill_index: usize) {
        // Skills can be used at any time (no turn restriction)
        if !self.skill_buttons[skill_index].is_ready() {
            return;
        }

        let skill_id = match self.skill_buttons[skill_index].skill_id {
            Some(id) => id,
            None => return,
        };

        let skill_data = match self.game_data.get_skill(skill_id) {
            Some(data) => data.clone(),
            None => return,
        };

        log::info!("Hero uses skill: {}", skill_data.name);

        match skill_data.skill_type {
            SkillType::Attack => {
                // Calculate skill damage
                if let Some(ref mut enemy) = self.game_enemy {
                    let base_damage = (self.hero.attack as u32 * skill_data.power) / 100;
                    let damage = base_damage.saturating_sub(enemy.def / 2).max(1);
                    enemy.take_damage(damage);
                    self.pending_enemy_damage = Some((damage, false, false));
                }
            }
            SkillType::Heal => {
                // Heal the hero
                let heal_amount = skill_data.power as i32;
                let old_hp = self.hero.current_health;
                self.hero.heal(heal_amount);
                let actual_heal = self.hero.current_health - old_hp;

                // Show heal number
                self.damage_numbers.push(DamageNumber::new_heal(
                    actual_heal as u32,
                    Point::new(80, 150),
                ));
            }
            _ => {
                // TODO: Implement buff/debuff
            }
        }

        // Put skill on cooldown
        self.skill_buttons[skill_index].use_skill();

        // Start attack animation
        if let Some(ref mut hero_entity) = self.hero_entity {
            hero_entity.start_attack();
        }

        self.turn = BattleTurn::HeroAttacking;
        self.turn_timer = Instant::now();
    }

    /// Handle touch input
    pub fn handle_touch(&mut self, x: i32, y: i32) {
        // Skills can be used at any time (not restricted to hero turn)
        // Check skill buttons
        for i in 0..3 {
            if self.skill_buttons[i].contains(x, y) {
                self.handle_skill_use(i);
                return;
            }
        }

        // Basic attack only on hero turn (though auto-attack is now used)
        if self.turn == BattleTurn::HeroTurn {
            let (bx, by, bw, bh) = self.attack_button_bounds;
            if x >= bx && x < bx + bw as i32 && y >= by && y < by + bh as i32 {
                self.handle_basic_attack();
            }
        }
    }

    /// Update turn state machine
    fn update_turn_state(&mut self) {
        let elapsed = self.turn_timer.elapsed();

        match self.turn {
            BattleTurn::HeroTurn => {
                // Auto-attack after delay (player can use skills to interrupt)
                if elapsed >= self.hero_auto_attack_delay {
                    log::info!("Auto-attack triggered after {:?}", elapsed);
                    self.handle_basic_attack();
                }
            }
            BattleTurn::HeroAttacking => {
                // Wait for attack animation to complete (or timeout)
                let animation_done = self.hero_entity
                    .as_ref()
                    .map(|e| e.is_animation_complete())
                    .unwrap_or(true);

                if animation_done || elapsed > self.animation_duration {
                    // Apply pending damage to enemy
                    if let Some((damage, is_crit, is_miss)) = self.pending_enemy_damage.take() {
                        log::info!("Hero dealt {} damage (crit={}, miss={})", damage, is_crit, is_miss);
                        // Show damage number
                        self.damage_numbers.push(DamageNumber::new_damage(
                            damage,
                            Point::new(270, 150),
                            is_crit,
                            is_miss,
                        ));

                        // Trigger enemy hit animation
                        if !is_miss {
                            if let Some(ref mut enemy_entity) = self.enemy_entity {
                                enemy_entity.start_attacked();
                            }
                        }
                    }

                    // Return hero to idle
                    if let Some(ref mut hero_entity) = self.hero_entity {
                        hero_entity.return_to_idle();
                    }

                    self.turn = BattleTurn::EnemyHit;
                    self.turn_timer = Instant::now();
                    log::info!("Transitioning to EnemyHit");
                }
            }
            BattleTurn::EnemyHit => {
                // Wait for hit animation
                if elapsed > Duration::from_millis(300) {
                    // Check if enemy died
                    if let Some(ref enemy) = self.game_enemy {
                        if enemy.current_hp == 0 {
                            if let Some(ref mut enemy_entity) = self.enemy_entity {
                                enemy_entity.die();
                            }
                            // Calculate rewards
                            let exp_gained = enemy.exp_reward as u32;
                            // TODO: Card drops based on enemy data
                            let cards_dropped = Vec::new();
                            self.result = BattleResult::Victory { exp_gained, cards_dropped };
                            self.turn = BattleTurn::BattleEnded;
                            log::info!("Victory! Gained {} exp", exp_gained);
                            return;
                        }
                    }

                    // Return enemy to idle and start enemy turn
                    if let Some(ref mut enemy_entity) = self.enemy_entity {
                        enemy_entity.return_to_idle();
                    }
                    self.turn = BattleTurn::EnemyTurn;
                    self.turn_timer = Instant::now();
                    log::info!("Transitioning to EnemyTurn");
                }
            }
            BattleTurn::EnemyTurn => {
                // Brief pause before enemy attacks
                if elapsed > Duration::from_millis(500) {
                    log::info!("Enemy turn - attacking hero");
                    // Enemy attacks hero
                    if let Some(ref enemy) = self.game_enemy {
                        let battle_state = BattleState::default();
                        let result = enemy_attack_with_battle_state(enemy, &mut self.hero, &battle_state);
                        self.pending_hero_damage = Some((result.damage, result.is_critical, result.is_miss));
                    }

                    // Start enemy attack animation
                    if let Some(ref mut enemy_entity) = self.enemy_entity {
                        enemy_entity.start_attack();
                    }

                    self.turn = BattleTurn::EnemyAttacking;
                    self.turn_timer = Instant::now();
                    log::info!("Transitioning to EnemyAttacking");
                }
            }
            BattleTurn::EnemyAttacking => {
                // Wait for attack animation (or timeout)
                let animation_done = self.enemy_entity
                    .as_ref()
                    .map(|e| e.is_animation_complete())
                    .unwrap_or(true);

                if animation_done || elapsed > self.animation_duration {
                    // Apply pending damage to hero
                    if let Some((damage, is_crit, is_miss)) = self.pending_hero_damage.take() {
                        log::info!("Enemy dealt {} damage (crit={}, miss={})", damage, is_crit, is_miss);
                        // Show damage number
                        self.damage_numbers.push(DamageNumber::new_damage(
                            damage,
                            Point::new(80, 150),
                            is_crit,
                            is_miss,
                        ));

                        // Trigger hero hit animation
                        if !is_miss {
                            if let Some(ref mut hero_entity) = self.hero_entity {
                                hero_entity.start_attacked();
                            }
                        }
                    }

                    // Return enemy to idle
                    if let Some(ref mut enemy_entity) = self.enemy_entity {
                        enemy_entity.return_to_idle();
                    }

                    self.turn = BattleTurn::HeroHit;
                    self.turn_timer = Instant::now();
                    log::info!("Transitioning to HeroHit");
                }
            }
            BattleTurn::HeroHit => {
                // Wait for hit animation
                if elapsed > Duration::from_millis(300) {
                    // Check if hero died
                    if self.hero.current_health <= 0 {
                        if let Some(ref mut hero_entity) = self.hero_entity {
                            hero_entity.die();
                        }
                        self.result = BattleResult::Defeat;
                        self.turn = BattleTurn::BattleEnded;
                        log::info!("Defeat!");
                        return;
                    }

                    // Return hero to idle and start hero turn
                    if let Some(ref mut hero_entity) = self.hero_entity {
                        hero_entity.return_to_idle();
                    }
                    self.turn = BattleTurn::HeroTurn;
                    self.turn_timer = Instant::now();
                }
            }
            BattleTurn::BattleEnded => {
                // Battle is over, waiting for page transition
            }
        }
    }

    /// Get hero reference
    pub fn get_hero(&self) -> &Hero {
        &self.hero
    }

    /// Get kill tracker
    pub fn get_kill_tracker(&self) -> &KillTracker {
        &self.kill_tracker
    }

    /// Get enemy reference
    pub fn get_enemy(&self) -> Option<&GameEnemy> {
        self.game_enemy.as_ref()
    }

    /// Draw HP bar
    fn draw_hp_bar(
        &self,
        display: &mut Sh8601Driver,
        current: i32,
        max: i32,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        label: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Background
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 50, 50)))
            .draw(display)?;

        // HP fill
        let fill_width = if max > 0 {
            ((current.max(0) as f32 / max as f32) * width as f32) as u32
        } else {
            0
        };

        let hp_color = if current * 100 / max.max(1) > 50 {
            Rgb888::new(0, 200, 0) // Green
        } else if current * 100 / max.max(1) > 25 {
            Rgb888::new(200, 200, 0) // Yellow
        } else {
            Rgb888::new(200, 0, 0) // Red
        };

        Rectangle::new(Point::new(x, y), Size::new(fill_width.min(width), height))
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;

        // Label
        let style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        Text::new(label, Point::new(x, y - 2), style).draw(display)?;

        // HP text
        use core::fmt::Write;
        let mut hp_text = heapless::String::<32>::new();
        write!(hp_text, "{}/{}", current.max(0), max).ok();
        Text::new(&hp_text, Point::new(x + width as i32 / 2 - 20, y + height as i32 / 2 + 3), style).draw(display)?;

        Ok(())
    }

    /// Draw skill button
    fn draw_skill_button(
        &self,
        display: &mut Sh8601Driver,
        button: &SkillButton,
    ) -> Result<(), Box<dyn Error>> {
        let (x, y, w, h) = button.bounds;

        // Button background
        let bg_color = if button.is_empty() {
            Rgb888::new(50, 50, 50) // Gray for empty
        } else if button.is_ready() {
            Rgb888::new(0, 100, 200) // Blue for ready
        } else {
            Rgb888::new(100, 50, 50) // Dark red for on cooldown
        };

        RoundedRectangle::new(
            Rectangle::new(Point::new(x, y), Size::new(w, h)),
            CornerRadii::new(Size::new(5, 5)),
        )
        .into_styled(PrimitiveStyle::with_fill(bg_color))
        .draw(display)?;

        // Button text
        let style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);

        if button.is_empty() {
            Text::new("Empty", Point::new(x + 20, y + 25), style).draw(display)?;
        } else {
            // Skill name (truncated)
            let display_name: String = button.skill_name.chars().take(8).collect();
            Text::new(&display_name, Point::new(x + 5, y + 15), style).draw(display)?;

            // Cooldown if not ready
            if !button.is_ready() {
                use core::fmt::Write;
                let mut cd_text = heapless::String::<8>::new();
                write!(cd_text, "{:.1}s", button.cooldown_remaining).ok();
                Text::new(&cd_text, Point::new(x + 25, y + 35), style).draw(display)?;
            } else {
                Text::new("Ready", Point::new(x + 20, y + 35), style).draw(display)?;
            }
        }

        Ok(())
    }

    /// Draw attack button
    fn draw_attack_button(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let (x, y, w, h) = self.attack_button_bounds;

        // Button background
        let bg_color = if self.turn == BattleTurn::HeroTurn {
            Rgb888::new(200, 50, 50) // Red for attack
        } else {
            Rgb888::new(100, 50, 50) // Dark red when not your turn
        };

        RoundedRectangle::new(
            Rectangle::new(Point::new(x, y), Size::new(w, h)),
            CornerRadii::new(Size::new(5, 5)),
        )
        .into_styled(PrimitiveStyle::with_fill(bg_color))
        .draw(display)?;

        // Button text
        let style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
        Text::new("ATK", Point::new(x + 20, y + 28), style).draw(display)?;

        Ok(())
    }

    /// Draw turn indicator
    fn draw_turn_indicator(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let (text, color) = match self.turn {
            BattleTurn::HeroTurn => ("YOUR TURN", Rgb888::new(0, 255, 0)),
            BattleTurn::EnemyTurn | BattleTurn::EnemyAttacking => ("ENEMY TURN", Rgb888::new(255, 0, 0)),
            BattleTurn::BattleEnded => {
                if matches!(self.result, BattleResult::Victory { .. }) {
                    ("VICTORY!", Rgb888::new(255, 215, 0))
                } else {
                    ("DEFEAT", Rgb888::new(255, 0, 0))
                }
            }
            _ => ("", Rgb888::WHITE),
        };

        if !text.is_empty() {
            let style = MonoTextStyle::new(&FONT_10X20, color);
            Text::new(text, Point::new(140, 360), style).draw(display)?;
        }

        Ok(())
    }

    /// Take the battle result (consumes it, returns None on subsequent calls)
    pub fn take_result(&mut self) -> Option<BattleResult> {
        if self.result == BattleResult::InProgress {
            None
        } else {
            Some(std::mem::replace(&mut self.result, BattleResult::InProgress))
        }
    }

    /// Get a reference to the current result
    pub fn get_result(&self) -> &BattleResult {
        &self.result
    }

    /// Attempt to flee from battle
    pub fn attempt_flee(&mut self) {
        if self.result != BattleResult::InProgress {
            return;
        }

        log::info!("Attempting to flee from MVP battle...");
        self.result = BattleResult::Fled;
        self.turn = BattleTurn::BattleEnded;
    }
}

impl Page for SemiActiveBattlePage {
    fn update(&mut self) -> bool {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Update skill cooldowns
        for button in &mut self.skill_buttons {
            button.update(delta);
        }

        // Update entities
        if let Some(ref mut hero_entity) = self.hero_entity {
            hero_entity.update();
        }
        if let Some(ref mut enemy_entity) = self.enemy_entity {
            enemy_entity.update();
        }

        // Update damage numbers
        self.damage_numbers.retain(|dn| !dn.is_expired());

        // Update turn state
        self.update_turn_state();

        self.needs_redraw = true;

        // Return true to keep page open, false when battle ends and transition is complete
        self.result == BattleResult::InProgress || self.turn_timer.elapsed() < Duration::from_secs(2)
    }

    fn draw(&mut self, display: &mut Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn Error>> {
        if full_redraw || self.first_draw {
            // Clear with background color
            Rectangle::new(Point::zero(), Size::new(480, 480))
                .into_styled(PrimitiveStyle::with_fill(self.background_color))
                .draw(display)?;
            self.first_draw = false;
        }

        // Clear and draw hero entity area (with 30px padding, +20px on top for floating damage)
        if let Some(ref hero_entity) = self.hero_entity {
            let (x, y, w, h) = hero_entity.bounds();
            let padding = 30i32;
            let top_padding = 50i32; // Extra 20px on top for floating damage numbers
            let clear_x = (x - padding).max(0);
            let clear_y = (y - top_padding).max(0);
            let clear_w = (w as i32 + padding * 2) as u32;
            let clear_h = (h as i32 + padding + top_padding) as u32;
            Rectangle::new(
                Point::new(clear_x, clear_y),
                Size::new(clear_w, clear_h),
            )
            .into_styled(PrimitiveStyle::with_fill(self.background_color))
            .draw(display)?;
            hero_entity.draw(display)?;
        }

        // Clear and draw enemy entity area (with 30px padding, +20px on top for floating damage)
        if let Some(ref enemy_entity) = self.enemy_entity {
            let (x, y, w, h) = enemy_entity.bounds();
            let padding = 30i32;
            let top_padding = 50i32; // Extra 20px on top for floating damage numbers
            let clear_x = (x - padding).max(0);
            let clear_y = (y - top_padding).max(0);
            let clear_w = (w as i32 + padding * 2) as u32;
            let clear_h = (h as i32 + padding + top_padding) as u32;
            Rectangle::new(
                Point::new(clear_x, clear_y),
                Size::new(clear_w, clear_h),
            )
            .into_styled(PrimitiveStyle::with_fill(self.background_color))
            .draw(display)?;
            enemy_entity.draw(display)?;
        }

        // Draw HP bars (compact, positioned within 368px visible area)
        self.draw_hp_bar(
            display,
            self.hero.current_health,
            self.hero.max_health,
            10, 15,
            140, 12,
            "HERO",
        )?;

        if let Some(ref enemy) = self.game_enemy {
            self.draw_hp_bar(
                display,
                enemy.current_hp as i32,
                enemy.max_hp as i32,
                215, 15,  // Right side
                140, 12,
                &enemy.name,
            )?;
        }

        // Draw skill buttons
        for button in &self.skill_buttons {
            self.draw_skill_button(display, button)?;
        }

        // Attack button removed - auto-attack mode

        // Draw turn indicator
        self.draw_turn_indicator(display)?;

        // Draw damage numbers
        for dn in &self.damage_numbers {
            dn.draw(display)?;
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering semi-active battle page");
        if let Err(e) = self.initialize() {
            log::error!("Failed to initialize battle: {}", e);
        }
    }

    fn on_exit(&mut self) {
        log::info!("Exiting semi-active battle page");
    }

    fn needs_clear(&self) -> bool {
        true
    }

    fn mark_dirty(&mut self) {
        self.needs_redraw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.first_draw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
