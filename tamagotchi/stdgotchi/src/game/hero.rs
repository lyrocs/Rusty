//! Hero system
//!
//! Manages hero stats, level, job progression, and HP/SP

use super::stats::Stats;
use serde::{Deserialize, Serialize};

/// Hero job types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Job {
    Novice,
    Swordsman,
    Knight,
}

impl Job {
    /// Get the next job in progression
    pub fn next_job(&self) -> Option<Job> {
        match self {
            Job::Novice => Some(Job::Swordsman),
            Job::Swordsman => Some(Job::Knight),
            Job::Knight => None,
        }
    }

    /// Get the level requirement for this job
    pub fn min_level(&self) -> u32 {
        match self {
            Job::Novice => 1,
            Job::Swordsman => 10,
            Job::Knight => 40,
        }
    }

    /// Get job name as string
    pub fn name(&self) -> &'static str {
        match self {
            Job::Novice => "Novice",
            Job::Swordsman => "Swordsman",
            Job::Knight => "Knight",
        }
    }
}

/// Hero character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hero {
    pub name: String,
    pub job: Job,
    pub level: u32,
    pub exp: u64,
    pub exp_to_next_level: u64,
    pub stats: Stats,
    
    // Current HP/SP
    pub current_hp: u32,
    pub max_hp: u32,
    pub current_sp: u32,
    pub max_sp: u32,
    
    // Combat stats
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub crit_rate: f32,
}

impl Hero {
    /// Create a new hero starting as Novice level 1
    pub fn new() -> Self {
        let stats = Stats::new();
        let level = 1;
        let max_hp = stats.calculate_max_hp(50, level);
        let max_sp = stats.calculate_max_sp(20, level);
        
        Self {
            name: "Hero".to_string(),
            job: Job::Novice,
            level,
            exp: 0,
            exp_to_next_level: Self::calculate_exp_for_level(2),
            stats,
            current_hp: max_hp,
            max_hp,
            current_sp: max_sp,
            max_sp,
            atk: stats.calculate_atk(),
            def: stats.calculate_def(),
            hit: stats.calculate_hit(level),
            flee: stats.calculate_flee(level),
            crit_rate: stats.calculate_crit_rate(),
        }
    }

    /// Calculate EXP required for a given level
    pub fn calculate_exp_for_level(level: u32) -> u64 {
        // Simple exponential formula: level^3 * 10
        ((level as u64).pow(3)) * 10
    }

    /// Gain experience points
    pub fn gain_exp(&mut self, exp: u64) {
        self.exp += exp;
        
        // Check for level up
        while self.exp >= self.exp_to_next_level && self.level < 99 {
            self.level_up();
        }
    }

    /// Level up the hero
    fn level_up(&mut self) {
        self.level += 1;
        self.exp -= self.exp_to_next_level;
        self.exp_to_next_level = Self::calculate_exp_for_level(self.level + 1);
        
        // Increase stats based on job
        self.apply_stat_growth();
        
        // Recalculate derived stats
        self.recalculate_stats();
        
        // Restore HP/SP on level up
        self.current_hp = self.max_hp;
        self.current_sp = self.max_sp;
        
        // Check for automatic job change
        self.check_job_change();
        
        log::info!("Level up! Now level {}", self.level);
    }

    /// Apply stat growth based on current job
    fn apply_stat_growth(&mut self) {
        match self.job {
            Job::Novice => {
                self.stats.str += 1;
                self.stats.agi += 1;
                self.stats.vit += 1;
                self.stats.int += 1;
                self.stats.dex += 1;
                self.stats.luk += 1;
            }
            Job::Swordsman => {
                self.stats.str += 2;
                self.stats.agi += 1;
                self.stats.vit += 2;
                self.stats.int += 0;
                self.stats.dex += 1;
                self.stats.luk += 1;
            }
            Job::Knight => {
                self.stats.str += 3;
                self.stats.agi += 2;
                self.stats.vit += 3;
                self.stats.int += 1;
                self.stats.dex += 2;
                self.stats.luk += 1;
            }
        }
    }

    /// Check if hero should change jobs automatically
    fn check_job_change(&mut self) {
        if self.level >= Job::Knight.min_level() && self.job == Job::Swordsman {
            self.change_job(Job::Knight);
        } else if self.level >= Job::Swordsman.min_level() && self.job == Job::Novice {
            self.change_job(Job::Swordsman);
        }
    }

    /// Change to a new job
    fn change_job(&mut self, new_job: Job) {
        log::info!("Job change: {} -> {}", self.job.name(), new_job.name());
        self.job = new_job;
        
        // Give bonus stats on job change
        match new_job {
            Job::Swordsman => {
                self.stats.str += 5;
                self.stats.vit += 5;
            }
            Job::Knight => {
                self.stats.str += 10;
                self.stats.vit += 10;
                self.stats.agi += 5;
            }
            _ => {}
        }
        
        self.recalculate_stats();
    }

    /// Recalculate all derived stats
    pub fn recalculate_stats(&mut self) {
        let base_hp = match self.job {
            Job::Novice => 50,
            Job::Swordsman => 100,
            Job::Knight => 200,
        };
        let base_sp = match self.job {
            Job::Novice => 20,
            Job::Swordsman => 30,
            Job::Knight => 50,
        };
        
        self.max_hp = self.stats.calculate_max_hp(base_hp, self.level);
        self.max_sp = self.stats.calculate_max_sp(base_sp, self.level);
        self.atk = self.stats.calculate_atk();
        self.def = self.stats.calculate_def();
        self.hit = self.stats.calculate_hit(self.level);
        self.flee = self.stats.calculate_flee(self.level);
        self.crit_rate = self.stats.calculate_crit_rate();
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.current_hp {
            self.current_hp = 0;
        } else {
            self.current_hp -= damage;
        }
    }

    /// Heal HP
    pub fn heal(&mut self, amount: u32) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    /// Check if hero is alive
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// Get attack interval in milliseconds
    pub fn get_attack_interval(&self) -> u64 {
        self.stats.calculate_attack_interval()
    }

    /// Get HP percentage
    pub fn hp_percentage(&self) -> f32 {
        (self.current_hp as f32 / self.max_hp as f32) * 100.0
    }

    /// Get SP percentage
    pub fn sp_percentage(&self) -> f32 {
        (self.current_sp as f32 / self.max_sp as f32) * 100.0
    }
}

impl Default for Hero {
    fn default() -> Self {
        Self::new()
    }
}
