use bevy_ecs::prelude::*;
use core::fmt::Write;
use heapless::String;

/// Game pages/screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePage {
    Overview,
    Farm,
    Rest,
    Battle,  // Whac-A-Mole mini-game
    Menu,
}

/// Hero character data
#[derive(Debug, Clone)]
pub struct Hero {
    pub name: &'static str,
    pub level: u16,
    pub exp: u32,
    pub exp_to_next_level: u32,
    pub job: &'static str,
    pub hp: u16,
    pub max_hp: u16,
    pub sp: u16,
    pub max_sp: u16,
    pub zeny: u32, // Currency
}

impl Hero {
    pub fn new() -> Self {
        Self {
            name: "Novice",
            level: 1,
            exp: 0,
            exp_to_next_level: 100,
            job: "Novice",
            hp: 100,
            max_hp: 100,
            sp: 50,
            max_sp: 50,
            zeny: 0,
        }
    }

    /// Add experience and handle level up
    pub fn add_exp(&mut self, amount: u32) {
        self.exp += amount;

        // Check for level up
        while self.exp >= self.exp_to_next_level {
            self.level_up();
        }
    }

    /// Level up the hero
    fn level_up(&mut self) {
        self.level += 1;
        self.exp -= self.exp_to_next_level;

        // Increase stats
        self.max_hp += 10;
        self.max_sp += 5;
        self.hp = self.max_hp;
        self.sp = self.max_sp;

        // Increase exp requirement
        self.exp_to_next_level = (self.exp_to_next_level as f32 * 1.2) as u32;

        // Job progression
        if self.level == 10 && self.job == "Novice" {
            self.job = "Swordsman";
            self.name = "Swordsman";
        }
    }

    /// Add zeny (currency)
    pub fn add_zeny(&mut self, amount: u32) {
        self.zeny += amount;
    }

    /// Use SP for activities
    pub fn use_sp(&mut self, amount: u16) -> bool {
        if self.sp >= amount {
            self.sp -= amount;
            true
        } else {
            false
        }
    }

    /// Regenerate SP while resting
    pub fn regenerate_sp(&mut self, amount: u16) {
        self.sp = (self.sp + amount).min(self.max_sp);
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: u16) {
        self.hp = self.hp.saturating_sub(damage);
    }

    /// Heal HP
    pub fn heal(&mut self, amount: u16) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Check if hero is alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Get HP percentage
    pub fn hp_percent(&self) -> u8 {
        ((self.hp as u32 * 100) / self.max_hp as u32) as u8
    }

    /// Get SP percentage
    pub fn sp_percent(&self) -> u8 {
        ((self.sp as u32 * 100) / self.max_sp as u32) as u8
    }

    /// Get EXP percentage
    pub fn exp_percent(&self) -> u8 {
        ((self.exp as u64 * 100) / self.exp_to_next_level as u64) as u8
    }

    /// Serialize hero data to a CSV-like string format
    /// Format: level,exp,exp_to_next,job,hp,max_hp,sp,max_sp,zeny
    pub fn to_save_string(&self) -> String<128> {
        let mut save_str = String::<128>::new();
        write!(
            save_str,
            "{},{},{},{},{},{},{},{},{}",
            self.level,
            self.exp,
            self.exp_to_next_level,
            self.job,
            self.hp,
            self.max_hp,
            self.sp,
            self.max_sp,
            self.zeny
        ).ok();
        save_str
    }

    /// Deserialize hero data from a CSV-like string
    pub fn from_save_string(data: &str) -> Option<Self> {
        // Trim whitespace and newlines first
        let data = data.trim();

        // Use splitn to limit splits and avoid overflow
        let mut parts = data.split(',');

        // Parse each field manually to avoid Vec overflow
        let level: u16 = parts.next()?.parse().ok()?;
        let exp: u32 = parts.next()?.parse().ok()?;
        let exp_to_next_level: u32 = parts.next()?.parse().ok()?;
        let job_str = parts.next()?;
        let hp: u16 = parts.next()?.parse().ok()?;
        let max_hp: u16 = parts.next()?.parse().ok()?;
        let sp: u16 = parts.next()?.parse().ok()?;
        let max_sp: u16 = parts.next()?.parse().ok()?;
        let zeny: u32 = parts.next()?.parse().ok()?;

        // Parse job to a static string
        let job: &'static str = if job_str == "Novice" { "Novice" } else { "Swordsman" };
        let name: &'static str = if job_str == "Novice" { "Novice" } else { "Swordsman" };

        Some(Hero {
            name,
            level,
            exp,
            exp_to_next_level,
            job,
            hp,
            max_hp,
            sp,
            max_sp,
            zeny,
        })
    }
}

/// Enemy data
#[derive(Debug, Clone)]
pub struct Enemy {
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub exp_reward: u32,
    pub zeny_reward: u32,
}

impl Enemy {
    pub fn poring(level: u16) -> Self {
        Self {
            name: "Poring",
            level,
            hp: 50 + level * 10,
            max_hp: 50 + level * 10,
            exp_reward: 10 + level as u32 * 5,
            zeny_reward: 5 + level as u32 * 2,
        }
    }

    pub fn lunatic(level: u16) -> Self {
        Self {
            name: "Lunatic",
            level,
            hp: 70 + level * 12,
            max_hp: 70 + level * 12,
            exp_reward: 15 + level as u32 * 7,
            zeny_reward: 8 + level as u32 * 3,
        }
    }

    pub fn spore(level: u16) -> Self {
        Self {
            name: "Spore",
            level,
            hp: 60 + level * 11,
            max_hp: 60 + level * 11,
            exp_reward: 12 + level as u32 * 6,
            zeny_reward: 6 + level as u32 * 2,
        }
    }

    /// Get a random enemy based on hero level
    pub fn random_for_level(hero_level: u16, rng_value: u8) -> Self {
        let enemy_level = hero_level.max(1);
        match rng_value % 3 {
            0 => Self::poring(enemy_level),
            1 => Self::lunatic(enemy_level),
            _ => Self::spore(enemy_level),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn hp_percent(&self) -> u8 {
        ((self.hp as u32 * 100) / self.max_hp as u32) as u8
    }
}

/// Farming state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarmState {
    Idle,
    Fighting,
    Victory,
    Defeat,
}

/// Rest state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestState {
    Resting,
    FullSP,
}

/// Battle state for Whac-A-Mole mini-game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleState {
    Idle,      // Waiting to start
    Playing,   // Active gameplay
    Victory,   // Won the game
    Defeat,    // Lost the game
}

/// Circle type for Whac-A-Mole game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleType {
    GoodTarget,   // Click to hit enemy (green) - gain score
    BadTarget,    // Enemy attack (red) - must click to block, else take damage
}

/// Active circle in the Whac-A-Mole game
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub x: i32,
    pub y: i32,
    pub radius: u32,
    pub circle_type: CircleType,
    pub spawn_time: u32,     // When it spawned
    pub lifetime_ms: u32,    // How long it lasts (1500ms)
}

impl Circle {
    pub fn new(x: i32, y: i32, circle_type: CircleType, spawn_time: u32) -> Self {
        Self {
            x,
            y,
            radius: 25,  // Fixed radius
            circle_type,
            spawn_time,
            lifetime_ms: 1500,  // 1.5 seconds to click
        }
    }

    pub fn is_expired(&self, current_time: u32) -> bool {
        current_time >= self.spawn_time + self.lifetime_ms
    }

    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        let dx = self.x - px;
        let dy = self.y - py;
        (dx * dx + dy * dy) <= (self.radius as i32 * self.radius as i32)
    }
}

/// Main game state resource
#[derive(Resource)]
pub struct GameState {
    pub current_page: GamePage,
    pub hero: Hero,
    pub current_enemy: Option<Enemy>,
    pub farm_state: FarmState,
    pub farm_progress: u32,      // 0-60000 (60 seconds in milliseconds)
    pub farm_duration_ms: u32,   // 60000 ms = 1 minute
    pub farm_touch_cooldown: u32, // Cooldown in ms to prevent immediate re-touch
    pub rest_state: RestState,
    pub rest_progress: u32,      // Progress in milliseconds
    pub sp_regen_rate: u16,      // SP per second while resting
    pub menu_selection: u8,      // 0 = Overview, 1 = Farm, 2 = Rest, 3 = Battle, 4 = Save
    pub battle_state: BattleState, // Current battle state
    pub battle_enemy: Option<Enemy>, // Enemy being fought
    pub battle_circles: [Option<Circle>; 4], // Up to 4 active circles
    pub battle_score: u16,       // Hits made in current battle
    pub battle_missed: u16,      // Circles missed or bad targets hit
    pub battle_next_spawn: u32,  // When next circle spawns
    pub battle_spawn_interval: u32, // Time between spawns (800ms)
    pub battle_duration: u32,    // Total battle time (30 seconds)
    pub battle_elapsed: u32,     // Time elapsed in battle
    pub battle_last_touch_x: i32, // Last touch X position for debug display
    pub battle_last_touch_y: i32, // Last touch Y position for debug display
    pub battle_last_touch_time: u32, // When last touch occurred (for fade out)
    pub last_update_ms: u32,     // Last update time for progress tracking
    pub save_requested: bool,    // Flag to trigger save
    pub save_status_msg: Option<&'static str>, // Status message after save
    pub save_status_timeout: u32, // Time when save message should clear (0 = no message)
    pub fps: u32,                // Current FPS
    pub frame_count: u32,        // Total frames rendered
    pub last_fps_update_ms: u32, // Last time FPS was calculated
    pub needs_redraw: bool,      // Flag to indicate screen needs redrawing
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_page: GamePage::Overview,
            hero: Hero::new(),
            current_enemy: None,
            farm_state: FarmState::Idle,
            farm_progress: 0,
            farm_duration_ms: 60000, // 1 minute
            farm_touch_cooldown: 0,
            rest_state: RestState::Resting,
            rest_progress: 0,
            sp_regen_rate: 5, // 5 SP per second
            menu_selection: 0,
            battle_state: BattleState::Idle,
            battle_enemy: None,
            battle_circles: [None, None, None, None],
            battle_score: 0,
            battle_missed: 0,
            battle_next_spawn: 0,
            battle_spawn_interval: 800,  // 800ms between spawns
            battle_duration: 30000,      // 30 seconds
            battle_elapsed: 0,
            battle_last_touch_x: 0,
            battle_last_touch_y: 0,
            battle_last_touch_time: 0,
            last_update_ms: 0,
            save_requested: false,
            save_status_msg: None,
            save_status_timeout: 0,
            fps: 0,
            frame_count: 0,
            last_fps_update_ms: 0,
            needs_redraw: true, // Start with needing a redraw
        }
    }
}

impl GameState {
    /// Start farming with a new enemy
    pub fn start_farming(&mut self, enemy: Enemy) {
        if self.hero.use_sp(20) {
            self.current_enemy = Some(enemy);
            self.farm_state = FarmState::Fighting;
            self.farm_progress = 0;
            self.current_page = GamePage::Farm;
        }
    }

    /// Update farming progress
    pub fn update_farm_progress(&mut self, delta_ms: u32) {
        if self.farm_state == FarmState::Fighting {
            self.farm_progress += delta_ms;

            if self.farm_progress >= self.farm_duration_ms {
                self.complete_farming();
            }
        }
    }

    /// Complete farming and award rewards
    fn complete_farming(&mut self) {
        if let Some(enemy) = &self.current_enemy {
            self.hero.add_exp(enemy.exp_reward);
            self.hero.add_zeny(enemy.zeny_reward);
            self.farm_state = FarmState::Victory;
        }
    }

    /// Reset farming state
    pub fn reset_farming(&mut self) {
        self.current_enemy = None;
        self.farm_state = FarmState::Idle;
        self.farm_progress = 0;
        // Set cooldown to prevent immediate re-touch (300ms)
        self.farm_touch_cooldown = 300;
    }

    /// Update rest progress
    pub fn update_rest_progress(&mut self, delta_ms: u32) {
        if self.rest_state == RestState::Resting {
            self.rest_progress += delta_ms;

            // Regenerate SP and HP every second
            if self.rest_progress >= 1000 {
                let seconds = self.rest_progress / 1000;

                // Regenerate SP (5 SP per second by default)
                self.hero.regenerate_sp((seconds as u16) * self.sp_regen_rate);

                // Regenerate HP (10 HP per second)
                let hp_regen_rate = 10u16;
                self.hero.heal((seconds as u16) * hp_regen_rate);

                self.rest_progress %= 1000;

                // Check if both SP and HP are full
                if self.hero.sp >= self.hero.max_sp && self.hero.hp >= self.hero.max_hp {
                    self.rest_state = RestState::FullSP;
                }
            }
        }
    }

    /// Get farm progress percentage
    pub fn farm_progress_percent(&self) -> u8 {
        ((self.farm_progress as u64 * 100) / self.farm_duration_ms as u64) as u8
    }

    /// Start Whac-A-Mole battle
    pub fn start_battle(&mut self, enemy: Enemy) {
        self.battle_enemy = Some(enemy);
        self.battle_state = BattleState::Playing;
        self.battle_circles = [None, None, None, None];
        self.battle_score = 0;
        self.battle_missed = 0;
        self.battle_elapsed = 0;
        self.battle_next_spawn = self.last_update_ms + 500; // First spawn in 500ms
    }

    /// Spawn a new circle in the battle
    pub fn spawn_battle_circle(&mut self, rng_value: u8) {
        // Find empty slot
        for slot in &mut self.battle_circles {
            if slot.is_none() {
                // Random position in play area (avoid edges)
                let x = 40 + ((rng_value as i32 * 7) % 280);
                let y = 100 + ((rng_value as i32 * 13) % 220);

                // 70% chance for GoodTarget, 30% for BadTarget
                let circle_type = if rng_value % 10 < 7 {
                    CircleType::GoodTarget
                } else {
                    CircleType::BadTarget
                };

                *slot = Some(Circle::new(x, y, circle_type, self.last_update_ms));
                break;
            }
        }
    }

    /// Update battle state
    pub fn update_battle(&mut self, delta_ms: u32) {
        if self.battle_state != BattleState::Playing {
            return;
        }

        self.battle_elapsed += delta_ms;

        // Check if enemy is defeated
        if let Some(enemy) = &self.battle_enemy {
            if enemy.hp == 0 {
                esp_println::println!("[BATTLE] Enemy defeated! Ending battle early.");
                self.complete_battle();
                return;
            }
        }

        // Check if battle time is up
        if self.battle_elapsed >= self.battle_duration {
            self.complete_battle();
            return;
        }

        // Spawn new circles
        if self.last_update_ms >= self.battle_next_spawn {
            let rng = (self.last_update_ms % 255) as u8;
            self.spawn_battle_circle(rng);
            self.battle_next_spawn = self.last_update_ms + self.battle_spawn_interval;
        }

        // Check for expired circles
        for circle in &mut self.battle_circles {
            if let Some(c) = circle {
                if c.is_expired(self.last_update_ms) {
                    // Circle expired - if it was a BadTarget (enemy attack), hero takes damage
                    if c.circle_type == CircleType::BadTarget {
                        // Simple damage calculation: 10 base damage + level
                        let damage = if let Some(enemy) = &self.battle_enemy {
                            10 + enemy.level
                        } else {
                            10
                        };
                        self.hero.hp = self.hero.hp.saturating_sub(damage);
                        esp_println::println!("[BATTLE] Missed red circle! Took {} damage", damage);
                        self.battle_missed += 1;
                    } else {
                        // Missed green circle - counts as miss
                        self.battle_missed += 1;
                    }
                    *circle = None;

                    // Check for defeat (hero HP reaches 0)
                    if self.hero.hp == 0 {
                        esp_println::println!("[BATTLE] Hero defeated! HP = 0");
                        self.battle_state = BattleState::Defeat;
                        return;
                    }
                }
            }
        }
    }

    /// Handle circle click at position
    pub fn click_battle_circle(&mut self, x: i32, y: i32) -> bool {
        let mut enemy_defeated = false;

        for circle in &mut self.battle_circles {
            if let Some(c) = circle {
                if c.contains_point(x, y) {
                    match c.circle_type {
                        CircleType::GoodTarget => {
                            // Hit enemy! Simple damage: 5 + hero level
                            self.battle_score += 1;
                            if let Some(enemy) = &mut self.battle_enemy {
                                let damage = 5 + self.hero.level;
                                enemy.hp = enemy.hp.saturating_sub(damage);
                                esp_println::println!("[BATTLE] Hit green! Dealt {} damage. Enemy HP: {}", damage, enemy.hp);

                                // Check if enemy is defeated
                                if enemy.hp == 0 {
                                    esp_println::println!("[BATTLE] Enemy HP reached 0! Victory!");
                                    enemy_defeated = true;
                                }
                            }
                        }
                        CircleType::BadTarget => {
                            // Blocked enemy attack!
                            self.battle_score += 1;
                            esp_println::println!("[BATTLE] Blocked red attack!");
                        }
                    }
                    *circle = None;

                    // Complete battle after modifying circles
                    if enemy_defeated {
                        self.complete_battle();
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Complete battle and calculate rewards
    fn complete_battle(&mut self) {
        if let Some(enemy) = &self.battle_enemy {
            // Win if enemy HP is 0 or we have more hits than misses
            if enemy.hp == 0 || self.battle_score > self.battle_missed * 2 {
                self.battle_state = BattleState::Victory;
                // Award rewards based on score
                let exp_mult = (self.battle_score as u32).max(1);
                self.hero.add_exp(enemy.exp_reward * exp_mult / 5);
                self.hero.add_zeny(enemy.zeny_reward * exp_mult / 5);
            } else {
                self.battle_state = BattleState::Defeat;
            }
        }
    }

    /// Reset battle state
    pub fn reset_battle(&mut self) {
        self.battle_enemy = None;
        self.battle_state = BattleState::Idle;
        self.battle_circles = [None, None, None, None];
        self.battle_score = 0;
        self.battle_missed = 0;
        self.battle_elapsed = 0;
        self.battle_last_touch_x = 0;
        self.battle_last_touch_y = 0;
        self.battle_last_touch_time = 0;
    }
}
