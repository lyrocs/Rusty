//! Combat State
//!
//! Manages turn-based combat state for dungeon floor battles.
//! Each floor has a single enemy (no wave system).

use crate::game::core::{Element, Monster, SkillEffectType};
use crate::game::calculations::combat::{
    update_atk_bar, update_skl_bar_after_attack, update_swap_cooldown,
};
use crate::game::calculations::damage::calculate_final_damage;

/// Combat state for turn-based floor battles
pub struct CombatState {
    // Player team
    pub player_monsters: Vec<Monster>,
    pub active_index: u8,
    pub swap_cooldowns: [f32; 3],

    // Enemy (single enemy per floor)
    pub enemy: Monster,
    pub enemy_aura: Option<Element>, // Element aura (persists until reaction)

    // Combat bars (0.0 to 1.0) - legacy, may be removed
    pub player_atk_bar: f32,
    pub player_skl_bar: f32,
    pub enemy_atk_bar: f32,
    pub enemy_skl_bar: f32,

    // Status effects
    pub player_stunned: f32, // Stun time remaining
    pub enemy_stunned: f32,

    // Run info
    pub current_floor: u16,
    pub is_boss_floor: bool,
    pub crystals_earned: u32,
    pub xp_earned: u32,

    // Combat result
    pub combat_ended: bool,
    pub player_won: bool,
}

impl CombatState {
    /// Create new combat state for a floor battle
    pub fn new(player_monsters: Vec<Monster>, enemy: Monster, current_floor: u16) -> Self {
        Self {
            player_monsters,
            active_index: 0,
            swap_cooldowns: [0.0; 3],
            enemy,
            enemy_aura: None,
            player_atk_bar: 0.0,
            player_skl_bar: 0.0,
            enemy_atk_bar: 0.0,
            enemy_skl_bar: 0.0,
            player_stunned: 0.0,
            enemy_stunned: 0.0,
            current_floor,
            is_boss_floor: current_floor > 0 && current_floor % 10 == 0,
            crystals_earned: 0,
            xp_earned: 0,
            combat_ended: false,
            player_won: false,
        }
    }

    /// Create new combat state with boss floor flag
    pub fn for_floor(
        player_monsters: Vec<Monster>,
        enemy: Monster,
        current_floor: u16,
        is_boss_floor: bool,
    ) -> Self {
        Self {
            player_monsters,
            active_index: 0,
            swap_cooldowns: [0.0; 3],
            enemy,
            enemy_aura: None,
            player_atk_bar: 0.0,
            player_skl_bar: 0.0,
            enemy_atk_bar: 0.0,
            enemy_skl_bar: 0.0,
            player_stunned: 0.0,
            enemy_stunned: 0.0,
            current_floor,
            is_boss_floor,
            crystals_earned: 0,
            xp_earned: 0,
            combat_ended: false,
            player_won: false,
        }
    }

    /// Get the active player monster
    pub fn active_monster(&self) -> Option<&Monster> {
        self.player_monsters.get(self.active_index as usize)
    }

    /// Get the active player monster mutably
    pub fn active_monster_mut(&mut self) -> Option<&mut Monster> {
        self.player_monsters.get_mut(self.active_index as usize)
    }

    /// Check if all player monsters are dead
    pub fn all_players_dead(&self) -> bool {
        self.player_monsters.iter().all(|m| !m.is_alive())
    }

    /// Update combat state for one frame
    /// Returns combat events that occurred this frame
    pub fn update(&mut self, delta_time: f32) -> Vec<CombatEvent> {
        let mut events = Vec::new();

        // Don't update if combat ended
        if self.combat_ended {
            return events;
        }

        // Update swap cooldowns
        for cooldown in &mut self.swap_cooldowns {
            *cooldown = update_swap_cooldown(*cooldown, delta_time);
        }

        // Update stun timers
        self.player_stunned = (self.player_stunned - delta_time).max(0.0);
        self.enemy_stunned = (self.enemy_stunned - delta_time).max(0.0);

        // Aura persists until reaction (no time decay)

        // Player ATK bar update (if not stunned)
        if self.player_stunned <= 0.0 {
            if let Some(monster) = self.active_monster() {
                if monster.is_alive() {
                    let spd = monster.spd;
                    self.player_atk_bar = update_atk_bar(self.player_atk_bar, spd, delta_time);

                    // Player auto-attack
                    if self.player_atk_bar >= 1.0 {
                        if let Some(event) = self.execute_player_attack() {
                            events.push(event);
                        }
                    }
                }
            }
        }

        // Enemy ATK bar update (if not stunned and alive)
        if self.enemy_stunned <= 0.0 && self.enemy.is_alive() {
            self.enemy_atk_bar = update_atk_bar(self.enemy_atk_bar, self.enemy.spd, delta_time);

            // Enemy auto-attack
            if self.enemy_atk_bar >= 1.0 {
                if let Some(event) = self.execute_enemy_attack() {
                    events.push(event);
                }
            }
        }

        // Check win/lose conditions
        if !self.enemy.is_alive() {
            // Award rewards for this floor (increased for better progression)
            let base_crystals = 10 + (self.current_floor as u32 / 3);
            let base_xp = 50 + (self.current_floor as u32 * 15);

            // Boss floors give bonus rewards
            if self.is_boss_floor {
                self.crystals_earned += base_crystals * 3;
                self.xp_earned += base_xp * 2;
            } else {
                self.crystals_earned += base_crystals;
                self.xp_earned += base_xp;
            }

            // Victory!
            self.combat_ended = true;
            self.player_won = true;
            events.push(CombatEvent::Victory {
                crystals: self.crystals_earned,
                xp: self.xp_earned
            });
        } else if self.all_players_dead() {
            self.combat_ended = true;
            self.player_won = false;
            events.push(CombatEvent::Defeat);
        }

        events
    }

    /// Execute player auto-attack
    fn execute_player_attack(&mut self) -> Option<CombatEvent> {
        let monster = self.active_monster()?;
        if !monster.is_alive() {
            return None;
        }

        let atk = monster.atk;
        let element = monster.element;
        let def = self.enemy.def;
        let enemy_element = self.enemy.element;

        // Check for reaction (returns multiplier and optional reaction name)
        let (reaction_mult, reaction_name, heal_amount) = self.check_reaction(element);
        let damage = calculate_final_damage(atk, def, element, enemy_element, reaction_mult);

        // Apply damage to enemy
        self.enemy.take_damage(damage);

        // Apply aura to enemy (persists until next reaction)
        self.enemy_aura = Some(element);

        // Update bars
        self.player_atk_bar = 0.0;
        self.player_skl_bar = update_skl_bar_after_attack(self.player_skl_bar);

        Some(CombatEvent::PlayerAttack {
            damage,
            element,
            reaction: reaction_name,
            heal_amount,
        })
    }

    /// Execute enemy auto-attack
    fn execute_enemy_attack(&mut self) -> Option<CombatEvent> {
        if !self.enemy.is_alive() {
            return None;
        }

        // Get enemy stats first (before borrowing player monster)
        let enemy_atk = self.enemy.atk;
        let enemy_element = self.enemy.element;

        // Get player monster stats
        let monster = self.active_monster()?;
        if !monster.is_alive() {
            return None;
        }
        let def = monster.def;
        let monster_element = monster.element;

        // Calculate damage (no reactions for enemy for now)
        let damage = calculate_final_damage(enemy_atk, def, enemy_element, monster_element, 1.0);

        // Apply damage to player monster (get mutable reference now)
        if let Some(monster) = self.active_monster_mut() {
            monster.take_damage(damage);
        }

        // Update bars
        self.enemy_atk_bar = 0.0;
        self.enemy_skl_bar = update_skl_bar_after_attack(self.enemy_skl_bar);

        Some(CombatEvent::EnemyAttack { damage, element: enemy_element })
    }

    /// Check for elemental reaction and return (multiplier, reaction_name, heal_amount)
    fn check_reaction(&mut self, attack_element: Element) -> (f32, Option<String>, Option<u16>) {
        if let Some(aura_element) = self.enemy_aura {
            // Check for reactions based on element_config.json
            let (mult, name, heal) = match (aura_element, attack_element) {
                // Water aura + Fire = VAPORIZE (x2 damage)
                (Element::Water, Element::Fire) => (2.0, Some("VAPORIZE"), None),
                // Fire aura + Water = VAPORIZE (x2 damage)
                (Element::Fire, Element::Water) => (2.0, Some("VAPORIZE"), None),
                // Water aura + Thunder = ELECTROCUTE (stun 1 sec)
                (Element::Water, Element::Thunder) => {
                    self.enemy_stunned = 1.0;
                    (1.0, Some("ELECTROCUTE"), None)
                },
                // Water aura + Earth = BLOOM (heal team 15%)
                (Element::Water, Element::Earth) => {
                    // Calculate heal amount (15% of max HP for each alive monster)
                    let heal_amount = self.calculate_bloom_heal();
                    (1.0, Some("BLOOM"), Some(heal_amount))
                },
                _ => (1.0, None, None),
            };

            if name.is_some() {
                // Clear aura after reaction
                self.enemy_aura = None;
            }

            (mult, name.map(|s| s.to_string()), heal)
        } else {
            (1.0, None, None)
        }
    }

    /// Calculate and apply BLOOM heal (15% of max HP to all alive monsters)
    fn calculate_bloom_heal(&mut self) -> u16 {
        let mut total_healed = 0u16;

        for monster in &mut self.player_monsters {
            if monster.is_alive() {
                let heal_amount = (monster.hp_max as f32 * 0.15) as u16;
                let old_hp = monster.hp_current;
                monster.hp_current = (monster.hp_current + heal_amount).min(monster.hp_max);
                total_healed += monster.hp_current - old_hp;
            }
        }

        log::info!("BLOOM reaction: healed team for {} total HP", total_healed);
        total_healed
    }

    /// Use player skill (requires full SKL bar)
    /// Uses the first equipped skill (slot 0) for now - will be updated for multi-skill battle UI
    pub fn use_skill(&mut self) -> Option<CombatEvent> {
        if self.player_skl_bar < 1.0 {
            return None;
        }

        let monster = self.active_monster()?;
        if !monster.is_alive() {
            return None;
        }

        // Get first equipped skill (fallback for old system)
        let skill = monster.equipped_skills.first()?.clone();
        let element = monster.element;
        let skill_name = skill.name.clone();
        let skill_effect = skill.effect_type.clone();
        let skill_value = skill.effect_value;

        // Handle different skill types
        match skill_effect {
            SkillEffectType::Heal => {
                // Heal skill - heal active monster by percentage of max HP
                let monster = self.active_monster_mut()?;
                let heal_amount = (monster.hp_max as f32 * skill_value) as u16;
                let old_hp = monster.hp_current;
                monster.hp_current = (monster.hp_current + heal_amount).min(monster.hp_max);
                let actual_heal = monster.hp_current - old_hp;

                // Reset skill bar
                self.player_skl_bar = 0.0;

                log::info!("Heal skill used: {} healed for {} HP", skill_name, actual_heal);

                Some(CombatEvent::PlayerSkillHeal {
                    skill_name,
                    heal_amount: actual_heal,
                })
            }
            _ => {
                // Damage skills
                let atk = monster.atk;
                let (reaction_mult, _reaction_name, _heal) = self.check_reaction(element);
                let damage = calculate_final_damage(
                    atk,
                    self.enemy.def,
                    element,
                    self.enemy.element,
                    skill_value * reaction_mult
                );

                // Apply damage
                self.enemy.take_damage(damage);

                // Apply aura from skill (persists until reaction)
                self.enemy_aura = Some(element);

                // Reset skill bar
                self.player_skl_bar = 0.0;

                Some(CombatEvent::PlayerSkill {
                    skill_name,
                    damage,
                    element
                })
            }
        }
    }

    /// Swap to a different monster
    pub fn swap_monster(&mut self, target_index: u8) -> Option<CombatEvent> {
        if target_index >= self.player_monsters.len() as u8 {
            return None;
        }

        // Check cooldown
        if self.swap_cooldowns[target_index as usize] > 0.0 {
            return None;
        }

        // Check if target is alive
        if !self.player_monsters[target_index as usize].is_alive() {
            return None;
        }

        let old_index = self.active_index;
        self.active_index = target_index;

        // Set cooldown on the swapped-out monster
        self.swap_cooldowns[old_index as usize] = 3.0;

        // Reset ATK bar on swap
        self.player_atk_bar = 0.0;

        Some(CombatEvent::MonsterSwap {
            from_index: old_index,
            to_index: target_index
        })
    }
}

/// Combat events for UI feedback
#[derive(Debug, Clone)]
pub enum CombatEvent {
    /// Player monster attacked
    PlayerAttack {
        damage: u16,
        element: Element,
        reaction: Option<String>,  // Reaction name (e.g., "VAPORIZE", "BLOOM")
        heal_amount: Option<u16>,  // Heal from BLOOM reaction
    },
    /// Enemy attacked
    EnemyAttack {
        damage: u16,
        element: Element,
    },
    /// Player used damage skill
    PlayerSkill {
        skill_name: String,
        damage: u16,
        element: Element,
    },
    /// Player used heal skill
    PlayerSkillHeal {
        skill_name: String,
        heal_amount: u16,
    },
    /// Monster swapped
    MonsterSwap {
        from_index: u8,
        to_index: u8,
    },
    /// Combat won
    Victory {
        crystals: u32,
        xp: u32,
    },
    /// Combat lost
    Defeat,
}
