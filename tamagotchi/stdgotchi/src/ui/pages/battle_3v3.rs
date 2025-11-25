//! 3v3 Battle Page
//!
//! Displays 3 heroes vs 3 enemies with HP bars and random targeting

use crate::assets::battle::{load_enemy_sprites_embedded};
use crate::display::Sh8601Driver;
use crate::game::{Enemy as GameEnemy, FragmentCollection, GameData, KillTracker, Rustymon, RustymonTeam};
use crate::ui::page::Page;
use crate::ui::sprite::AnimatedSprite;
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::error::Error;
use std::time::{Duration, Instant};
use rand::Rng;

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
/// Memory-optimized: Heap-allocated sprites to avoid stack overflow (12 sprites × 11KB each)
pub struct BattleEntity {
    idle_sprite: Box<AnimatedSprite>,
    attack_sprite: Box<AnimatedSprite>,
    current_animation: AnimationType,
    role: EntityRole,
    last_attack_time: Instant,
    attack_interval: Duration,
    animation_start_time: Instant,
    is_dead: bool,
    death_time: Option<Instant>,
}

impl BattleEntity {
    /// Create a new battle entity with optional horizontal flip
    /// For 3v3: Heap-allocates sprites to avoid stack overflow (Box moves 11KB buffers to heap)
    pub fn new_with_flip(
        role: EntityRole,
        idle_data: &[u8],
        attack_data: &[u8],
        _attacked_data: &[u8],  // Not used
        _death_data: Option<&[u8]>,  // Not loaded to save memory
        position: (i32, i32),
        attack_interval: Duration,
        flip_horizontal: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let frame_delay = Duration::from_millis(100);

        // Load idle sprite (heap-allocated to avoid stack overflow)
        let mut idle_sprite = AnimatedSprite::new(idle_data, position, frame_delay, None)?;
        idle_sprite.set_flip_horizontal(flip_horizontal);
        idle_sprite.set_center_positioned(true);

        // Load attack sprite (heap-allocated to avoid stack overflow)
        let mut attack_sprite = AnimatedSprite::new(attack_data, position, frame_delay, None)?;
        attack_sprite.set_flip_horizontal(flip_horizontal);
        attack_sprite.set_center_positioned(true);

        Ok(Self {
            idle_sprite: Box::new(idle_sprite),
            attack_sprite: Box::new(attack_sprite),
            current_animation: AnimationType::Idle,
            role,
            last_attack_time: Instant::now(),
            attack_interval,
            animation_start_time: Instant::now(),
            is_dead: false,
            death_time: None,
        })
    }

    /// Update entity animation and state
    pub fn update(&mut self, _dt: Duration) {
        // Update current sprite animation
        match self.current_animation {
            AnimationType::Idle => {
                self.idle_sprite.update();
            }
            AnimationType::Attack => {
                self.attack_sprite.update();
                // Return to idle after attack animation completes (500ms)
                if self.animation_start_time.elapsed() >= Duration::from_millis(500) {
                    self.current_animation = AnimationType::Idle;
                }
            }
            AnimationType::Death => {
                // No death animation - just stop updating (stays on last idle frame)
            }
            _ => {}
        }
    }

    /// Draw the entity
    pub fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        // Don't draw if dead
        if self.is_dead {
            return Ok(());
        }

        match self.current_animation {
            AnimationType::Idle | AnimationType::Death => {
                self.idle_sprite.draw(display)?;
            }
            AnimationType::Attack => {
                self.attack_sprite.draw(display)?;
            }
            _ => {
                self.idle_sprite.draw(display)?;
            }
        }
        Ok(())
    }

    /// Trigger attack animation
    pub fn attack(&mut self) {
        if !self.is_dead {
            self.current_animation = AnimationType::Attack;
            self.animation_start_time = Instant::now();
            self.last_attack_time = Instant::now();
            log::debug!("🗡️ Attack animation triggered");
        }
    }

    /// Trigger hit (no visual change for now)
    pub fn take_hit(&mut self) {
        // Could add Attacked animation here if needed
    }

    /// Mark as dead and trigger death animation
    pub fn die(&mut self) {
        if !self.is_dead {
            self.is_dead = true;
            self.death_time = Some(Instant::now());
            self.current_animation = AnimationType::Death;
            self.animation_start_time = Instant::now();
            log::info!("💀 Death animation triggered");
        }
    }

    /// Check if ready to attack
    pub fn can_attack(&self) -> bool {
        !self.is_dead && self.last_attack_time.elapsed() >= self.attack_interval
    }
}

/// Damage number floating animation
struct DamageNumber {
    damage: u32,
    position: (i32, i32),
    spawn_time: Instant,
    is_critical: bool,
}

impl DamageNumber {
    fn new(damage: u32, position: (i32, i32), is_critical: bool) -> Self {
        Self {
            damage,
            position,
            spawn_time: Instant::now(),
            is_critical,
        }
    }

    fn is_expired(&self) -> bool {
        self.spawn_time.elapsed().as_millis() > 800
    }

    fn draw(&self, display: &mut Sh8601Driver) -> Result<(), Box<dyn Error>> {
        let elapsed = self.spawn_time.elapsed().as_millis() as i32;
        let offset_y = -(elapsed / 16); // Float up 50px over 800ms
        let alpha = ((800 - elapsed) as f32 / 800.0 * 255.0) as u8;

        let color = if self.is_critical {
            Rgb888::new(255, alpha, 0) // Orange for crits
        } else {
            Rgb888::new(255, alpha, alpha) // White fading
        };

        let text = format!("{}", self.damage);
        let style = MonoTextStyle::new(&FONT_6X10, color);

        Text::new(
            &text,
            Point::new(self.position.0, self.position.1 + offset_y),
            style,
        )
        .draw(display)?;

        Ok(())
    }
}

/// 3v3 Battle Page
pub struct Battle3v3Page {
    background_color: Rgb888,

    // 3 heroes on the right side
    heroes: [Option<BattleEntity>; 3],
    hero_rustymon: [Option<Rustymon>; 3], // Snapshot of rustymon stats at battle start

    // 3 enemies on the left side
    enemies: [Option<BattleEntity>; 3],
    enemy_data: [Option<GameEnemy>; 3],

    // Game state
    rustymon_collection: Vec<Rustymon>,
    rustymon_team: RustymonTeam,
    game_data: GameData,
    kill_tracker: KillTracker,
    fragment_collection: FragmentCollection,

    // Battle state
    damage_numbers: Vec<DamageNumber>,
    battle_result: BattleResult,
    last_update: Instant,
    fragment_drops: Vec<(u32, String)>, // (enemy_id, enemy_name) - track all fragment drops

    // Wave system
    current_wave: u32,
    total_waves: u32,
    wave_cleared: bool, // Track if current wave is cleared and waiting for next spawn
    wave_clear_time: Option<Instant>, // When wave was cleared (for delay before next wave)
    enemy_ids_pool: Vec<u32>, // Pool of enemy IDs to spawn from

    // Turn management
    turn_timer: Instant,
    turn_delay: Duration, // Delay between attacks
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleResult {
    Ongoing,
    Victory,
    Defeat,
}

impl Battle3v3Page {
    /// Calculate attack interval from AGI stat
    /// Formula: 3000ms / (1 + agi/100)
    /// Example: AGI 50 = 2000ms, AGI 100 = 1500ms, AGI 200 = 1000ms
    fn calculate_attack_interval(agi: u32) -> Duration {
        let base_ms = 3000.0;
        let agi_factor = 1.0 + (agi as f32 / 100.0);
        let interval_ms = (base_ms / agi_factor) as u64;
        let interval_ms = interval_ms.clamp(500, 5000); // Min 0.5s, Max 5s
        Duration::from_millis(interval_ms)
    }

    /// Create a new 3v3 battle
    pub fn new(
        background_color: Rgb888,
        kill_tracker: KillTracker,
        game_data: GameData,
        rustymon_collection: Vec<Rustymon>,
        rustymon_team: RustymonTeam,
        fragment_collection: FragmentCollection,
    ) -> Self {
        Self {
            background_color,
            heroes: [None, None, None],
            hero_rustymon: [None, None, None],
            enemies: [None, None, None],
            enemy_data: [None, None, None],
            rustymon_collection,
            rustymon_team,
            game_data,
            kill_tracker,
            fragment_collection,
            damage_numbers: Vec::new(),
            battle_result: BattleResult::Ongoing,
            last_update: Instant::now(),
            fragment_drops: Vec::new(),
            current_wave: 1,
            total_waves: 5,
            wave_cleared: false,
            wave_clear_time: None,
            enemy_ids_pool: Vec::new(),
            turn_timer: Instant::now(),
            turn_delay: Duration::from_millis(1500), // No longer used, kept for compatibility
        }
    }

    /// Get hero positions (right side, facing left)
    fn get_hero_positions() -> [(i32, i32); 3] {
        [
            (200, 80),   // Top
            (200, 170),  // Middle
            (200, 260),  // Bottom
        ]
    }

    /// Get enemy positions (left side, facing right)
    fn get_enemy_positions() -> [(i32, i32); 3] {
        [
            (60, 80),    // Top
            (60, 170),   // Middle
            (60, 260),   // Bottom
        ]
    }

    /// Initialize battle with first 3 rustymon from team
    /// Returns the number of heroes successfully loaded
    pub fn setup_heroes(&mut self) -> Result<usize, Box<dyn Error>> {
        let positions = Self::get_hero_positions();
        let mut loaded = 0;

        for (i, slot) in self.rustymon_team.active_slots.iter().enumerate() {
            if loaded >= 3 {
                break;
            }

            if let Some(rustymon_id) = slot {
                if let Some(rustymon) = self.rustymon_collection.iter().find(|r| &r.id == rustymon_id) {
                    if rustymon.current_hp > 0 {
                        // Load sprites using species_id (which maps to enemy sprites)
                        let (idle, attack, attacked, death) = load_enemy_sprites_embedded(rustymon.species_id)
                            .ok_or("Failed to load rustymon sprites")?;

                        // Calculate attack interval based on flee (represents agility)
                        let attack_interval = Self::calculate_attack_interval(rustymon.flee);

                        let entity = BattleEntity::new_with_flip(
                            EntityRole::Hero,
                            &idle,
                            &attack,
                            &attacked,
                            death.as_deref(),
                            positions[loaded],
                            attack_interval,
                            true, // Flip to face left
                        )?;

                        self.heroes[loaded] = Some(entity);
                        self.hero_rustymon[loaded] = Some(rustymon.clone());

                        log::info!("Loaded hero {} at team slot {} → array index {}: {} [AGI/FLEE={}] (HP: {}/{}) attack_interval={:?}",
                            loaded, i, loaded, rustymon.name, rustymon.flee,
                            rustymon.current_hp, rustymon.max_hp, attack_interval);
                        loaded += 1;
                    }
                }
            }
        }

        if loaded == 0 {
            return Err("No alive rustymon in team!".into());
        }

        log::info!("Setup {} heroes for 3v3 battle", loaded);
        Ok(loaded)
    }

    /// Add enemies to battle
    pub fn add_enemies(&mut self, enemy_ids: &[u32]) -> Result<(), Box<dyn Error>> {
        // Store enemy pool for wave spawning
        if self.enemy_ids_pool.is_empty() {
            self.enemy_ids_pool = enemy_ids.to_vec();
        }

        let positions = Self::get_enemy_positions();
        let count = enemy_ids.len().min(3);

        for i in 0..count {
            let enemy_id = enemy_ids[i];

            // Get enemy data
            let enemy_data = self.game_data.get_enemy(enemy_id)
                .ok_or(format!("Enemy {} not found", enemy_id))?;

            // Load sprites
            let (idle, attack, attacked, death) = load_enemy_sprites_embedded(enemy_id)
                .ok_or(format!("No sprites for enemy {}", enemy_id))?;

            // Calculate attack interval based on enemy AGI
            let attack_interval = Self::calculate_attack_interval(enemy_data.agi);

            let entity = BattleEntity::new_with_flip(
                EntityRole::Enemy,
                &idle,
                &attack,
                &attacked,
                death.as_deref(),
                positions[i],
                attack_interval,
                false, // Don't flip - face right
            )?;

            let game_enemy = GameEnemy::from_data(
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

            self.enemies[i] = Some(entity);
            self.enemy_data[i] = Some(game_enemy);

            log::info!("Loaded enemy {} at position {}: {} [AGI={}] (HP: {}) attack_interval={:?}",
                i, i, enemy_data.name, enemy_data.agi, enemy_data.hp, attack_interval);
        }

        log::info!("Setup {} enemies for 3v3 battle (Wave {}/{})", count, self.current_wave, self.total_waves);
        Ok(())
    }

    /// Check if all enemies in current wave are dead (and death animations complete)
    fn is_wave_cleared(&self) -> bool {
        // All enemies must be dead
        let all_dead = (0..3).all(|i| {
            self.enemy_data[i].as_ref()
                .map(|e| !e.is_alive())
                .unwrap_or(true) // No enemy data = cleared
        });

        if !all_dead {
            return false;
        }

        // Wait for death animations to complete (1.5 seconds after last death)
        const DEATH_ANIMATION_DURATION: Duration = Duration::from_millis(1500);

        if let Some(wave_clear_time) = self.wave_clear_time {
            wave_clear_time.elapsed() >= DEATH_ANIMATION_DURATION
        } else {
            false
        }
    }

    /// Spawn next wave of enemies
    fn spawn_next_wave(&mut self) -> Result<(), Box<dyn Error>> {
        if self.enemy_ids_pool.is_empty() {
            return Err("No enemy pool available for next wave".into());
        }

        self.current_wave += 1;
        self.wave_cleared = false;
        self.wave_clear_time = None;

        log::info!("🌊 Spawning wave {}/{}", self.current_wave, self.total_waves);

        // Clear old enemies
        for i in 0..3 {
            self.enemies[i] = None;
            self.enemy_data[i] = None;
        }

        // Spawn new enemies from pool (randomly pick)
        let mut enemy_ids = Vec::new();
        let count = self.enemy_ids_pool.len().min(3);

        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let enemy_id = self.enemy_ids_pool[rng.gen_range(0..self.enemy_ids_pool.len())];
            enemy_ids.push(enemy_id);
        }

        self.add_enemies(&enemy_ids)?;

        Ok(())
    }

    /// Get a random alive enemy index
    fn get_random_alive_enemy(&self) -> Option<usize> {
        let alive_enemies: Vec<usize> = (0..3)
            .filter(|&i| {
                self.enemy_data[i].as_ref()
                    .map(|e| e.is_alive())
                    .unwrap_or(false)
            })
            .collect();

        if alive_enemies.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            Some(alive_enemies[rng.gen_range(0..alive_enemies.len())])
        }
    }

    /// Get a random alive hero index
    fn get_random_alive_hero(&self) -> Option<usize> {
        let alive_heroes: Vec<usize> = (0..3)
            .filter(|&i| {
                self.hero_rustymon[i].as_ref()
                    .map(|h| h.current_hp > 0)
                    .unwrap_or(false)
            })
            .collect();

        if alive_heroes.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            Some(alive_heroes[rng.gen_range(0..alive_heroes.len())])
        }
    }

    /// Check battle result and handle waves
    fn check_battle_result(&mut self) {
        // Check if all heroes dead
        let all_heroes_dead = self.hero_rustymon.iter()
            .all(|h| h.as_ref().map(|r| r.current_hp == 0).unwrap_or(true));

        if all_heroes_dead {
            self.battle_result = BattleResult::Defeat;
            log::info!("Battle lost - all heroes defeated!");
            return;
        }

        // Check if all enemies in current wave are dead
        let all_enemies_dead = self.enemy_data.iter()
            .all(|e| e.as_ref().map(|e| !e.is_alive()).unwrap_or(true));

        if all_enemies_dead && !self.wave_cleared {
            // Mark wave as cleared and start timer for death animation
            self.wave_cleared = true;
            self.wave_clear_time = Some(Instant::now());
            log::info!("🌊 Wave {}/{} cleared! Waiting for death animations...", self.current_wave, self.total_waves);
        }

        // Check if we should spawn next wave
        if self.wave_cleared && self.is_wave_cleared() {
            if self.current_wave < self.total_waves {
                // Spawn next wave
                if let Err(e) = self.spawn_next_wave() {
                    log::error!("Failed to spawn next wave: {:?}", e);
                    self.battle_result = BattleResult::Victory; // End battle if spawn fails
                }
            } else {
                // All waves completed - Victory!
                self.battle_result = BattleResult::Victory;
                log::info!("🎉 Battle won - all {} waves defeated!", self.total_waves);

                // Award experience
                for enemy in &self.enemy_data {
                    if let Some(e) = enemy {
                        log::info!("Defeated {} - {} exp", e.name, e.exp_reward);
                    }
                }
            }
        }
    }

    /// Process one turn of combat with AGI-based attack speed
    fn process_turn(&mut self) {
        if self.battle_result != BattleResult::Ongoing {
            return;
        }

        // Process heroes - each attacks independently based on their AGI/attack interval
        for hero_idx in 0..3 {
            // Check if hero entity is ready to attack
            let can_attack = if let Some(hero_entity) = &self.heroes[hero_idx] {
                hero_entity.can_attack()
            } else {
                false
            };

            if can_attack {
                if let Some(hero_rustymon) = &self.hero_rustymon[hero_idx] {
                    if hero_rustymon.current_hp > 0 {
                        // Find random enemy target
                        if let Some(target_idx) = self.get_random_alive_enemy() {
                            // Trigger attack animation
                            if let Some(hero_entity) = &mut self.heroes[hero_idx] {
                                hero_entity.attack();
                            }

                        // Calculate damage
                        let attacker_atk = hero_rustymon.atk;
                        if let Some(enemy) = &mut self.enemy_data[target_idx] {
                            let mut damage = if attacker_atk > enemy.def {
                                attacker_atk - enemy.def
                            } else {
                                1
                            };

                            // Random variance 80-120%
                            let mut rng = rand::thread_rng();
                            let variance = rng.gen_range(80..=120) as f32 / 100.0;
                            damage = (damage as f32 * variance) as u32;
                            damage = damage.max(1);

                            // Apply damage
                            enemy.take_damage(damage);

                            // Check if enemy died from this attack
                            let enemy_died = !enemy.is_alive();
                            let enemy_id = enemy.id;
                            let enemy_name = enemy.name.clone();

                            // Trigger hit animation
                            if let Some(enemy_entity) = &mut self.enemies[target_idx] {
                                enemy_entity.take_hit();
                                if enemy_died {
                                    enemy_entity.die();

                                    // Roll for fragment drop
                                    if let Some(enemy_data) = self.game_data.get_enemy(enemy_id) {
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

                                                log::info!("✨ Fragment obtained: {}! ({}/{})",
                                                    enemy_name, current_count + 1, required_count);
                                            } else {
                                                log::debug!("Fragment cap reached for {} ({}/{}), no drop",
                                                    enemy_name, current_count, required_count);
                                            }
                                        }
                                    }
                                }
                            }

                            // Show damage number
                            if let Some(enemy_entity) = &self.enemies[target_idx] {
                                let pos = Self::get_enemy_positions()[target_idx];
                                self.damage_numbers.push(DamageNumber::new(damage, pos, false));
                            }

                            log::info!("{} attacks {} for {} damage! (HP: {})",
                                hero_rustymon.name, enemy.name, damage, enemy.current_hp);
                        }
                    }
                }
            }
        }

        // Process enemies - each attacks independently based on their AGI/attack interval
        for enemy_idx in 0..3 {
            // Check if enemy entity is ready to attack
            let can_attack = if let Some(enemy_entity) = &self.enemies[enemy_idx] {
                enemy_entity.can_attack()
            } else {
                false
            };

            if can_attack {
                if let Some(enemy) = &self.enemy_data[enemy_idx] {
                    if enemy.is_alive() {
                        // Find random hero target
                        if let Some(target_idx) = self.get_random_alive_hero() {
                            // Trigger attack animation
                            if let Some(enemy_entity) = &mut self.enemies[enemy_idx] {
                                enemy_entity.attack();
                            }

                        // Calculate damage
                        let attacker_atk = enemy.atk;
                        if let Some(hero_rustymon) = &mut self.hero_rustymon[target_idx] {
                            let mut damage = if attacker_atk > hero_rustymon.def {
                                attacker_atk - hero_rustymon.def
                            } else {
                                1
                            };

                            // Random variance
                            let mut rng = rand::thread_rng();
                            let variance = rng.gen_range(80..=120) as f32 / 100.0;
                            damage = (damage as f32 * variance) as u32;
                            damage = damage.max(1);

                            // Apply damage
                            if damage >= hero_rustymon.current_hp {
                                hero_rustymon.current_hp = 0;
                            } else {
                                hero_rustymon.current_hp -= damage;
                            }

                            // Trigger hit animation
                            if let Some(hero_entity) = &mut self.heroes[target_idx] {
                                hero_entity.take_hit();
                                if hero_rustymon.current_hp == 0 {
                                    hero_entity.die();
                                }
                            }

                            // Show damage number
                            let pos = Self::get_hero_positions()[target_idx];
                            self.damage_numbers.push(DamageNumber::new(damage, pos, false));

                            log::info!("{} attacks {} for {} damage! (HP: {})",
                                enemy.name, hero_rustymon.name, damage, hero_rustymon.current_hp);
                        }
                    }
                }
                    }
                }
            }
        }

        // Check for battle end after any attacks
        self.check_battle_result();
    }

    /// Draw HP bar above a character
    fn draw_hp_bar(
        display: &mut Sh8601Driver,
        name: &str,
        current_hp: u32,
        max_hp: u32,
        position: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        let bar_width = 50;
        let bar_height = 4;
        let bar_x = position.0 - bar_width / 2;
        let bar_y = position.1 - 40; // Above the character

        // Background bar (dark)
        Rectangle::new(
            Point::new(bar_x, bar_y),
            Size::new(bar_width as u32, bar_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 50, 50)))
        .draw(display)?;

        // HP bar (colored based on percentage)
        let hp_percentage = (current_hp as f32 / max_hp as f32) * 100.0;
        let hp_color = if hp_percentage > 60.0 {
            Rgb888::new(0, 255, 0) // Green
        } else if hp_percentage > 30.0 {
            Rgb888::new(255, 255, 0) // Yellow
        } else {
            Rgb888::new(255, 0, 0) // Red
        };

        let filled_width = ((current_hp as f32 / max_hp as f32) * bar_width as f32) as u32;
        if filled_width > 0 {
            Rectangle::new(
                Point::new(bar_x, bar_y),
                Size::new(filled_width, bar_height as u32),
            )
            .into_styled(PrimitiveStyle::with_fill(hp_color))
            .draw(display)?;
        }

        // Name text above HP bar
        let name_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let name_x = position.0 - (name.len() as i32 * 3); // Center the name
        Text::new(name, Point::new(name_x, bar_y - 3), name_style).draw(display)?;

        // HP text
        let hp_text = format!("{}/{}", current_hp, max_hp);
        let hp_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let hp_x = position.0 - (hp_text.len() as i32 * 3);
        Text::new(&hp_text, Point::new(hp_x, bar_y + 15), hp_style).draw(display)?;

        Ok(())
    }

    /// Get battle result
    pub fn get_result(&self) -> BattleResult {
        self.battle_result
    }

    /// Get updated rustymon collection (with HP changes)
    pub fn get_rustymon_collection(&self) -> Vec<Rustymon> {
        let mut collection = self.rustymon_collection.clone();

        // Update HP for heroes that were in battle
        // Iterate over hero_rustymon array (not team slots) to ensure all battle participants get synced
        for hero_rustymon in &self.hero_rustymon {
            if let Some(hero) = hero_rustymon {
                // Find the rustymon in collection by ID and update HP
                if let Some(rustymon) = collection.iter_mut().find(|r| r.id == hero.id) {
                    let old_hp = rustymon.current_hp;
                    rustymon.current_hp = hero.current_hp;
                    log::debug!("Synced {} HP: {} -> {}", rustymon.name, old_hp, hero.current_hp);
                }
            }
        }

        collection
    }

    /// Get fragment drops from this battle
    pub fn get_fragment_drops(&self) -> Vec<(u32, String)> {
        self.fragment_drops.clone()
    }
}

impl Page for Battle3v3Page {
    fn update(&mut self) -> bool {
        let dt = self.last_update.elapsed();
        self.last_update = Instant::now();

        // Debug: Log once at start
        static mut FIRST_UPDATE: bool = true;
        unsafe {
            if FIRST_UPDATE {
                log::info!("🎮 Battle3v3Page::update() - First call");
                FIRST_UPDATE = false;
            }
        }

        // Update all entities
        for hero in &mut self.heroes {
            if let Some(entity) = hero {
                entity.update(dt);
            }
        }

        for enemy in &mut self.enemies {
            if let Some(entity) = enemy {
                entity.update(dt);
            }
        }

        // Remove expired damage numbers
        self.damage_numbers.retain(|d| !d.is_expired());

        // Process combat
        self.process_turn();

        // Continue battle unless result is set
        true
    }

    fn draw(&mut self, display: &mut Sh8601Driver, _full_redraw: bool) -> Result<(), Box<dyn Error>> {
        // Debug: Log first draw with hero info
        static mut FIRST_DRAW: bool = true;
        unsafe {
            if FIRST_DRAW {
                log::info!("🎨 Battle3v3Page::draw() - First call");
                // Log all heroes being drawn
                for i in 0..3 {
                    if let Some(ref hero_rustymon) = self.hero_rustymon[i] {
                        let pos = Self::get_hero_positions()[i];
                        log::info!("  Hero {}: {} at position {:?} (HP: {}/{})",
                            i, hero_rustymon.name, pos, hero_rustymon.current_hp, hero_rustymon.max_hp);
                    } else {
                        log::info!("  Hero {}: None", i);
                    }
                }
                FIRST_DRAW = false;
            }
        }

        // Clear background
        display.clear(self.background_color)?;

        // Draw wave counter at top center
        let wave_style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
        let mut wave_text = heapless::String::<32>::new();
        use core::fmt::Write;
        let _ = write!(wave_text, "Wave {}/{}", self.current_wave, self.total_waves);
        Text::new(&wave_text, Point::new(105, 10), wave_style).draw(display)?;

        // Draw all enemies
        for i in 0..3 {
            if let Some(enemy_entity) = &self.enemies[i] {
                enemy_entity.draw(display)?;

                // Draw HP bar
                if let Some(enemy) = &self.enemy_data[i] {
                    let pos = Self::get_enemy_positions()[i];
                    Self::draw_hp_bar(
                        display,
                        &enemy.name,
                        enemy.current_hp,
                        enemy.max_hp,
                        pos,
                    )?;
                }
            }
        }

        // Draw all heroes
        for i in 0..3 {
            if let Some(hero_entity) = &self.heroes[i] {
                hero_entity.draw(display)?;

                // Draw HP bar
                if let Some(hero) = &self.hero_rustymon[i] {
                    let pos = Self::get_hero_positions()[i];
                    Self::draw_hp_bar(
                        display,
                        &hero.name,
                        hero.current_hp,
                        hero.max_hp,
                        pos,
                    )?;
                }
            }
        }

        // Draw damage numbers
        for damage in &self.damage_numbers {
            damage.draw(display)?;
        }

        // Draw battle result if ended
        if self.battle_result != BattleResult::Ongoing {
            let result_text = match self.battle_result {
                BattleResult::Victory => "VICTORY!",
                BattleResult::Defeat => "DEFEAT!",
                BattleResult::Ongoing => "",
            };

            if !result_text.is_empty() {
                let style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
                Text::new(result_text, Point::new(100, 20), style).draw(display)?;
            }
        }

        // Flush to display
        display.flush()?;

        Ok(())
    }

    fn mark_dirty(&mut self) {
        // No-op for now
    }

    fn needs_full_redraw(&self) -> bool {
        true // Always redraw for animations
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
