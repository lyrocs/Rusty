use bevy_ecs::prelude::*;
use core::fmt::Write;
use heapless::String;

/// Game pages/screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePage {
    Overview,
    Farm,
    Rest,
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
        let parts: heapless::Vec<&str, 9> = data.split(',').collect();
        if parts.len() != 9 {
            return None;
        }

        // Parse job to a static string
        let job: &'static str = if parts[3] == "Novice" { "Novice" } else { "Swordsman" };
        let name: &'static str = if parts[3] == "Novice" { "Novice" } else { "Swordsman" };

        Some(Hero {
            name,
            level: parts[0].parse().ok()?,
            exp: parts[1].parse().ok()?,
            exp_to_next_level: parts[2].parse().ok()?,
            job,
            hp: parts[4].parse().ok()?,
            max_hp: parts[5].parse().ok()?,
            sp: parts[6].parse().ok()?,
            max_sp: parts[7].parse().ok()?,
            zeny: parts[8].parse().ok()?,
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

/// Main game state resource
#[derive(Resource)]
pub struct GameState {
    pub current_page: GamePage,
    pub hero: Hero,
    pub current_enemy: Option<Enemy>,
    pub farm_state: FarmState,
    pub farm_progress: u32,      // 0-60000 (60 seconds in milliseconds)
    pub farm_duration_ms: u32,   // 60000 ms = 1 minute
    pub rest_state: RestState,
    pub rest_progress: u32,      // Progress in milliseconds
    pub sp_regen_rate: u16,      // SP per second while resting
    pub menu_selection: u8,      // 0 = Overview, 1 = Farm, 2 = Rest, 3 = Save
    pub last_update_ms: u32,     // Last update time for progress tracking
    pub save_requested: bool,    // Flag to trigger save
    pub save_status_msg: Option<&'static str>, // Status message after save
    pub save_status_timeout: u32, // Time when save message should clear (0 = no message)
    pub fps: u32,                // Current FPS
    pub frame_count: u32,        // Total frames rendered
    pub last_fps_update_ms: u32, // Last time FPS was calculated
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
            rest_state: RestState::Resting,
            rest_progress: 0,
            sp_regen_rate: 5, // 5 SP per second
            menu_selection: 0,
            last_update_ms: 0,
            save_requested: false,
            save_status_msg: None,
            save_status_timeout: 0,
            fps: 0,
            frame_count: 0,
            last_fps_update_ms: 0,
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
    }

    /// Update rest progress
    pub fn update_rest_progress(&mut self, delta_ms: u32) {
        if self.rest_state == RestState::Resting {
            self.rest_progress += delta_ms;

            // Regenerate SP every second
            if self.rest_progress >= 1000 {
                let seconds = self.rest_progress / 1000;
                self.hero.regenerate_sp((seconds as u16) * self.sp_regen_rate);
                self.rest_progress %= 1000;

                // Check if SP is full
                if self.hero.sp >= self.hero.max_sp {
                    self.rest_state = RestState::FullSP;
                }
            }
        }
    }

    /// Get farm progress percentage
    pub fn farm_progress_percent(&self) -> u8 {
        ((self.farm_progress as u64 * 100) / self.farm_duration_ms as u64) as u8
    }
}
