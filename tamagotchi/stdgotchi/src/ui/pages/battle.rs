//! Battle Page
//!
//! Displays a map background with animated characters in battle.

use crate::assets::battle::{load_enemy_sprites, load_enemy_sprites_embedded};
use crate::assets::AssetLoader;
use crate::display::Sh8601Driver;
use crate::ecs::resources::SdCardWrapper;
use crate::game::{self, Enemy as GameEnemy, FragmentCollection, GameData, Hero, KillTracker, Rustymon, RustymonTeam};
use crate::game::battle::DamageResult;
use crate::ui::page::Page;
use crate::ui::sprite::{AnimatedSprite, Background};
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

        if self.death_sprite.is_some() {
            log::info!("Entity death started with animation");
        } else {
            log::info!("Entity death started without animation (2 second delay)");
        }
    }

    /// Check if death animation is complete and waiting period is over
    fn is_death_complete(&self) -> bool {
        if let Some(death_time) = self.death_time {
            // If there's a death animation, wait for both time and animation to complete
            // If no death animation (falls back to idle), just wait for the time
            if self.death_sprite.is_some() {
                death_time.elapsed() >= Duration::from_secs(2) && self.is_animation_complete()
            } else {
                // No death animation - just wait 2 seconds
                death_time.elapsed() >= Duration::from_secs(2)
            }
        } else {
            false
        }
    }
}

// Enemy types now use numeric IDs from GameData:
// 1002 - Poring
// 1004 - Hornet
// 1007 - Fabre
// 1051 - Thief Bug

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

/// Actions from battle page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleAction {
    SwitchRustymon(usize), // Switch to team slot index
}

/// Touch area for battle interactions
#[derive(Debug, Clone)]
struct TouchArea {
    bounds: (i32, i32, u32, u32), // (x, y, width, height)
    action: BattleAction,
}

impl TouchArea {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.bounds.0
            && x < self.bounds.0 + self.bounds.2 as i32
            && y >= self.bounds.1
            && y < self.bounds.1 + self.bounds.3 as i32
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
    enemy_ids: Vec<u32>,         // Enemy IDs to spawn
    current_enemy_index: usize,

    // RPG game state
    game_hero: Hero,              // Keep hero for now (future: may be removed)
    game_enemy: Option<GameEnemy>,
    kill_tracker: KillTracker,
    game_data: GameData,          // Game data for enemy loading

    // Rustymon system
    rustymon_collection: Vec<Rustymon>, // All owned Rustymon
    rustymon_team: RustymonTeam,        // Active team and bank
    fragment_collection: FragmentCollection, // Current fragment counts

    // Damage number animations
    damage_numbers: Vec<DamageNumber>,

    // HP regeneration
    last_hp_regen: Instant,

    // Hero death flag (kept for compatibility)
    hero_died: bool,

    // SD card asset loading
    asset_loader: Option<AssetLoader<SdCardWrapper>>,

    // Fragment drops during this battle session
    fragment_drops: Vec<(u32, String)>, // (enemy_id, enemy_name)
    fragment_notification: Option<(String, Instant)>, // (message, timestamp)

    // Touch interaction
    touch_areas: Vec<TouchArea>,
}

impl BattlePage {
    /// Create a new battle page with a solid color background
    ///
    /// # Arguments
    /// * `background_color` - RGB color for background
    /// * `hero` - Hero state from GameManager
    /// * `kill_tracker` - Kill tracker from GameManager
    /// * `game_data` - Game data for enemy loading
    /// * `rustymon_collection` - All owned Rustymon
    /// * `rustymon_team` - Active team and bank
    /// * `fragment_collection` - Current fragment counts
    /// * `asset_loader` - Optional AssetLoader for SD card support
    pub fn new(
        background_color: Rgb888,
        hero: Hero,
        kill_tracker: KillTracker,
        game_data: GameData,
        rustymon_collection: Vec<Rustymon>,
        rustymon_team: RustymonTeam,
        fragment_collection: FragmentCollection,
        asset_loader: Option<AssetLoader<SdCardWrapper>>,
    ) -> Self {
        Self {
            background: None,
            background_color,
            hero: None,
            enemy: None,
            fps: 0.0,
            first_draw: true,
            enemy_ids: Vec::new(),
            current_enemy_index: 0,
            game_hero: hero,
            game_enemy: None,
            kill_tracker,
            game_data,
            rustymon_collection,
            rustymon_team,
            fragment_collection,
            damage_numbers: Vec::new(),
            last_hp_regen: Instant::now(),
            hero_died: false,
            asset_loader,
            fragment_drops: Vec::new(),
            fragment_notification: None,
            touch_areas: Vec::new(),
        }
    }

    /// Create a new battle page with a GIF background (memory intensive!)
    ///
    /// # Arguments
    /// * `map_data` - GIF data for the background map
    /// * `map_position` - Position of the map
    /// * `hero` - Hero state from GameManager
    /// * `kill_tracker` - Kill tracker from GameManager
    /// * `game_data` - Game data for enemy loading
    /// * `rustymon_collection` - All owned Rustymon
    /// * `rustymon_team` - Active team and bank
    /// * `fragment_collection` - Current fragment counts
    /// * `asset_loader` - Optional AssetLoader for SD card support
    #[allow(dead_code)]
    pub fn new_with_background(
        map_data: &[u8],
        map_position: (i32, i32),
        hero: Hero,
        kill_tracker: KillTracker,
        game_data: GameData,
        rustymon_collection: Vec<Rustymon>,
        rustymon_team: RustymonTeam,
        fragment_collection: FragmentCollection,
        asset_loader: Option<AssetLoader<SdCardWrapper>>,
    ) -> Result<Self, Box<dyn Error>> {
        let background = Background::new(map_data, map_position)?;

        Ok(Self {
            background: Some(background),
            background_color: Rgb888::BLACK,
            hero: None,
            enemy: None,
            fps: 0.0,
            first_draw: true,
            enemy_ids: Vec::new(),
            current_enemy_index: 0,
            game_hero: hero,
            game_enemy: None,
            kill_tracker,
            game_data,
            rustymon_collection,
            rustymon_team,
            fragment_collection,
            damage_numbers: Vec::new(),
            last_hp_regen: Instant::now(),
            hero_died: false,
            asset_loader,
            fragment_drops: Vec::new(),
            fragment_notification: None,
            touch_areas: Vec::new(),
        })
    }

    /// Get the updated hero state (to sync back to GameManager)
    pub fn get_hero(&self) -> &Hero {
        &self.game_hero
    }

    /// Get the updated kill tracker (to sync back to GameManager)
    pub fn get_kill_tracker(&self) -> &KillTracker {
        &self.kill_tracker
    }

    /// Get the updated Rustymon collection (to sync back to GameManager)
    pub fn get_rustymon_collection(&self) -> &Vec<Rustymon> {
        &self.rustymon_collection
    }

    /// Get the updated Rustymon team (to sync back to GameManager)
    pub fn get_rustymon_team(&self) -> &RustymonTeam {
        &self.rustymon_team
    }

    /// Get the currently active Rustymon (if any)
    fn get_active_rustymon(&self) -> Option<&Rustymon> {
        let active_id = self.rustymon_team.get_active_rustymon_id()?;
        self.rustymon_collection
            .iter()
            .find(|r| &r.id == active_id)
    }

    /// Get the currently active Rustymon (mutable, if any)
    fn get_active_rustymon_mut(&mut self) -> Option<&mut Rustymon> {
        let active_id = self.rustymon_team.get_active_rustymon_id()?.clone();
        self.rustymon_collection
            .iter_mut()
            .find(|r| r.id == active_id)
    }

    /// Get and clear fragment drops from this battle session
    /// Returns list of (enemy_id, enemy_name) that dropped fragments
    pub fn take_fragment_drops(&mut self) -> Vec<(u32, String)> {
        std::mem::take(&mut self.fragment_drops)
    }

    /// Check if hero died
    pub fn hero_died(&self) -> bool {
        self.hero_died
    }

    /// Handle touch input on battle screen
    pub fn handle_touch(&mut self, x: i32, y: i32) -> Option<BattleAction> {
        for area in &self.touch_areas {
            if area.contains(x, y) {
                return Some(area.action);
            }
        }
        None
    }

    /// Switch to a different Rustymon from the team
    pub fn switch_rustymon(&mut self, team_slot: usize) -> Result<(), Box<dyn Error>> {
        // Get the team rustymon IDs
        let team_rustymon_ids = self.rustymon_team.get_team_ids();

        if team_slot >= team_rustymon_ids.len() {
            log::warn!("Invalid team slot: {}", team_slot);
            return Ok(());
        }

        let new_rustymon_id = &team_rustymon_ids[team_slot];
        let active_id = self.rustymon_team.get_active_rustymon_id();

        // Don't switch if already active
        if active_id.map(|id| id == new_rustymon_id).unwrap_or(false) {
            log::info!("Rustymon already active");
            return Ok(());
        }

        // Find the Rustymon in collection
        let rustymon = self.rustymon_collection
            .iter()
            .find(|r| r.id == *new_rustymon_id);

        if let Some(rustymon) = rustymon {
            // Check if Rustymon is alive
            if rustymon.current_hp == 0 {
                log::warn!("Cannot switch to {} - HP is 0", rustymon.name);
                return Ok(());
            }

            log::info!("Switching to {} (Lv{})", rustymon.name, rustymon.level);

            // Set as active in team
            self.rustymon_team.set_active_rustymon(new_rustymon_id.clone());

            // Reload hero sprite with Rustymon sprite
            use crate::assets::battle::load_enemy_sprites_embedded;
            if let Some((idle, attack, attacked, _death)) = load_enemy_sprites_embedded(rustymon.species_id) {
                // Remove old hero
                self.hero = None;

                // Add new Rustymon sprite as hero
                let hero_result = BattleEntity::new(
                    EntityRole::Hero,
                    &idle,
                    &attack,
                    &attacked,
                    None,
                    (175, 170),
                    Duration::from_secs(2),
                    (0, 0), // Center attack for Rustymon
                );

                match hero_result {
                    Ok(hero) => {
                        self.hero = Some(hero);
                        log::info!("✅ Switched to {} successfully", rustymon.name);
                    }
                    Err(e) => {
                        log::error!("Failed to load Rustymon sprite: {:?}", e);
                    }
                }
            } else {
                log::error!("Failed to load sprites for species {}", rustymon.species_id);
            }
        } else {
            log::warn!("Rustymon {} not found in collection", new_rustymon_id);
        }

        Ok(())
    }

    /// Get enemy sprites with SD card support
    /// Tries to load from SD card first, falls back to embedded assets
    fn get_enemy_sprites_with_sd(
        &mut self,
        enemy_id: u32,
    ) -> (Vec<u8>, Vec<u8>, Vec<u8>, Option<Vec<u8>>) {
        // Try SD card first if available
        if let Some(ref mut loader) = self.asset_loader {
            match load_enemy_sprites(loader, enemy_id) {
                Ok(sprites) => {
                    log::info!("✅ Loaded enemy {} sprites from SD card", enemy_id);
                    return sprites;
                }
                Err(e) => {
                    log::warn!("⚠️  SD card load failed for enemy {}: {}", enemy_id, e);
                    log::info!("📦 Falling back to embedded sprites");
                }
            }
        }

        // Fallback to embedded
        log::info!("📦 Loading enemy {} sprites from embedded assets", enemy_id);
        load_enemy_sprites_embedded(enemy_id).unwrap_or_else(|| {
            log::error!("No embedded sprites for enemy {}", enemy_id);
            // Return poring sprites as fallback
            load_enemy_sprites_embedded(1002).unwrap()
        })
    }

    /// Add hero to the battle
    ///
    /// # Arguments
    /// * `idle_data` - GIF for idle/standing animation
    /// * `attack_data` - GIF for attack animation
    /// * `attacked_data` - GIF for being attacked animation
    /// * `position` - Position on screen
    /// * `attack_offset` - Position offset for attack animation
    pub fn add_hero(
        &mut self,
        idle_data: &[u8],
        attack_data: &[u8],
        attacked_data: &[u8],
        position: (i32, i32),
        attack_offset: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        let hero = BattleEntity::new(
            EntityRole::Hero,
            idle_data,
            attack_data,
            attacked_data,
            None, // Heroes don't die in this version
            position,
            Duration::from_secs(2), // Hero attacks every 2 seconds
            attack_offset,          // Job-specific attack animation offset
        )?;

        self.hero = Some(hero);
        Ok(())
    }

    /// Add enemy to the battle by ID
    ///
    /// # Arguments
    /// * `enemy_id` - ID of enemy to add
    /// * `position` - Position on screen
    pub fn add_enemy(
        &mut self,
        enemy_id: u32,
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        // Load sprites with SD card support
        let (idle_data, attack_data, attacked_data, death_data) =
            self.get_enemy_sprites_with_sd(enemy_id);

        let enemy = BattleEntity::new(
            EntityRole::Enemy,
            &idle_data,
            &attack_data,
            &attacked_data,
            death_data.as_deref(),
            position,
            Duration::from_secs(3), // Enemy attacks every 3 seconds
            (0, 0),                 // No offset for enemies
        )?;

        // Get enemy data from GameData and create game enemy
        if let Some(enemy_data) = self.game_data.get_enemy(enemy_id) {
            let game_enemy = GameEnemy::from_data(
                enemy_data.id,
                enemy_data.name.clone(),
                enemy_data.level,
                enemy_data.hp,
                enemy_data.attack,
                enemy_data.defense,
                enemy_data.base_exp,
                enemy_data.get_element(),
            );
            log::info!(
                "Spawned {} (Lv {}, HP: {}, ATK: {})",
                game_enemy.name,
                game_enemy.level,
                game_enemy.max_hp,
                game_enemy.atk
            );

            // Store enemy ID for respawning
            if self.enemy_ids.is_empty() {
                self.enemy_ids.push(enemy_id);
            }

            self.enemy = Some(enemy);
            self.game_enemy = Some(game_enemy);
            Ok(())
        } else {
            Err(format!("Enemy {} not found in game data", enemy_id).into())
        }
    }

    /// Add enemy ID to respawn pool (for cycling through different enemies)
    pub fn add_enemy_id_to_pool(&mut self, enemy_id: u32) {
        self.enemy_ids.push(enemy_id);
    }

    /// Respawn enemy with next one in pool
    fn respawn_enemy(&mut self) -> Result<(), Box<dyn Error>> {
        if self.enemy_ids.is_empty() {
            return Ok(());
        }

        // Drop old enemy to free memory
        self.enemy = None;
        self.game_enemy = None;

        // Cycle to next enemy ID
        self.current_enemy_index = (self.current_enemy_index + 1) % self.enemy_ids.len();
        let enemy_id = self.enemy_ids[self.current_enemy_index];

        log::info!("Respawning enemy ID: {}", enemy_id);

        // Position enemy on left side (matching initial battle setup)
        let x = 75;
        let y = 170;

        // Load sprites with SD card support
        let (idle_data, attack_data, attacked_data, death_data) =
            self.get_enemy_sprites_with_sd(enemy_id);

        let enemy = BattleEntity::new(
            EntityRole::Enemy,
            &idle_data,
            &attack_data,
            &attacked_data,
            death_data.as_deref(),
            (x, y),
            Duration::from_secs(3),
            (0, 0), // No offset for enemies
        )?;

        // Get enemy data and create game enemy
        let Some(enemy_data) = self.game_data.get_enemy(enemy_id) else {
            return Err(format!("Enemy {} not found in game data", enemy_id).into());
        };

        let game_enemy = GameEnemy::from_data(
            enemy_data.id,
            enemy_data.name.clone(),
            enemy_data.level,
            enemy_data.hp,
            enemy_data.attack,
            enemy_data.defense,
            enemy_data.base_exp,
            enemy_data.get_element(),
        );
        log::info!(
            "Respawned {} (Lv {}, HP: {}, ATK: {})",
            game_enemy.name,
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
        let panel_height = 70;
        Rectangle::new(Point::new(0, 0), Size::new(368, panel_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 20, 30)))
            .draw(display)?;

        let text_style_name = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 200));
        let text_style_info = MonoTextStyle::new(&FONT_10X20, Rgb888::new(180, 180, 180));

        // LEFT SIDE - MONSTER INFO
        if let Some(game_enemy) = &self.game_enemy {
            let left_x = 25;
            let name_y = 20;

            // Monster name
            let mut name_str = heapless::String::<32>::new();
            write!(name_str, "{}", game_enemy.name).ok();
            Text::new(&name_str, Point::new(left_x, name_y), text_style_name).draw(display)?;

            // Monster level
            let mut lvl_str = heapless::String::<16>::new();
            write!(lvl_str, "Lv {}", game_enemy.level).ok();
            Text::new(&lvl_str, Point::new(left_x, name_y + 20), text_style_info).draw(display)?;

            // Monster HP bar
            let hp_bar_y = 45;
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
            Text::new(&hp_str, Point::new(left_x, hp_bar_y + 20), text_style_info).draw(display)?;
        }

        // RIGHT SIDE - RUSTYMON or HERO INFO
        let right_x = 368 - 140; // Right aligned with some margin
        let name_y = 20;

        // Show Rustymon info if present, otherwise hero
        if let Some(rustymon) = self.get_active_rustymon() {
            // Rustymon name
            let mut name_str = heapless::String::<32>::new();
            write!(name_str, "{}", rustymon.name).ok();
            Text::new(&name_str, Point::new(right_x, name_y), text_style_name).draw(display)?;

            // Rustymon level and element
            let mut lvl_str = heapless::String::<24>::new();
            write!(lvl_str, "Lv {} {}", rustymon.level, rustymon.element.as_str()).ok();
            Text::new(&lvl_str, Point::new(right_x, name_y + 20), text_style_info).draw(display)?;

            // Rustymon HP bar
            let hp_bar_y = 45;
            let hp_bar_width = 100;
            self.draw_hp_bar(
                display,
                (right_x, hp_bar_y),
                rustymon.current_hp,
                rustymon.max_hp,
                hp_bar_width,
            )?;

            // HP text
            let mut hp_str = heapless::String::<32>::new();
            write!(
                hp_str,
                "HP:{}/{}",
                rustymon.current_hp, rustymon.max_hp
            )
            .ok();
            Text::new(&hp_str, Point::new(right_x, hp_bar_y + 20), text_style_info).draw(display)?;
        } else {
            // Hero job/name
            let mut name_str = heapless::String::<32>::new();
            write!(name_str, "{}", self.game_hero.job.name()).ok();
            Text::new(&name_str, Point::new(right_x, name_y), text_style_name).draw(display)?;

            // Hero level
            let mut lvl_str = heapless::String::<16>::new();
            write!(lvl_str, "Lv {}", self.game_hero.level).ok();
            Text::new(&lvl_str, Point::new(right_x, name_y + 20), text_style_info).draw(display)?;

            // Hero HP bar
            let hp_bar_y = 45;
            let hp_bar_width = 100;
            self.draw_hp_bar(
                display,
                (right_x, hp_bar_y),
                self.game_hero.current_hp,
                self.game_hero.max_hp,
                hp_bar_width,
            )?;

            // HP text only (SP removed)
            let mut hp_str = heapless::String::<32>::new();
            write!(
                hp_str,
                "HP:{}/{}",
                self.game_hero.current_hp, self.game_hero.max_hp
            )
            .ok();
            Text::new(&hp_str, Point::new(right_x, hp_bar_y + 20), text_style_info).draw(display)?;
        }

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

    /// Draw fragment drop notification (centered at bottom)
    fn draw_fragment_notification(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        if let Some((message, timestamp)) = &self.fragment_notification {
            // Show notification for 2 seconds
            let elapsed = timestamp.elapsed().as_secs_f32();
            if elapsed < 2.0 {
                use core::fmt::Write;

                // Calculate opacity (fade out in last 0.5 seconds)
                let opacity = if elapsed > 1.5 {
                    ((2.0 - elapsed) / 0.5) * 255.0
                } else {
                    255.0
                } as u8;

                // Draw background panel at bottom center
                let panel_width = 300;
                let panel_height = 40;
                let panel_x = (368 - panel_width) / 2;
                let panel_y = 450 - panel_height - 20; // 20px from bottom

                // Semi-transparent dark background
                Rectangle::new(
                    Point::new(panel_x as i32, panel_y as i32),
                    Size::new(panel_width, panel_height),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 20, 60)))
                .draw(display)?;

                // Draw border
                Rectangle::new(
                    Point::new(panel_x as i32, panel_y as i32),
                    Size::new(panel_width, panel_height),
                )
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(150, 100, 255), 2))
                .draw(display)?;

                // Draw text (centered)
                let text_color = Rgb888::new(255, 255, 100); // Yellow
                let text_style = MonoTextStyle::new(&FONT_10X20, text_color);

                // Center the text
                let text_x = panel_x as i32 + 10;
                let text_y = panel_y as i32 + 25;

                let mut msg_str = heapless::String::<64>::new();
                write!(msg_str, "✨ {}", message).ok();
                Text::new(&msg_str, Point::new(text_x, text_y), text_style).draw(display)?;
            } else {
                // Clear notification after 2 seconds
                self.fragment_notification = None;
            }
        }

        Ok(())
    }

    /// Draw team Rustymon buttons at bottom of screen
    fn draw_team_buttons(&mut self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        use core::fmt::Write;

        // Clear touch areas before redrawing
        self.touch_areas.clear();

        // Get team Rustymon IDs
        let team_rustymon_ids = self.rustymon_team.get_team_ids();
        let active_id = self.rustymon_team.get_active_rustymon_id();

        if team_rustymon_ids.is_empty() {
            return Ok(());
        }

        // Button dimensions and positions
        let button_width = 55u32;
        let button_height = 50u32;
        let spacing = 5i32;
        let start_x = 10i32;
        let y = 400i32; // Bottom of screen

        // Draw up to 6 team buttons
        for (slot_index, rustymon_id) in team_rustymon_ids.iter().enumerate().take(6) {
            let x = start_x + (slot_index as i32 * (button_width as i32 + spacing));

            // Find the Rustymon in collection
            let rustymon = self.rustymon_collection
                .iter()
                .find(|r| r.id == *rustymon_id);

            if let Some(rustymon) = rustymon {
                let is_active = active_id.map(|id| id == rustymon_id).unwrap_or(false);
                let is_fainted = rustymon.current_hp == 0;

                // Button color based on state
                let (bg_color, border_color) = if is_fainted {
                    (Rgb888::new(60, 30, 30), Rgb888::new(100, 50, 50)) // Red for fainted
                } else if is_active {
                    (Rgb888::new(40, 80, 120), Rgb888::new(80, 160, 255)) // Blue for active
                } else {
                    (Rgb888::new(40, 60, 40), Rgb888::new(80, 120, 80)) // Green for available
                };

                // Draw button background
                Rectangle::new(
                    Point::new(x, y),
                    Size::new(button_width, button_height),
                )
                .into_styled(
                    embedded_graphics::primitives::PrimitiveStyleBuilder::new()
                        .fill_color(bg_color)
                        .stroke_color(border_color)
                        .stroke_width(if is_active { 3 } else { 2 })
                        .build(),
                )
                .draw(display)?;

                // Draw level
                let level_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                let mut level_str = heapless::String::<8>::new();
                write!(level_str, "Lv{}", rustymon.level).ok();
                Text::new(&level_str, Point::new(x + 2, y + 10), level_style).draw(display)?;

                // Draw HP bar
                let hp_bar_width = button_width - 4;
                let hp_bar_height = 4u32;
                let hp_bar_x = x + 2;
                let hp_bar_y = y + 15;

                // HP bar background
                Rectangle::new(
                    Point::new(hp_bar_x, hp_bar_y),
                    Size::new(hp_bar_width, hp_bar_height),
                )
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
                .draw(display)?;

                // HP bar fill
                let hp_percentage = rustymon.current_hp as f32 / rustymon.max_hp as f32;
                let hp_fill_width = (hp_bar_width as f32 * hp_percentage) as u32;
                let hp_color = if hp_percentage > 0.5 {
                    Rgb888::new(100, 255, 100) // Green
                } else if hp_percentage > 0.25 {
                    Rgb888::new(255, 255, 100) // Yellow
                } else {
                    Rgb888::new(255, 100, 100) // Red
                };

                if hp_fill_width > 0 {
                    Rectangle::new(
                        Point::new(hp_bar_x, hp_bar_y),
                        Size::new(hp_fill_width, hp_bar_height),
                    )
                    .into_styled(PrimitiveStyle::with_fill(hp_color))
                    .draw(display)?;
                }

                // Draw name (truncated)
                let name_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                let mut name_str = heapless::String::<12>::new();
                if rustymon.name.len() > 9 {
                    write!(name_str, "{}.", &rustymon.name[..8]).ok();
                } else {
                    write!(name_str, "{}", rustymon.name).ok();
                }
                Text::new(&name_str, Point::new(x + 2, y + 30), name_style).draw(display)?;

                // Draw element indicator (small colored box)
                let element_color = crate::game::element_system::get_element_color(rustymon.element);
                Rectangle::new(
                    Point::new(x + 2, y + 35),
                    Size::new(button_width - 4, 10),
                )
                .into_styled(PrimitiveStyle::with_fill(element_color))
                .draw(display)?;

                // Add touch area (only if not fainted)
                if !is_fainted {
                    self.touch_areas.push(TouchArea {
                        bounds: (x, y, button_width, button_height),
                        action: BattleAction::SwitchRustymon(slot_index),
                    });
                }
            }
        }

        Ok(())
    }
}

impl Page for BattlePage {
    fn update(&mut self) -> bool {
        // Check for attacks and update animations
        // Only allow attacks if target is alive
        let enemy_is_alive = self.enemy.as_ref().map_or(false, |e| !e.is_dead);
        let hero_attacking =
            enemy_is_alive && self.hero.as_ref().map_or(false, |h| h.should_attack());
        let enemy_attacking = self
            .enemy
            .as_ref()
            .map_or(false, |e| e.should_attack() && !e.is_dead);

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
        // Check for hero attack hit point first (before mutably borrowing)
        let hero_at_hit_point = self.hero.as_ref().map_or(false, |h| h.is_attack_hit_point());

        // Check if we have an active Rustymon (before complex borrows)
        let has_active_rustymon = self.rustymon_team.get_active_rustymon_id().is_some();

        if let Some(hero) = &mut self.hero {
            hero.update();
        }

        // Deal damage mid-attack animation (when hit lands)
        if hero_at_hit_point {
            // Get enemy bounds first
            let enemy_bounds = self.enemy.as_ref().map(|e| e.bounds());
            let enemy_is_dead = self.enemy.as_ref().map_or(true, |e| e.is_dead);

            if !enemy_is_dead {
                // Calculate damage using RPG system
                // Prefer Rustymon attack if available, otherwise use hero
                let damage_result = if has_active_rustymon {
                    // Need to borrow rustymon_collection and game_enemy mutably at the same time
                    // Can't use helper method because it borrows through self
                    let active_id = self.rustymon_team.get_active_rustymon_id().map(|id| id.clone());
                    if let Some(id) = active_id {
                        // Find the active rustymon and borrow game_enemy in the same scope
                        let rustymon = self.rustymon_collection.iter_mut().find(|r| r.id == id);
                        match (rustymon, &mut self.game_enemy) {
                            (Some(rustymon), Some(game_enemy)) => {
                                game::rustymon_attack_enemy(rustymon, game_enemy)
                            }
                            (_, Some(game_enemy)) => {
                                // Rustymon not found, fallback to hero
                                game::hero_attack(&self.game_hero, game_enemy)
                            }
                            _ => {
                                // No game_enemy, shouldn't happen but handle gracefully
                                DamageResult { damage: 0, is_critical: false, is_miss: true }
                            }
                        }
                    } else {
                        // No active rustymon, use hero
                        if let Some(game_enemy) = &mut self.game_enemy {
                            game::hero_attack(&self.game_hero, game_enemy)
                        } else {
                            DamageResult { damage: 0, is_critical: false, is_miss: true }
                        }
                    }
                } else {
                    // No active rustymon, use hero
                    if let Some(game_enemy) = &mut self.game_enemy {
                        game::hero_attack(&self.game_hero, game_enemy)
                    } else {
                        DamageResult { damage: 0, is_critical: false, is_miss: true }
                    }
                };

                // Create floating damage number near enemy
                if let Some(bounds) = enemy_bounds {
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
                }

                // Check if enemy died from the attack
                if let Some(game_enemy) = &mut self.game_enemy {
                    if !game_enemy.is_alive() {
                        // Enemy died - show death animation
                        if let Some(enemy) = &mut self.enemy {
                            enemy.start_death();
                        }

                        // Award EXP and record kill
                        let exp_reward = game_enemy.exp_reward;
                        let enemy_id = game_enemy.id;
                        let enemy_name = game_enemy.name.clone();

                        // Award EXP to active Rustymon if present, otherwise to hero
                        if has_active_rustymon {
                            if let Some(rustymon) = self.get_active_rustymon_mut() {
                                let exp_to_give = exp_reward as u32; // Cast u64 to u32
                                let leveled_up = rustymon.gain_exp(exp_to_give);
                                if leveled_up {
                                    log::info!("🎉 {} leveled up to Lv {}!", rustymon.name, rustymon.level);
                                }
                                log::info!(
                                    "{} defeated! {} gained {} EXP (Lv {} - {}/{})",
                                    enemy_name,
                                    rustymon.name,
                                    exp_to_give,
                                    rustymon.level,
                                    rustymon.exp,
                                    rustymon.exp_to_next
                                );
                            }
                        } else {
                            self.game_hero.gain_exp(exp_reward);
                            log::info!(
                                "{} defeated! Gained {} EXP (Hero: Lv {} - {}/{})",
                                enemy_name,
                                exp_reward,
                                self.game_hero.level,
                                self.game_hero.exp,
                                self.game_hero.exp_to_next_level
                            );
                        }

                        self.kill_tracker.record_kill(enemy_id, &enemy_name);

                        // Award gold and process drops
                        if let Some(enemy_data) = self.game_data.get_enemy(enemy_id) {
                            // Award gold (random amount between min and max)
                            let gold_reward = if enemy_data.gold_max > enemy_data.gold_min {
                                use rand::Rng;
                                let mut rng = rand::thread_rng();
                                rng.gen_range(enemy_data.gold_min..=enemy_data.gold_max)
                            } else {
                                enemy_data.gold_min
                            };
                            self.game_hero.gold += gold_reward;
                            log::info!("💰 Gained {} gold (Total: {})", gold_reward, self.game_hero.gold);

                            // Process item drops
                            for drop in &enemy_data.drops {
                                if drop.should_drop() {
                                    let quantity = drop.random_quantity();

                                    // Add to inventory
                                    if let Err(e) = self.game_hero.inventory.add_material(
                                        drop.item_id,
                                        quantity,
                                        self.game_data.get_all_items()
                                    ) {
                                        log::warn!("Failed to add drop {}: {}", drop.name, e);
                                    } else {
                                        log::info!("🎁 Dropped: {} x{}", drop.name, quantity);
                                    }
                                }
                            }

                            // Check for fragment drop
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let roll: f32 = rng.gen();

                            if roll < enemy_data.fragment_drop_rate {
                                // Check if we can still collect fragments (not at cap)
                                let current_count = self.fragment_collection.get_fragment_count(enemy_id);
                                let required_count = enemy_data.fragments_required;

                                if current_count < required_count {
                                    // Fragment dropped!
                                    self.fragment_drops.push((enemy_id, enemy_name.clone()));

                                    // Update local fragment_collection to enforce cap in same battle
                                    self.fragment_collection.add_fragment(enemy_id, 1);

                                    let message = format!("Fragment obtained: {}! ({}/{})",
                                        enemy_name, current_count + 1, required_count);
                                    self.fragment_notification = Some((message.clone(), Instant::now()));
                                    log::info!("✨ {}", message);
                                } else {
                                    log::debug!("Fragment cap reached for {} ({}/{}), no drop",
                                        enemy_name, current_count, required_count);
                                }
                            }
                        }
                    } else {
                        // Enemy survived - show attacked animation
                        if let Some(enemy) = &mut self.enemy {
                            enemy.start_attacked();
                        }
                    }
                }

                // Mark damage as dealt
                if let Some(hero) = &mut self.hero {
                    hero.mark_damage_dealt();
                }
            }
        }

        // Return to idle after attack/attacked animation completes
        if let Some(hero) = &mut self.hero {
            if hero.current_animation == AnimationType::Attack && hero.is_animation_complete() {
                hero.set_animation(AnimationType::Idle);
            }

            if hero.current_animation == AnimationType::Attacked && hero.is_animation_complete() {
                hero.set_animation(AnimationType::Idle);
            }
        }

        // Check for enemy attack hit point first (before mutably borrowing)
        let enemy_at_hit_point = self.enemy.as_ref().map_or(false, |e| e.is_attack_hit_point());

        if let Some(enemy) = &mut self.enemy {
            enemy.update();
        }

        // Deal damage mid-attack animation (when hit lands)
        if enemy_at_hit_point {
            // Get hero bounds first
            let hero_bounds = self.hero.as_ref().map(|h| h.bounds());

            // Calculate damage using RPG system
            // Prefer attacking Rustymon if available, otherwise attack hero
            let damage_result = if has_active_rustymon {
                // Need to borrow rustymon_collection and game_enemy at the same time
                let active_id = self.rustymon_team.get_active_rustymon_id().map(|id| id.clone());
                if let Some(id) = active_id {
                    let rustymon = self.rustymon_collection.iter_mut().find(|r| r.id == id);
                    match (rustymon, &self.game_enemy) {
                        (Some(rustymon), Some(game_enemy)) => {
                            game::enemy_attack_rustymon(game_enemy, rustymon)
                        }
                        (_, Some(game_enemy)) => {
                            // Rustymon not found, fallback to hero
                            game::enemy_attack(game_enemy, &mut self.game_hero)
                        }
                        _ => {
                            DamageResult { damage: 0, is_critical: false, is_miss: true }
                        }
                    }
                } else {
                    // No active rustymon, attack hero
                    if let Some(game_enemy) = &self.game_enemy {
                        game::enemy_attack(game_enemy, &mut self.game_hero)
                    } else {
                        DamageResult { damage: 0, is_critical: false, is_miss: true }
                    }
                }
            } else {
                // No active rustymon, attack hero
                if let Some(game_enemy) = &self.game_enemy {
                    game::enemy_attack(game_enemy, &mut self.game_hero)
                } else {
                    DamageResult { damage: 0, is_critical: false, is_miss: true }
                }
            };

            if self.game_enemy.is_some() {

                // Create floating damage number near hero
                if let Some(bounds) = hero_bounds {
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
                }

                // Show attacked animation on hero
                if let Some(hero) = &mut self.hero {
                    hero.start_attacked();
                }

                // Check if active Rustymon fainted or hero died
                if has_active_rustymon {
                    if let Some(rustymon) = self.get_active_rustymon() {
                        if !rustymon.is_alive() {
                            log::warn!("💀 {} fainted!", rustymon.name);
                            // TODO: Auto-switch to next available Rustymon
                            // For now, treat as hero death
                            self.hero_died = true;
                        }
                    }
                } else {
                    if !self.game_hero.is_alive() {
                        log::warn!("💀 Hero defeated!");
                        self.hero_died = true;
                    }
                }

                // Death will be handled by battle system (switch to death page)
            }

            // Mark damage as dealt
            if let Some(enemy) = &mut self.enemy {
                enemy.mark_damage_dealt();
            }
        }

        // Return to idle after attack/attacked animation completes
        if let Some(enemy) = &mut self.enemy {
            if enemy.current_animation == AnimationType::Attack && enemy.is_animation_complete() {
                enemy.set_animation(AnimationType::Idle);
            }

            if enemy.current_animation == AnimationType::Attacked && enemy.is_animation_complete() {
                if !enemy.is_dead {
                    enemy.set_animation(AnimationType::Idle);
                }
            }
        }

        // Remove completed damage number animations
        self.damage_numbers.retain(|dmg| !dmg.is_complete());

        // HP Regeneration (every 5 seconds)
        if self.last_hp_regen.elapsed() >= Duration::from_secs(5) {
            // Heal active Rustymon if present, otherwise heal hero
            if let Some(rustymon) = self.get_active_rustymon_mut() {
                // Rustymon regenerate 5% of max HP
                let hp_regen = (rustymon.max_hp as f32 * 0.05) as u32;
                let old_hp = rustymon.current_hp;
                rustymon.heal(hp_regen);

                if rustymon.current_hp > old_hp {
                    log::info!("❤️ {} HP Regen: +{} ({}/{})", rustymon.name, hp_regen, rustymon.current_hp, rustymon.max_hp);
                }
            } else {
                let hp_regen = self.game_hero.stats.calculate_hp_regen();
                let old_hp = self.game_hero.current_hp;
                self.game_hero.current_hp = (self.game_hero.current_hp + hp_regen).min(self.game_hero.max_hp);

                if self.game_hero.current_hp > old_hp {
                    log::info!("❤️ HP Regen: +{} ({}/{})", hp_regen, self.game_hero.current_hp, self.game_hero.max_hp);
                }
            }

            self.last_hp_regen = Instant::now();
        }

        // Check if enemy death animation is complete and should respawn
        // This is done at the END of update so the death animation renders properly
        if let Some(enemy) = &self.enemy {
            if enemy.is_dead && enemy.is_death_complete() {
                log::info!("Enemy death animation complete, respawning...");
                if let Err(e) = self.respawn_enemy() {
                    log::error!("Failed to respawn enemy: {:?}", e);
                }
            }
        }

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

        // Draw fragment notification (if any)
        self.draw_fragment_notification(display)?;

        // Draw team Rustymon buttons at bottom
        self.draw_team_buttons(display)?;

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
