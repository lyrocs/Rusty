/// Combat models
///
/// Core combat entities including Enemy and basic combat states.
use crate::tamagotchi::get_enemy_data;

/// Enemy data (based on data/enemies.json)
/// Note: JSON files in data/ folder serve as source of truth
/// This struct contains runtime enemy data used in battles
#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: u32, // Enemy ID from JSON
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub attack: u16,      // Added from JSON
    pub defense: u16,     // Added from JSON
    pub base_exp: u32,    // Renamed from exp_reward
    pub zeny_reward: u32, // Calculated zeny (base_exp / 10)
}

impl Enemy {
    /// Get a random enemy based on hero level (from enemies.json)
    /// Uses generated data from build.rs
    pub fn random_for_level(_hero_level: u16, rng_value: u8) -> Self {
        // Enemy IDs from enemies.json: Poring=1002, Fabre=1007, Hornet=1004, Thief Bug=1051
        let enemy_id = match rng_value % 4 {
            0 => 1002, // Poring
            1 => 1007, // Fabre
            2 => 1004, // Hornet
            _ => 1051, // Thief Bug
        };

        // Use generated function to get enemy data from JSON
        get_enemy_data(enemy_id).expect("Enemy ID should exist in enemies.json")
    }

    /// Get enemy by ID from JSON data (convenience function)
    pub fn from_id(id: u32) -> Option<Self> {
        get_enemy_data(id)
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

/// IDLE farming state (new continuous farming system)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleFarmState {
    Idle,           // Not farming
    Active,         // Currently farming
    Cooldown,       // Died, waiting for cooldown
}

/// IDLE farming session data (with real-time combat simulation)
#[derive(Debug, Clone)]
pub struct IdleFarmSession {
    pub state: IdleFarmState,
    pub map_id: u32,
    pub enemy_id: u32,
    pub start_time_ms: u32,
    pub last_update_ms: u32,

    // Session statistics
    pub monsters_killed: u32,
    pub zeny_earned: u32,
    pub exp_gained: u32,
    pub items_collected: u16,

    // Hero combat state
    pub current_hp: u16,
    pub hero_level: u16,                // Hero level (for hit rate and HP regen)
    pub hero_dex: u16,                  // Hero DEX (for hit rate)
    pub hero_vit: u16,                  // Hero VIT (for HP regen)
    pub hero_attack_delay_ms: u32,      // MS between hero attacks (based on ASPD)
    pub last_hero_attack_ms: u32,       // When hero last attacked
    pub next_hero_attack_ms: u32,       // When hero will attack next
    pub hero_attack_pending: bool,      // Attack animation started, damage not yet applied
    pub hero_damage_apply_ms: u32,      // When to apply the pending damage
    pub pending_hero_damage: u16,       // Damage to apply when animation completes
    pub pending_hero_miss: bool,        // Whether pending attack will miss

    // Enemy combat state
    pub current_enemy_hp: u16,          // Current enemy HP
    pub enemy_max_hp: u16,              // Enemy max HP (for respawn)
    pub enemy_level: u16,               // Enemy level (for hit rate calculations)
    pub enemy_attack_delay_ms: u32,     // MS between enemy attacks (based on enemy ASPD)
    pub last_enemy_attack_ms: u32,      // When enemy last attacked
    pub next_enemy_attack_ms: u32,      // When enemy will attack next
    pub enemy_attack_pending: bool,     // Attack animation started, damage not yet applied
    pub enemy_damage_apply_ms: u32,     // When to apply the pending damage
    pub pending_enemy_damage: u16,      // Damage to apply when animation completes
    pub pending_enemy_miss: bool,       // Whether pending attack will miss
    pub enemy_dying: bool,              // True if enemy is playing death animation
    pub enemy_death_complete_ms: u32,   // When death animation completes
    pub enemy_spawning: bool,           // True if enemy is respawning (after death anim)
    pub enemy_spawn_complete_ms: u32,   // When new enemy will spawn

    // Skill system
    pub skill_cooldown_ms: u32,         // Skill cooldown duration
    pub last_skill_use_ms: u32,         // When hero last used skill
    pub next_skill_use_ms: u32,         // When skill will be available

    // HP regeneration (separate from combat damage)
    pub hp_regen_rate: f32,             // HP regen per second (from VIT)
    pub last_hp_regen_ms: u32,          // When HP last regenerated

    // Damage display tracking (for visual feedback)
    pub last_hero_damage: u16,          // Last damage hero dealt to enemy
    pub last_enemy_damage: u16,         // Last damage enemy dealt to hero
    pub hero_attack_missed: bool,       // True if hero's last attack missed
    pub enemy_attack_missed: bool,      // True if enemy's last attack missed
    pub last_skill_used: bool,          // True if last hero attack was a skill

    // Calculated rates (per minute) - kept for display
    pub kills_per_minute: f32,
    pub zeny_per_minute: f32,
    pub damage_per_minute: f32,
    pub regen_per_minute: f32,

    // Old kill tracking (deprecated but kept for compatibility)
    pub time_since_last_kill_ms: u32,
    pub ms_per_kill: u32,

    // Cooldown (after death)
    pub cooldown_end_ms: u32,
}

impl IdleFarmSession {
    pub fn new(map_id: u32, enemy_id: u32, start_time_ms: u32, current_hp: u16,
               kills_per_min: f32, zeny_per_min: f32, damage_per_min: f32, regen_per_min: f32,
               hero_level: u16, hero_agi: u16, hero_vit: u16, hero_dex: u16,
               enemy_hp: u16, enemy_level: u16) -> Self {
        let ms_per_kill = if kills_per_min > 0.0 {
            (60_000.0 / kills_per_min) as u32
        } else {
            10_000 // Default to 10 seconds if calculation fails
        };

        // Calculate hero attack delay based on AGI (ASPD formula)
        // Slower attacks to allow animations to complete
        // Formula: 4000 - (AGI * 20) = attack delay in ms
        // Min delay = 2500ms (allows 750ms+ for attack animation + reaction time)
        let hero_attack_delay_ms = (4000 - (hero_agi as u32 * 20)).max(2500); // 2500-4000ms

        // Calculate enemy attack delay based on level
        // Higher level enemies attack faster but still slow enough for animations
        // Formula: 5000 - (level * 30) = attack delay in ms
        let enemy_attack_delay_ms = (5000 - (enemy_level as u32 * 30)).max(3000).min(5000); // 3000-5000ms

        // HP regen rate calculation:
        // Base: VIT / 10 HP per second
        // Level bonus: Level / 20 HP per second
        // Total: (VIT/10 + Level/20) HP per second, minimum 1.0 HP/sec
        // This means at Level 1 VIT 1: ~0.15 HP/sec, at Level 50 VIT 50: ~7.5 HP/sec
        let hp_regen_rate = ((hero_vit as f32 / 10.0) + (hero_level as f32 / 20.0)).max(1.0);

        // Skill cooldown: 10 seconds
        let skill_cooldown_ms = 10_000;

        Self {
            state: IdleFarmState::Active,
            map_id,
            enemy_id,
            start_time_ms,
            last_update_ms: start_time_ms,
            monsters_killed: 0,
            zeny_earned: 0,
            exp_gained: 0,
            items_collected: 0,
            // Hero combat
            current_hp,
            hero_level,
            hero_dex,
            hero_vit,
            hero_attack_delay_ms,
            last_hero_attack_ms: start_time_ms,
            next_hero_attack_ms: start_time_ms + hero_attack_delay_ms,
            hero_attack_pending: false,
            hero_damage_apply_ms: 0,
            pending_hero_damage: 0,
            pending_hero_miss: false,
            // Enemy combat
            current_enemy_hp: enemy_hp,
            enemy_max_hp: enemy_hp,
            enemy_level,
            enemy_attack_delay_ms,
            last_enemy_attack_ms: start_time_ms,
            next_enemy_attack_ms: start_time_ms + enemy_attack_delay_ms,
            enemy_attack_pending: false,
            enemy_damage_apply_ms: 0,
            pending_enemy_damage: 0,
            pending_enemy_miss: false,
            enemy_dying: false,
            enemy_death_complete_ms: 0,
            enemy_spawning: false,
            enemy_spawn_complete_ms: 0,
            // Skills
            skill_cooldown_ms,
            last_skill_use_ms: 0,
            next_skill_use_ms: start_time_ms + skill_cooldown_ms, // Available after first cooldown
            // HP regen
            hp_regen_rate,
            last_hp_regen_ms: start_time_ms,
            // Damage display
            last_hero_damage: 0,
            last_enemy_damage: 0,
            hero_attack_missed: false,
            enemy_attack_missed: false,
            last_skill_used: false,
            // Display rates
            kills_per_minute: kills_per_min,
            zeny_per_minute: zeny_per_min,
            damage_per_minute: damage_per_min,
            regen_per_minute: regen_per_min,
            // Legacy
            time_since_last_kill_ms: 0,
            ms_per_kill,
            cooldown_end_ms: 0,
        }
    }

    pub fn duration_ms(&self, current_time_ms: u32) -> u32 {
        current_time_ms.saturating_sub(self.start_time_ms)
    }

    pub fn is_active(&self) -> bool {
        self.state == IdleFarmState::Active
    }

    pub fn is_in_cooldown(&self) -> bool {
        self.state == IdleFarmState::Cooldown
    }
}

/// Rest state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestState {
    Resting,
    FullSP,
    Complete,
}

/// Farm duration options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarmDuration {
    OneMinute,   // 60 seconds, 20 SP, 1.0x multiplier per minute
    FiveMinutes, // 300 seconds, 90 SP, 0.90x multiplier per minute (4.5x total)
    TenMinutes,  // 600 seconds, 180 SP, 0.85x multiplier per minute (8.5x total)
}

impl FarmDuration {
    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        match self {
            FarmDuration::OneMinute => 60_000,
            FarmDuration::FiveMinutes => 300_000,
            FarmDuration::TenMinutes => 600_000,
        }
    }

    /// Get SP cost
    pub fn sp_cost(&self) -> u16 {
        match self {
            FarmDuration::OneMinute => 20,
            FarmDuration::FiveMinutes => 90,
            FarmDuration::TenMinutes => 180,
        }
    }

    /// Get efficiency multiplier per minute (for reward penalty on longer farms)
    pub fn multiplier_per_minute(&self) -> f32 {
        match self {
            FarmDuration::OneMinute => 1.0,
            FarmDuration::FiveMinutes => 0.90,
            FarmDuration::TenMinutes => 0.85,
        }
    }

    /// Get total multiplier for the entire duration
    pub fn total_multiplier(&self) -> f32 {
        match self {
            FarmDuration::OneMinute => 1.0,
            FarmDuration::FiveMinutes => 4.5,  // 0.90 × 5
            FarmDuration::TenMinutes => 8.5,   // 0.85 × 10
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            FarmDuration::OneMinute => "1 MIN",
            FarmDuration::FiveMinutes => "5 MIN",
            FarmDuration::TenMinutes => "10 MIN",
        }
    }
}

/// Farm efficiency rating based on hero power vs enemy power
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfficiencyRating {
    Excellent, // ≥2.0 ratio, 2.5x multiplier, ★★★
    Good,      // 1.5-2.0 ratio, 1.5x multiplier, ★★
    Fair,      // 1.0-1.5 ratio, 1.0x multiplier, ★
    Risky,     // 0.7-1.0 ratio, 0.7x multiplier, ⚠
    Impossible, // <0.7 ratio, blocked, ✗
}

impl EfficiencyRating {
    /// Calculate rating from power ratio
    pub fn from_power_ratio(ratio: f32) -> Self {
        if ratio >= 2.0 {
            EfficiencyRating::Excellent
        } else if ratio >= 1.5 {
            EfficiencyRating::Good
        } else if ratio >= 1.0 {
            EfficiencyRating::Fair
        } else if ratio >= 0.7 {
            EfficiencyRating::Risky
        } else {
            EfficiencyRating::Impossible
        }
    }

    /// Get efficiency multiplier for rewards
    pub fn multiplier(&self) -> f32 {
        match self {
            EfficiencyRating::Excellent => 2.5,
            EfficiencyRating::Good => 1.5,
            EfficiencyRating::Fair => 1.0,
            EfficiencyRating::Risky => 0.7,
            EfficiencyRating::Impossible => 0.0,
        }
    }

    /// Get display icon
    pub fn icon(&self) -> &'static str {
        match self {
            EfficiencyRating::Excellent => "***",
            EfficiencyRating::Good => "**",
            EfficiencyRating::Fair => "*",
            EfficiencyRating::Risky => "!",
            EfficiencyRating::Impossible => "X",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            EfficiencyRating::Excellent => "Excellent",
            EfficiencyRating::Good => "Good",
            EfficiencyRating::Fair => "Fair",
            EfficiencyRating::Risky => "Risky",
            EfficiencyRating::Impossible => "Impossible",
        }
    }

    /// Check if farming is allowed
    pub fn is_allowed(&self) -> bool {
        !matches!(self, EfficiencyRating::Impossible)
    }
}

/// Combat result type (for JRPG battles)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResult {
    Normal,
    Critical,
    Lucky,
    Miss,
}

/// MVP Battle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MvpBattleState {
    Idle,          // No battle active
    Start,         // Battle starting
    Playing,       // Battle in progress
    Victory,       // Battle won
    Defeat,        // Battle lost
}

/// MVP Battle phase (difficulty progression)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MvpBattlePhase {
    Phase1,  // 100-70% HP: Learning phase
    Phase2,  // 70-30% HP: Boss enraged
    Phase3,  // 30-0% HP: Berserk mode
}

impl MvpBattlePhase {
    /// Get phase based on boss HP percentage
    pub fn from_hp_percent(hp_percent: u8) -> Self {
        if hp_percent > 70 {
            MvpBattlePhase::Phase1
        } else if hp_percent > 30 {
            MvpBattlePhase::Phase2
        } else {
            MvpBattlePhase::Phase3
        }
    }
}

/// MVP Battle rank (based on performance)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MvpBattleRank {
    S,  // Perfect performance
    A,  // Excellent
    B,  // Good
    C,  // Average
    D,  // Poor
}

impl MvpBattleRank {
    /// Calculate rank based on combat time, perfect hits, and health remaining
    pub fn calculate(combat_time_ms: u32, perfect_hits: u16, hero_hp_percent: u8) -> Self {
        let mut score = 0;

        // Time bonus (faster = better)
        if combat_time_ms < 60_000 {  // < 1 minute
            score += 3;
        } else if combat_time_ms < 120_000 {  // < 2 minutes
            score += 2;
        } else if combat_time_ms < 180_000 {  // < 3 minutes
            score += 1;
        }

        // Perfect hits bonus
        if perfect_hits > 20 {
            score += 3;
        } else if perfect_hits > 10 {
            score += 2;
        } else if perfect_hits > 5 {
            score += 1;
        }

        // Health remaining bonus
        if hero_hp_percent > 80 {
            score += 3;
        } else if hero_hp_percent > 50 {
            score += 2;
        } else if hero_hp_percent > 20 {
            score += 1;
        }

        // Map score to rank
        match score {
            9 => MvpBattleRank::S,
            7..=8 => MvpBattleRank::A,
            5..=6 => MvpBattleRank::B,
            3..=4 => MvpBattleRank::C,
            _ => MvpBattleRank::D,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            MvpBattleRank::S => "S",
            MvpBattleRank::A => "A",
            MvpBattleRank::B => "B",
            MvpBattleRank::C => "C",
            MvpBattleRank::D => "D",
        }
    }

    /// Get reward multiplier based on rank
    pub fn reward_multiplier(&self) -> f32 {
        match self {
            MvpBattleRank::S => 2.0,
            MvpBattleRank::A => 1.5,
            MvpBattleRank::B => 1.2,
            MvpBattleRank::C => 1.0,
            MvpBattleRank::D => 0.8,
        }
    }
}

/// MVP Battle skill definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MvpSkillType {
    Bash,     // Quick tap for burst damage (3s cooldown)
    Provoke,  // Long press to reduce boss DEF (8s cooldown)
    Potion,   // Swipe up for emergency heal (6s cooldown)
}

impl MvpSkillType {
    /// Get cooldown in milliseconds
    pub fn cooldown_ms(&self) -> u32 {
        match self {
            MvpSkillType::Bash => 3_000,
            MvpSkillType::Provoke => 8_000,
            MvpSkillType::Potion => 6_000, // Balanced at 6s
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            MvpSkillType::Bash => "Bash",
            MvpSkillType::Provoke => "Provoke",
            MvpSkillType::Potion => "Potion",
        }
    }
}

/// Active MVP skill cooldown
#[derive(Debug, Clone, Copy)]
pub struct MvpSkillCooldown {
    pub skill_type: MvpSkillType,
    pub last_used_ms: u32,  // When skill was last used
}

impl MvpSkillCooldown {
    /// Create new cooldown tracker
    pub fn new(skill_type: MvpSkillType, last_used_ms: u32) -> Self {
        Self {
            skill_type,
            last_used_ms,
        }
    }

    /// Check if skill is ready to use
    pub fn is_ready(&self, current_ms: u32) -> bool {
        current_ms >= self.last_used_ms + self.skill_type.cooldown_ms()
    }

    /// Get remaining cooldown in milliseconds
    pub fn remaining_ms(&self, current_ms: u32) -> u32 {
        let ready_at = self.last_used_ms + self.skill_type.cooldown_ms();
        ready_at.saturating_sub(current_ms)
    }

    /// Get cooldown progress (0.0 to 1.0)
    pub fn progress(&self, current_ms: u32) -> f32 {
        if self.is_ready(current_ms) {
            return 1.0;
        }
        let elapsed = current_ms.saturating_sub(self.last_used_ms) as f32;
        let total = self.skill_type.cooldown_ms() as f32;
        (elapsed / total).min(1.0)
    }
}
