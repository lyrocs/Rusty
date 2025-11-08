//! Battle Page
//!
//! Displays a map background with animated characters in battle.

use crate::display::Sh8601Driver;
use crate::game::{self, Enemy as GameEnemy, EnemyType as GameEnemyType, Hero, KillTracker};
use crate::ui::page::Page;
use crate::ui::sprite::{AnimatedSprite, Background};
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10, FONT_9X15, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
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
    last_attack_time: Instant,
    attack_interval: Duration,
    is_dead: bool,
    death_time: Option<Instant>,
    attack_offset: (i32, i32), // Position offset for attack animation
    attack_damage_dealt: bool, // Track if damage has been dealt for current attack
}

impl BattleEntity {
    /// Create a new battle entity
    pub fn new(
        role: EntityRole,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        death_data: Option<&[u8]>,
        position: (i32, i32),
        attack_interval: Duration,
        attack_offset: (i32, i32),
    ) -> Result<Self, Box<dyn Error>> {
        let frame_delay = Duration::from_millis(100);

        let idle_sprite = AnimatedSprite::new(idle_data, position, frame_delay, None)?;

        // Apply offset to attack animation position
        let attack_position = (position.0 + attack_offset.0, position.1 + attack_offset.1);
        let attack_sprite =
            AnimatedSprite::new(attack_data, attack_position, frame_delay, Some(1))?;

        let attacked_sprite = AnimatedSprite::new(attacked_data, position, frame_delay, Some(1))?;
        let death_sprite = death_data
            .map(|data| AnimatedSprite::new(data, position, frame_delay, Some(1)))
            .transpose()?;

        Ok(Self {
            idle_sprite,
            attack_sprite,
            attacked_sprite,
            death_sprite,
            current_animation: AnimationType::Idle,
            role,
            last_attack_time: Instant::now(),
            attack_interval,
            is_dead: false,
            death_time: None,
            attack_offset,
            attack_damage_dealt: false,
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
            // Reset the new animation's sprite
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

    /// Get bounding box
    fn bounds(&self) -> (i32, i32, u32, u32) {
        self.current_sprite().bounds()
    }

    /// Check if it's time to attack
    fn should_attack(&self) -> bool {
        !self.is_dead && self.last_attack_time.elapsed() >= self.attack_interval
    }

    /// Trigger attack
    fn start_attack(&mut self) {
        self.set_animation(AnimationType::Attack);
        self.last_attack_time = Instant::now();
        self.attack_damage_dealt = false; // Reset damage flag for new attack
    }

    /// Check if attack animation has reached the hit point (50% progress)
    fn is_attack_hit_point(&self) -> bool {
        if self.current_animation == AnimationType::Attack {
            let sprite = self.current_sprite();
            let total_frames = sprite.frame_count();
            let current_frame = sprite.current_frame_index();

            // Damage lands at 50% through the attack animation
            current_frame >= total_frames / 2 && !self.attack_damage_dealt
        } else {
            false
        }
    }

    /// Mark that damage has been dealt for current attack
    fn mark_damage_dealt(&mut self) {
        self.attack_damage_dealt = true;
    }

    /// Trigger being attacked
    fn start_attacked(&mut self) {
        if !self.is_dead {
            self.set_animation(AnimationType::Attacked);
        }
    }

    /// Trigger death
    fn start_death(&mut self) {
        self.is_dead = true;
        self.death_time = Some(Instant::now());
        self.set_animation(AnimationType::Death);
    }

    /// Check if death animation is complete and waiting period is over
    fn is_death_complete(&self) -> bool {
        if let Some(death_time) = self.death_time {
            death_time.elapsed() >= Duration::from_secs(2) && self.is_animation_complete()
        } else {
            false
        }
    }
}

/// Enemy type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    Hornet,
    Poring,
    Fabre,
}

/// Floating damage number animation
#[derive(Debug, Clone)]
struct DamageNumber {
    value: u32,
    position: (i32, i32),
    start_time: Instant,
    duration: Duration,
    is_critical: bool,
    is_miss: bool,
}

impl DamageNumber {
    fn new(value: u32, position: (i32, i32), is_critical: bool, is_miss: bool) -> Self {
        Self {
            value,
            position,
            start_time: Instant::now(),
            duration: Duration::from_millis(800), // Animation lasts 800ms
            is_critical,
            is_miss,
        }
    }

    /// Check if animation is complete
    fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    /// Get current position (floats upward)
    fn current_position(&self) -> (i32, i32) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let progress = (elapsed / self.duration.as_secs_f32()).min(1.0);
        let offset_y = (progress * 50.0) as i32; // Float up 50 pixels

        (self.position.0, self.position.1 - offset_y)
    }

    /// Get current opacity (0-255)
    fn current_opacity(&self) -> u8 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let progress = (elapsed / self.duration.as_secs_f32()).min(1.0);
        let opacity = 1.0 - progress;
        (opacity * 255.0) as u8
    }
}

/// Battle page showing map and battle entities
pub struct BattlePage {
    background: Option<Background>,
    background_color: Rgb888,
    hero: Option<BattleEntity>,
    enemy: Option<BattleEntity>,
    fps: f32,
    first_draw: bool,
    enemy_types: Vec<EnemyType>,
    current_enemy_index: usize,

    // RPG game state
    game_hero: Hero,
    game_enemy: Option<GameEnemy>,
    kill_tracker: KillTracker,

    // Damage number animations
    damage_numbers: Vec<DamageNumber>,
}

impl BattlePage {
    /// Create a new battle page with a solid color background
    ///
    /// # Arguments
    /// * `background_color` - RGB color for background
    pub fn new(background_color: Rgb888) -> Self {
        Self {
            background: None,
            background_color,
            hero: None,
            enemy: None,
            fps: 0.0,
            first_draw: true,
            enemy_types: Vec::new(),
            current_enemy_index: 0,
            game_hero: Hero::new(),
            game_enemy: None,
            kill_tracker: KillTracker::new(),
            damage_numbers: Vec::new(),
        }
    }

    /// Create a new battle page with a GIF background (memory intensive!)
    ///
    /// # Arguments
    /// * `map_data` - GIF data for the background map
    /// * `map_position` - Position of the map
    #[allow(dead_code)]
    pub fn new_with_background(
        map_data: &[u8],
        map_position: (i32, i32),
    ) -> Result<Self, Box<dyn Error>> {
        let background = Background::new(map_data, map_position)?;

        Ok(Self {
            background: Some(background),
            background_color: Rgb888::BLACK,
            hero: None,
            enemy: None,
            fps: 0.0,
            first_draw: true,
            enemy_types: Vec::new(),
            current_enemy_index: 0,
            game_hero: Hero::new(),
            game_enemy: None,
            kill_tracker: KillTracker::new(),
            damage_numbers: Vec::new(),
        })
    }

    /// Convert UI EnemyType to game EnemyType
    fn to_game_enemy_type(enemy_type: EnemyType) -> GameEnemyType {
        match enemy_type {
            EnemyType::Hornet => GameEnemyType::Hornet,
            EnemyType::Poring => GameEnemyType::Poring,
            EnemyType::Fabre => GameEnemyType::Fabre,
        }
    }

    /// Get enemy GIF data by type
    /// Returns (idle, attack, attacked, death)
    fn get_enemy_data(
        enemy_type: EnemyType,
    ) -> (
        &'static [u8],
        &'static [u8],
        &'static [u8],
        Option<&'static [u8]>,
    ) {
        match enemy_type {
            EnemyType::Hornet => (
                include_bytes!("../../../assets/images/hornet/6.gif"), // idle
                include_bytes!("../../../assets/images/hornet/22.gif"), // attack
                include_bytes!("../../../assets/images/hornet/30.gif"), // attacked
                Some(include_bytes!("../../../assets/images/hornet/38.gif")), // death
            ),
            EnemyType::Poring => (
                include_bytes!("../../../assets/images/poring/6.gif"), // idle
                include_bytes!("../../../assets/images/poring/22.gif"), // attack
                include_bytes!("../../../assets/images/poring/30.gif"), // attacked
                None, // death - 38.gif is 341x336 (too large!), using fade out instead
            ),
            EnemyType::Fabre => (
                include_bytes!("../../../assets/images/fabre/6.gif"), // idle
                include_bytes!("../../../assets/images/fabre/22.gif"), // attack
                include_bytes!("../../../assets/images/fabre/30.gif"), // attacked
                Some(include_bytes!("../../../assets/images/fabre/38.gif")), // death
            ),
        }
    }

    /// Add hero to the battle
    ///
    /// # Arguments
    /// * `idle_data` - GIF for idle/standing animation
    /// * `attack_data` - GIF for attack animation
    /// * `attacked_data` - GIF for being attacked animation
    /// * `position` - Position on screen
    pub fn add_hero(
        &mut self,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        let hero = BattleEntity::new(
            EntityRole::Hero,
            idle_data,
            attack_data,
            attacked_data,
            None, // Heroes don't die in this version
            position,
            Duration::from_secs(2), // Hero attacks every 2 seconds
            (-40, 10),              // Move attack animation 20px left for Novice
        )?;

        self.hero = Some(hero);
        Ok(())
    }

    /// Add enemy to the battle by type
    ///
    /// # Arguments
    /// * `enemy_type` - Type of enemy to add
    /// * `position` - Position on screen
    pub fn add_enemy(
        &mut self,
        enemy_type: EnemyType,
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        // Get enemy data based on type
        let (idle_data, attack_data, attacked_data, death_data) = Self::get_enemy_data(enemy_type);

        let enemy = BattleEntity::new(
            EntityRole::Enemy,
            idle_data,
            attack_data,
            attacked_data,
            death_data,
            position,
            Duration::from_secs(3), // Enemy attacks every 3 seconds
            (0, 0),                 // No offset for enemies
        )?;

        // Create game enemy with RPG stats
        let game_enemy_type = Self::to_game_enemy_type(enemy_type);
        let game_enemy = GameEnemy::new(game_enemy_type, self.game_hero.level);
        log::info!(
            "Spawned {} (Lv {}, HP: {}, ATK: {})",
            game_enemy.enemy_type.name(),
            game_enemy.level,
            game_enemy.max_hp,
            game_enemy.atk
        );

        // Store enemy type for respawning
        if self.enemy_types.is_empty() {
            self.enemy_types.push(enemy_type);
        }

        self.enemy = Some(enemy);
        self.game_enemy = Some(game_enemy);
        Ok(())
    }

    /// Add enemy type to respawn pool (for cycling through different enemies)
    pub fn add_enemy_type_to_pool(&mut self, enemy_type: EnemyType) {
        self.enemy_types.push(enemy_type);
    }

    /// Respawn enemy with next one in pool
    fn respawn_enemy(&mut self) -> Result<(), Box<dyn Error>> {
        if self.enemy_types.is_empty() {
            return Ok(());
        }

        // Drop old enemy to free memory
        self.enemy = None;
        self.game_enemy = None;

        // Cycle to next enemy type
        self.current_enemy_index = (self.current_enemy_index + 1) % self.enemy_types.len();
        let enemy_type = self.enemy_types[self.current_enemy_index];

        log::info!("Respawning enemy: {:?}", enemy_type);

        // Calculate left-centered position for enemy
        const DISPLAY_WIDTH: i32 = 368;
        const DISPLAY_HEIGHT: i32 = 448;
        const HALF_WIDTH: i32 = DISPLAY_WIDTH / 2;

        // Position enemy on left side
        let x = HALF_WIDTH / 2;
        let y = DISPLAY_HEIGHT / 2;

        // Load enemy data on-demand
        let (idle, attack, attacked, death) = Self::get_enemy_data(enemy_type);

        let enemy = BattleEntity::new(
            EntityRole::Enemy,
            idle,
            attack,
            attacked,
            death,
            (x, y),
            Duration::from_secs(3),
            (0, 0), // No offset for enemies
        )?;

        // Create game enemy with RPG stats
        let game_enemy_type = Self::to_game_enemy_type(enemy_type);
        let game_enemy = GameEnemy::new(game_enemy_type, self.game_hero.level);
        log::info!(
            "Respawned {} (Lv {}, HP: {}, ATK: {})",
            game_enemy.enemy_type.name(),
            game_enemy.level,
            game_enemy.max_hp,
            game_enemy.atk
        );

        self.enemy = Some(enemy);
        self.game_enemy = Some(game_enemy);
        Ok(())
    }

    /// Set FPS for display
    pub fn set_fps(&mut self, fps: f32) {
        self.fps = fps;
    }

    /// Draw FPS overlay (without flushing)
    fn draw_fps_overlay(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Draw semi-transparent background box for FPS
        Rectangle::new(Point::new(5, 2), Size::new(70, 15))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Draw FPS text
        let mut fps_str = heapless::String::<16>::new();
        write!(fps_str, "FPS: {:.1}", self.fps).ok();

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
        Text::new(&fps_str, Point::new(10, 10), text_style).draw(display)?;

        Ok(())
    }

    /// Draw HP bar
    fn draw_hp_bar(
        &self,
        display: &mut Sh8601Driver,
        position: (i32, i32),
        current_hp: u32,
        max_hp: u32,
        width: u32,
    ) -> Result<(), Box<dyn Error>> {
        let height = 6;
        let (x, y) = position;

        // Calculate HP percentage
        let hp_percent = (current_hp as f32 / max_hp as f32).clamp(0.0, 1.0);
        let filled_width = (width as f32 * hp_percent) as u32;

        // Draw background (black)
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 20, 20)))
            .draw(display)?;

        // Draw HP fill (color based on percentage)
        let hp_color = if hp_percent > 0.6 {
            Rgb888::new(0, 255, 0) // Green
        } else if hp_percent > 0.3 {
            Rgb888::new(255, 255, 0) // Yellow
        } else {
            Rgb888::new(255, 0, 0) // Red
        };

        if filled_width > 0 {
            Rectangle::new(Point::new(x, y), Size::new(filled_width, height))
                .into_styled(PrimitiveStyle::with_fill(hp_color))
                .draw(display)?;
        }

        // Draw border (white)
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .draw(display)?;

        Ok(())
    }

    /// Draw SP bar (for hero only)
    fn draw_sp_bar(
        &self,
        display: &mut Sh8601Driver,
        position: (i32, i32),
        current_sp: u32,
        max_sp: u32,
        width: u32,
    ) -> Result<(), Box<dyn Error>> {
        let height = 4;
        let (x, y) = position;

        // Calculate SP percentage
        let sp_percent = (current_sp as f32 / max_sp as f32).clamp(0.0, 1.0);
        let filled_width = (width as f32 * sp_percent) as u32;

        // Draw background (black)
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 20, 20)))
            .draw(display)?;

        // Draw SP fill (blue)
        if filled_width > 0 {
            Rectangle::new(Point::new(x, y), Size::new(filled_width, height))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 100, 255)))
                .draw(display)?;
        }

        // Draw border (cyan)
        Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(0, 200, 255), 1))
            .draw(display)?;

        Ok(())
    }

    /// Draw top info panel with monster and hero information
    fn draw_top_info_panel(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Draw dark background panel at top
        let panel_height = 60;
        Rectangle::new(Point::new(0, 0), Size::new(368, panel_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 20, 30)))
            .draw(display)?;

        let text_style_name = MonoTextStyle::new(&FONT_6X10, Rgb888::new(255, 255, 200));
        let text_style_info = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));

        // LEFT SIDE - MONSTER INFO
        if let Some(game_enemy) = &self.game_enemy {
            let left_x = 25;
            let name_y = 12;

            // Monster name
            let mut name_str = heapless::String::<32>::new();
            write!(name_str, "{}", game_enemy.enemy_type.name()).ok();
            Text::new(&name_str, Point::new(left_x, name_y), text_style_name).draw(display)?;

            // Monster level
            let mut lvl_str = heapless::String::<16>::new();
            write!(lvl_str, "Lv {}", game_enemy.level).ok();
            Text::new(&lvl_str, Point::new(left_x, name_y + 12), text_style_info).draw(display)?;

            // Monster HP bar
            let hp_bar_y = 30;
            let hp_bar_width = 100;
            self.draw_hp_bar(
                display,
                (left_x, hp_bar_y),
                game_enemy.current_hp,
                game_enemy.max_hp,
                hp_bar_width,
            )?;

            // HP text
            let mut hp_str = heapless::String::<32>::new();
            write!(hp_str, "{}/{}", game_enemy.current_hp, game_enemy.max_hp).ok();
            Text::new(&hp_str, Point::new(left_x, hp_bar_y + 15), text_style_info).draw(display)?;
        }

        // RIGHT SIDE - HERO INFO
        let right_x = 368 - 140; // Right aligned with some margin
        let name_y = 12;

        // Hero job/name
        let mut name_str = heapless::String::<32>::new();
        write!(name_str, "{}", self.game_hero.job.name()).ok();
        Text::new(&name_str, Point::new(right_x, name_y), text_style_name).draw(display)?;

        // Hero level
        let mut lvl_str = heapless::String::<16>::new();
        write!(lvl_str, "Lv {}", self.game_hero.level).ok();
        Text::new(&lvl_str, Point::new(right_x, name_y + 12), text_style_info).draw(display)?;

        // Hero HP bar
        let hp_bar_y = 30;
        let hp_bar_width = 100;
        self.draw_hp_bar(
            display,
            (right_x, hp_bar_y),
            self.game_hero.current_hp,
            self.game_hero.max_hp,
            hp_bar_width,
        )?;

        // Hero SP bar
        let sp_bar_y = hp_bar_y + 8;
        self.draw_sp_bar(
            display,
            (right_x, sp_bar_y),
            self.game_hero.current_sp,
            self.game_hero.max_sp,
            hp_bar_width,
        )?;

        // HP/SP text
        let mut hp_str = heapless::String::<32>::new();
        write!(
            hp_str,
            "HP:{}/{}",
            self.game_hero.current_hp, self.game_hero.max_hp
        )
        .ok();
        Text::new(&hp_str, Point::new(right_x, sp_bar_y + 14), text_style_info).draw(display)?;

        let mut sp_str = heapless::String::<32>::new();
        write!(
            sp_str,
            "SP:{}/{}",
            self.game_hero.current_sp, self.game_hero.max_sp
        )
        .ok();
        Text::new(&sp_str, Point::new(right_x, sp_bar_y + 24), text_style_info).draw(display)?;

        Ok(())
    }

    /// Draw floating damage numbers
    fn draw_damage_numbers(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        for dmg in &self.damage_numbers {
            let (x, y) = dmg.current_position();
            let opacity = dmg.current_opacity();

            // Skip if nearly invisible
            if opacity < 20 {
                continue;
            }

            // Choose color based on damage type
            let color = if dmg.is_miss {
                Rgb888::new(20, 20, 20) // Light gray for miss
            } else if dmg.is_critical {
                Rgb888::new(255, 200, 0) // Yellow-orange for critical
            } else {
                Rgb888::BLACK // White for normal damage
            };

            // Format damage text
            let mut dmg_str = heapless::String::<16>::new();
            if dmg.is_miss {
                write!(dmg_str, "MISS").ok();
            } else {
                write!(dmg_str, "{}", dmg.value).ok();
            }

            // Draw damage number with large font (3x size)
            let text_style = MonoTextStyle::new(&FONT_10X20, color);
            Text::new(&dmg_str, Point::new(x, y), text_style).draw(display)?;
        }

        Ok(())
    }
}

impl Page for BattlePage {
    fn update(&mut self) -> bool {
        // Check if enemy is dead and should respawn
        if let Some(enemy) = &self.enemy {
            if enemy.is_dead && enemy.is_death_complete() {
                log::info!("Enemy death complete, respawning...");
                if let Err(e) = self.respawn_enemy() {
                    log::error!("Failed to respawn enemy: {:?}", e);
                }
            }
        }

        // Check for attacks and update animations
        let hero_attacking = self.hero.as_ref().map_or(false, |h| h.should_attack());
        let enemy_attacking = self.enemy.as_ref().map_or(false, |e| e.should_attack());

        // Handle hero attack
        if hero_attacking {
            if let Some(hero) = &mut self.hero {
                hero.start_attack();
                log::info!("Hero attacks!");
            }
        }

        // Handle enemy attack
        if enemy_attacking {
            if let Some(enemy) = &mut self.enemy {
                enemy.start_attack();
                log::info!("Enemy attacks!");
            }
        }

        // Update entity animations
        if let Some(hero) = &mut self.hero {
            hero.update();

            // Deal damage mid-attack animation (when hit lands)
            if hero.is_attack_hit_point() {
                if let (Some(enemy), Some(game_enemy)) = (&mut self.enemy, &mut self.game_enemy) {
                    if !enemy.is_dead {
                        // Calculate damage using RPG system
                        let damage_result = game::hero_attack(&self.game_hero, game_enemy);

                        // Create floating damage number near enemy
                        let bounds = enemy.bounds();
                        let damage_pos = (
                            bounds.0 + (bounds.2 / 2) as i32, // Center of sprite
                            bounds.1 + 10,                    // Slightly below top
                        );
                        let damage_num = DamageNumber::new(
                            damage_result.damage,
                            damage_pos,
                            damage_result.is_critical,
                            damage_result.is_miss,
                        );
                        self.damage_numbers.push(damage_num);

                        // Mark damage as dealt
                        hero.mark_damage_dealt();

                        // Check if enemy died from the attack
                        if !game_enemy.is_alive() {
                            // Enemy died - show death animation
                            enemy.start_death();

                            // Award EXP and record kill
                            let exp_reward = game_enemy.exp_reward;
                            let enemy_type = game_enemy.enemy_type;
                            self.game_hero.gain_exp(exp_reward);
                            self.kill_tracker.record_kill(enemy_type);

                            log::info!(
                                "{} defeated! Gained {} EXP (Hero: Lv {} - {}/{})",
                                enemy_type.name(),
                                exp_reward,
                                self.game_hero.level,
                                self.game_hero.exp,
                                self.game_hero.exp_to_next_level
                            );
                        } else {
                            // Enemy survived - show attacked animation
                            enemy.start_attacked();
                        }
                    }
                }
            }

            // Return to idle after attack animation completes
            if hero.current_animation == AnimationType::Attack && hero.is_animation_complete() {
                hero.set_animation(AnimationType::Idle);
            }

            // Return to idle after being attacked
            if hero.current_animation == AnimationType::Attacked && hero.is_animation_complete() {
                hero.set_animation(AnimationType::Idle);
            }
        }

        if let Some(enemy) = &mut self.enemy {
            enemy.update();

            // Deal damage mid-attack animation (when hit lands)
            if enemy.is_attack_hit_point() {
                if let Some(game_enemy) = &self.game_enemy {
                    // Calculate damage using RPG system
                    let damage_result = game::enemy_attack(game_enemy, &mut self.game_hero);

                    // Create floating damage number near hero
                    if let Some(hero) = &mut self.hero {
                        let bounds = hero.bounds();
                        let damage_pos = (
                            bounds.0 + (bounds.2 / 2) as i32, // Center of sprite
                            bounds.1 + 10,                    // Slightly below top
                        );
                        let damage_num = DamageNumber::new(
                            damage_result.damage,
                            damage_pos,
                            damage_result.is_critical,
                            damage_result.is_miss,
                        );
                        self.damage_numbers.push(damage_num);

                        // Show attacked animation on hero
                        hero.start_attacked();
                    }

                    // Mark damage as dealt
                    enemy.mark_damage_dealt();

                    // Check if hero died (game over logic can be added later)
                    if !self.game_hero.is_alive() {
                        log::warn!("Hero defeated! (Game over logic not implemented)");
                        // For now, just restore hero HP
                        self.game_hero.current_hp = self.game_hero.max_hp / 2;
                    }
                }
            }

            // Return to idle after attack animation completes
            if enemy.current_animation == AnimationType::Attack && enemy.is_animation_complete() {
                enemy.set_animation(AnimationType::Idle);
            }

            // Return to idle after being attacked
            if enemy.current_animation == AnimationType::Attacked && enemy.is_animation_complete() {
                if !enemy.is_dead {
                    enemy.set_animation(AnimationType::Idle);
                }
            }
        }

        // Remove completed damage number animations
        self.damage_numbers.retain(|dmg| !dmg.is_complete());

        // Continue running
        true
    }

    fn draw(
        &mut self,
        display: &mut Sh8601Driver,
        full_redraw: bool,
    ) -> Result<(), Box<dyn Error>> {
        use embedded_graphics::prelude::*;
        use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};

        // Draw background
        if full_redraw {
            if let Some(background) = &self.background {
                // Draw GIF background
                background.draw(display)?;
            } else {
                // Draw solid color background
                display.clear(self.background_color)?;
            }
        } else {
            // For subsequent frames, only clear and redraw entity zones
            // Add padding to fully clean sprite area (extra 20px from top)
            const PADDING: i32 = 20;
            const TOP_PADDING: i32 = 50; // Extra padding from top (20 + 20)

            if let Some(hero) = &self.hero {
                let bounds = hero.bounds();
                let padded_bounds = (
                    bounds.0 - PADDING,
                    bounds.1 - TOP_PADDING, // Extra padding from top
                    bounds.2 + (PADDING * 2) as u32,
                    bounds.3 + (PADDING + TOP_PADDING) as u32, // Total top + bottom padding
                );

                if let Some(background) = &self.background {
                    background.draw_region(display, padded_bounds)?;
                } else {
                    // Clear with solid color
                    let rect = Rectangle::new(
                        Point::new(padded_bounds.0, padded_bounds.1),
                        Size::new(padded_bounds.2, padded_bounds.3),
                    );
                    rect.into_styled(
                        PrimitiveStyleBuilder::new()
                            .fill_color(self.background_color)
                            .build(),
                    )
                    .draw(display)?;
                }
            }

            if let Some(enemy) = &self.enemy {
                let bounds = enemy.bounds();
                let padded_bounds = (
                    bounds.0 - PADDING,
                    bounds.1 - TOP_PADDING, // Extra padding from top
                    bounds.2 + (PADDING * 2) as u32,
                    bounds.3 + (PADDING + TOP_PADDING) as u32, // Total top + bottom padding
                );

                if let Some(background) = &self.background {
                    background.draw_region(display, padded_bounds)?;
                } else {
                    // Clear with solid color
                    let rect = Rectangle::new(
                        Point::new(padded_bounds.0, padded_bounds.1),
                        Size::new(padded_bounds.2, padded_bounds.3),
                    );
                    rect.into_styled(
                        PrimitiveStyleBuilder::new()
                            .fill_color(self.background_color)
                            .build(),
                    )
                    .draw(display)?;
                }
            }
        }

        // Draw entities
        if let Some(hero) = &self.hero {
            hero.draw(display)?;
        }

        if let Some(enemy) = &self.enemy {
            enemy.draw(display)?;
        }

        // Draw floating damage numbers
        self.draw_damage_numbers(display)?;

        // Draw top info panel with monster and hero information
        self.draw_top_info_panel(display)?;

        // Draw FPS overlay (no flush)
        self.draw_fps_overlay(display)?;

        // Flush to display once at the end
        display.flush()?;

        // Mark that we've done the first draw
        if self.first_draw {
            self.first_draw = false;
        }

        Ok(())
    }

    fn on_enter(&mut self) {
        log::info!("Entering battle page");
        self.first_draw = true; // Force full redraw when entering
    }

    fn on_exit(&mut self) {
        log::info!("Exiting battle page");
    }

    fn mark_dirty(&mut self) {
        self.first_draw = true;
    }

    fn needs_full_redraw(&self) -> bool {
        self.first_draw
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
