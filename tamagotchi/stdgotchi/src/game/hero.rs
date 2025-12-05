//! Hero System
//!
//! Represents the player's hero character with job class and stats.

use serde::{Deserialize, Serialize};
use super::element_system::Element;
use super::job_system::JobClass;
use super::expedition::{Card, HeroState};
use super::skill::{EquippedSkillSlot, ActiveSkill};

/// Hero character with stats and job progression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hero {
    // Identity
    pub name: String,
    pub job: JobClass,

    // Core Stats
    pub level: u32,
    pub experience: u32,
    pub experience_to_next_level: u32,

    // Health
    pub current_health: i32,
    pub max_health: i32,

    // Base Stats (Primary attributes)
    pub strength: u16,      // Affects physical attack
    pub dexterity: u16,     // Affects hit, flee, aspd
    pub intelligence: u16,  // Affects magic attack
    pub vitality: u16,      // Affects HP and defense
    pub agility: u16,       // Affects speed and flee

    // Derived Stats (Calculated from base stats + job bonuses)
    pub attack: u16,
    pub defense: u16,
    pub magic_attack: u16,
    pub magic_defense: u16,
    pub speed: u16,
    pub hit: u16,
    pub flee: u16,
    pub critical: u16,
    pub aspd: f32,          // Attack speed

    // Element affinity (could be based on job)
    pub element: Element,

    // Expedition system
    pub state: HeroState,
    pub cards: Vec<Card>,

    // Skill system - 3 slots for equipped skill cards
    #[serde(default)]
    pub equipped_skill_slots: [EquippedSkillSlot; 3],
    /// Active skills during battle (tracks cooldowns)
    #[serde(skip)]
    pub active_skills: Vec<ActiveSkill>,
}

impl Hero {
    /// Create a new hero with the Novice job class
    pub fn new(name: String) -> Self {
        let mut hero = Self {
            name,
            job: JobClass::Novice,
            level: 1,
            experience: 0,
            experience_to_next_level: 100,
            current_health: 100,
            max_health: 100,

            // Starting base stats for Novice
            strength: 5,
            dexterity: 5,
            intelligence: 5,
            vitality: 5,
            agility: 5,

            // Will be calculated
            attack: 0,
            defense: 0,
            magic_attack: 0,
            magic_defense: 0,
            speed: 0,
            hit: 0,
            flee: 0,
            critical: 0,
            aspd: 1.0,

            element: Element::Neutral,

            // Expedition system
            state: HeroState::Ready,
            cards: Vec::new(),

            // Skill system
            equipped_skill_slots: Default::default(),
            active_skills: Vec::new(),
        };

        hero.recalculate_stats();
        hero
    }

    /// Recalculate all derived stats based on base stats and job bonuses
    pub fn recalculate_stats(&mut self) {
        let job_bonus = self.job.get_stat_bonus();

        // Apply base stats + job bonuses
        let effective_str = self.strength + job_bonus.strength;
        let effective_dex = self.dexterity + job_bonus.dexterity;
        let effective_int = self.intelligence + job_bonus.intelligence;
        let effective_vit = self.vitality + job_bonus.vitality;
        let effective_agi = self.agility + job_bonus.agility;

        // Calculate derived stats
        self.attack = (effective_str * 2) + (effective_dex / 2);
        self.defense = (effective_vit * 2) + (effective_agi / 2);
        self.magic_attack = (effective_int * 2) + (effective_dex / 2);
        self.magic_defense = (effective_int * 2) + (effective_vit / 2);
        self.speed = effective_agi + (effective_dex / 2);
        self.hit = effective_dex + (self.level as u16 * 2);
        self.flee = effective_agi + (self.level as u16);
        self.critical = (effective_dex / 3) + job_bonus.critical_bonus;
        self.aspd = 1.0 + (effective_agi as f32 * 0.01) + (effective_dex as f32 * 0.005);

        // Max HP based on vitality and level
        self.max_health = ((effective_vit as i32 * 10) + (self.level as i32 * 50)) as i32;

        // Update element based on job
        self.element = self.job.get_element();
    }

    /// Gain experience and check for level up
    pub fn gain_experience(&mut self, exp: u32) -> bool {
        self.experience += exp;

        if self.experience >= self.experience_to_next_level {
            self.level_up();
            return true;
        }
        false
    }

    /// Level up the hero
    fn level_up(&mut self) {
        self.level += 1;
        self.experience = 0;
        self.experience_to_next_level = self.calculate_exp_needed(self.level);

        // Stat growth based on job
        let growth = self.job.get_stat_growth();
        self.strength += growth.strength;
        self.dexterity += growth.dexterity;
        self.intelligence += growth.intelligence;
        self.vitality += growth.vitality;
        self.agility += growth.agility;

        // Recalculate all derived stats
        self.recalculate_stats();

        // Heal to full on level up
        self.current_health = self.max_health;
    }

    /// Calculate experience needed for a given level
    fn calculate_exp_needed(&self, level: u32) -> u32 {
        // Exponential growth: 100 * level^1.5
        (100.0 * (level as f32).powf(1.5)) as u32
    }

    /// Check if hero can evolve to a new job
    pub fn can_evolve(&self) -> bool {
        self.job.can_evolve(self.level)
    }

    /// Get available job evolutions
    pub fn get_available_evolutions(&self) -> Vec<JobClass> {
        self.job.get_evolutions(self.level)
    }

    /// Evolve to a new job class
    pub fn evolve_job(&mut self, new_job: JobClass) -> Result<(), String> {
        let available = self.get_available_evolutions();

        if !available.contains(&new_job) {
            return Err("Cannot evolve to this job class".to_string());
        }

        self.job = new_job;
        self.recalculate_stats();

        // Full heal on job change
        self.current_health = self.max_health;

        Ok(())
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: i32) {
        self.current_health = (self.current_health - damage).max(0);
    }

    /// Heal
    pub fn heal(&mut self, amount: i32) {
        self.current_health = (self.current_health + amount).min(self.max_health);
    }

    /// Check if hero is alive
    pub fn is_alive(&self) -> bool {
        self.current_health > 0
    }

    /// Rest to restore health
    pub fn rest(&mut self) {
        let heal_amount = self.max_health / 4; // Heal 25% per rest
        self.heal(heal_amount);
    }

    // ============================================================================
    // SKILL SYSTEM
    // ============================================================================

    /// Equip a skill card to a specific slot (0, 1, or 2)
    pub fn equip_skill(&mut self, slot_index: usize, card_monster_id: u32, skill_id: u32) -> Result<(), String> {
        if slot_index >= 3 {
            return Err("Invalid slot index (must be 0-2)".to_string());
        }

        // Check if this card is already equipped in another slot
        for (i, slot) in self.equipped_skill_slots.iter().enumerate() {
            if i != slot_index && slot.card_monster_id == Some(card_monster_id) {
                return Err("This card is already equipped in another slot".to_string());
            }
        }

        self.equipped_skill_slots[slot_index].equip(card_monster_id, skill_id);
        Ok(())
    }

    /// Unequip a skill from a slot
    pub fn unequip_skill(&mut self, slot_index: usize) {
        if slot_index < 3 {
            self.equipped_skill_slots[slot_index].unequip();
        }
    }

    /// Get skill ID for a specific slot (if equipped)
    pub fn get_equipped_skill(&self, slot_index: usize) -> Option<u32> {
        if slot_index < 3 {
            self.equipped_skill_slots[slot_index].skill_id
        } else {
            None
        }
    }

    /// Get all equipped skill IDs
    pub fn get_all_equipped_skills(&self) -> Vec<u32> {
        self.equipped_skill_slots
            .iter()
            .filter_map(|slot| slot.skill_id)
            .collect()
    }

    /// Initialize active skills at the start of battle
    pub fn initialize_battle_skills(&mut self) {
        self.active_skills.clear();
        for slot in &self.equipped_skill_slots {
            if let Some(skill_id) = slot.skill_id {
                self.active_skills.push(ActiveSkill::new(skill_id));
            }
        }
    }

    /// Update all skill cooldowns (call every frame)
    pub fn update_skill_cooldowns(&mut self, delta_time: f32) {
        for skill in &mut self.active_skills {
            skill.update(delta_time);
        }
    }

    /// Check if a skill at a given index is ready
    pub fn is_skill_ready(&self, skill_index: usize) -> bool {
        self.active_skills
            .get(skill_index)
            .map(|s| s.is_ready())
            .unwrap_or(false)
    }

    /// Use a skill at a given index (puts it on cooldown)
    pub fn use_skill(&mut self, skill_index: usize, cooldown_seconds: f32) {
        if let Some(skill) = self.active_skills.get_mut(skill_index) {
            skill.use_skill(cooldown_seconds);
        }
    }

    /// Get remaining cooldown for a skill at a given index
    pub fn get_skill_cooldown(&self, skill_index: usize) -> Option<f32> {
        self.active_skills
            .get(skill_index)
            .map(|s| s.remaining_cooldown)
    }

    /// Count how many skill cards are equipped
    pub fn equipped_skill_count(&self) -> usize {
        self.equipped_skill_slots
            .iter()
            .filter(|slot| !slot.is_empty())
            .count()
    }

    /// Check if a card with the given monster ID is equipped
    pub fn is_card_equipped(&self, card_monster_id: u32) -> bool {
        self.equipped_skill_slots
            .iter()
            .any(|slot| slot.card_monster_id == Some(card_monster_id))
    }

    /// Add experience points to the hero, leveling up if necessary
    pub fn add_experience(&mut self, exp: u32) {
        self.experience += exp;
        while self.experience >= self.experience_to_next_level {
            self.experience -= self.experience_to_next_level;
            self.level += 1;
            self.experience_to_next_level = 100 + (self.level * 50);
            log::info!("Hero leveled up to level {}!", self.level);
            self.recalculate_stats();
        }
    }

    /// Set the hero to KO state with recovery time
    pub fn set_ko(&mut self, recovery_seconds: u64) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.state = HeroState::KO {
            recovery_time: now + recovery_seconds,
        };
        self.current_health = 0;
        log::info!("Hero is KO! Recovery in {} seconds", recovery_seconds);
    }

    /// Add a card to the hero's collection
    pub fn add_card(&mut self, card: Card) {
        // Check if already have this card
        if self.cards.iter().any(|c| c.monster_id == card.monster_id) {
            log::info!("Already have {} card", card.name);
        } else {
            log::info!("New card acquired: {}", card.name);
            self.cards.push(card);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_creation() {
        let hero = Hero::new("TestHero".to_string());
        assert_eq!(hero.name, "TestHero");
        assert_eq!(hero.level, 1);
        assert!(matches!(hero.job, JobClass::Novice));
        assert!(hero.is_alive());
    }

    #[test]
    fn test_level_up() {
        let mut hero = Hero::new("TestHero".to_string());
        let initial_level = hero.level;
        hero.gain_experience(100);
        assert_eq!(hero.level, initial_level + 1);
        assert_eq!(hero.current_health, hero.max_health); // Full heal on level up
    }

    #[test]
    fn test_damage_and_healing() {
        let mut hero = Hero::new("TestHero".to_string());
        let max_hp = hero.max_health;

        hero.take_damage(30);
        assert_eq!(hero.current_health, max_hp - 30);

        hero.heal(20);
        assert_eq!(hero.current_health, max_hp - 10);
    }
}
