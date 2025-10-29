/// JRPG turn-based battle game logic
///
/// Extension methods for GameState related to JRPG-style turn-based battles.

use crate::core::{GameState, GamePage};
use crate::combat::{
    ActiveStatusEffect, BattleAnimationPhase, CombatResult, Enemy, HeroAnimation, JrpgBattleMenu, JrpgBattleState,
    JrpgCombatant, JrpgSkill, MonsterAnimation, MonsterAttackedAnimation,
    SkillEffect, SkillType, StatusEffectType, calculate_jrpg_damage,
};
use crate::quest::QuestAction;

impl GameState {
    pub fn start_jrpg_battle(&mut self, enemy: Enemy) {
        esp_println::println!("[JRPG] Starting battle with {}", enemy.name);

        // Load skills for hero's job
        let hero_skills_array = JrpgSkill::get_skills_for_job(self.hero.job);
        let mut hero_skills = heapless::Vec::new();
        for skill in hero_skills_array {
            let _ = hero_skills.push(skill);
        }

        // Get equipment bonuses
        let weapon = &self.hero.equipped_weapon;
        let armor = &self.hero.equipped_armor;
        let accessory = &self.hero.equipped_accessory;

        // Calculate total stats with equipment bonuses
        let total_str = self.hero.base_str as i16 + weapon.str_bonus + armor.str_bonus + accessory.str_bonus;
        let total_agi = self.hero.base_agi as i16 + weapon.agi_bonus + armor.agi_bonus + accessory.agi_bonus;
        let total_vit = self.hero.base_vit as i16 + weapon.vit_bonus + armor.vit_bonus + accessory.vit_bonus;
        let total_int = self.hero.base_int as i16 + weapon.int_bonus + armor.int_bonus + accessory.int_bonus;
        let total_dex = self.hero.base_dex as i16 + weapon.dex_bonus + armor.dex_bonus + accessory.dex_bonus;
        let total_luk = self.hero.base_luk as i16 + weapon.luk_bonus + armor.luk_bonus + accessory.luk_bonus;

        // Calculate ATK with equipment
        let weapon_atk = weapon.total_atk();
        let total_atk = 10 + (total_str.max(0) as u16 * 2) + weapon_atk;

        // Calculate DEF with equipment
        let armor_def = armor.total_def();
        let total_def = 5 + total_vit.max(0) as u16 + armor_def;

        // Calculate max HP/SP with equipment
        let equipment_hp = armor.hp_bonus;
        let equipment_sp = weapon.sp_bonus + accessory.sp_bonus;

        // Create hero combatant from current hero stats + equipment
        self.jrpg_hero_combatant = Some(JrpgCombatant {
            name: self.hero.name,
            level: self.hero.level,
            hp: self.hero.hp,
            max_hp: self.hero.max_hp + equipment_hp,
            sp: self.hero.sp,
            max_sp: self.hero.max_sp + equipment_sp,
            attack: total_atk,
            defense: total_def,
            // Stats with equipment bonuses
            agility: total_agi.max(0) as u16,
            luck: total_luk.max(0) as u16,
            intelligence: total_int.max(0) as u16,
            dexterity: total_dex.max(0) as u16,
            active_effects: heapless::Vec::new(),
            available_skills: hero_skills,
        });

        // Create enemy combatant
        self.jrpg_enemy_combatant = Some(JrpgCombatant {
            name: enemy.name,
            level: enemy.level,
            hp: enemy.hp,
            max_hp: enemy.max_hp,
            sp: 0,       // Enemies don't use SP for now
            max_sp: 0,
            attack: enemy.attack,
            defense: enemy.defense,
            // Enemy stats based on level
            agility: enemy.level,
            luck: enemy.level / 2,
            intelligence: enemy.level,
            dexterity: enemy.level,
            active_effects: heapless::Vec::new(),
            available_skills: heapless::Vec::new(), // Enemies don't use skills yet
        });

        // Store original enemy for rewards
        self.battle_enemy = Some(enemy);

        // Set initial state - start directly in PlayerTurn so menu is visible
        self.jrpg_battle_state = JrpgBattleState::PlayerTurn;
        self.jrpg_battle_menu = JrpgBattleMenu::Main;
        self.jrpg_menu_selection = 0;
        self.jrpg_battle_message = None; // No message at start
        self.jrpg_battle_message_timer = 0;
        self.jrpg_damage_dealt = 0;
        self.jrpg_action_animation_timer = 0;

        // Switch to JRPG battle page
        self.current_page = GamePage::JrpgBattle;
        self.needs_redraw = true;

        // Set animations to idle
        self.hero_animation = HeroAnimation::Idle;
        self.hero_animation_frame = 0;
        self.hero_animation_started_ms = self.gif_animation_clock_ms;
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }

    /// Execute player attack in JRPG battle
    pub fn jrpg_player_attack(&mut self) {
        if let (Some(hero), Some(enemy)) = (&self.jrpg_hero_combatant, &mut self.jrpg_enemy_combatant) {
            // Generate random value for damage calculation
            let rng_value = (self.last_update_ms.wrapping_add(self.gif_animation_clock_ms) % 255) as u8;

            // Check for double attack (AGI-based)
            let double_attack_chance = (hero.agility / 10).min(30); // Max 30% at AGI 300+
            let double_attack_roll = (rng_value as u16 * 100) / 255;
            let is_double_attack = double_attack_roll < double_attack_chance;

            // Calculate damage with variance, crits, lucky strikes, and miss chance
            let (damage, combat_result) = calculate_jrpg_damage(
                hero.attack,
                hero.luck,
                hero.dexterity,
                enemy.defense,
                enemy.agility,
                rng_value,
            );

            enemy.hp = enemy.hp.saturating_sub(damage);
            self.jrpg_damage_dealt = damage;
            self.jrpg_last_combat_result = combat_result;

            // Set damage animation position (near enemy at x=80, y=150)
            self.jrpg_damage_animation_timer = 1000; // 1 second animation
            self.jrpg_damage_x = 80 + 32; // Center of enemy GIF (64x64)
            self.jrpg_damage_y = 150 + 20; // Slightly below center

            let result_str = match combat_result {
                CombatResult::Critical => " CRITICAL!",
                CombatResult::Lucky => " LUCKY STRIKE!",
                CombatResult::Miss => " MISS!",
                CombatResult::Normal => "",
            };

            esp_println::println!("[JRPG] Hero dealt {} damage{}. Enemy HP: {}/{}",
                damage, result_str, enemy.hp, enemy.max_hp);

            // Set attack animation
            self.hero_animation = HeroAnimation::Attacking;
            self.hero_animation_frame = 0;
            self.hero_animation_started_ms = self.gif_animation_clock_ms;

            // Enemy hit animation
            self.monster_attacked_animation = MonsterAttackedAnimation::Attacked;
            self.monster_attacked_frame = 0;
            self.monster_attacked_started_ms = self.gif_animation_clock_ms;

            self.needs_redraw = true;

            // Handle double attack
            if is_double_attack && enemy.hp > 0 {
                // Second hit with different RNG
                let rng_value2 = (rng_value.wrapping_add(17)) % 255;
                let (damage2, _combat_result2) = calculate_jrpg_damage(
                    hero.attack,
                    hero.luck,
                    hero.dexterity,
                    enemy.defense,
                    enemy.agility,
                    rng_value2,
                );

                enemy.hp = enemy.hp.saturating_sub(damage2);
                self.jrpg_damage_dealt += damage2; // Add to total damage display

                esp_println::println!("[JRPG] Double Attack! Hero dealt additional {} damage. Enemy HP: {}/{}",
                    damage2, enemy.hp, enemy.max_hp);
            }
        }
    }

    /// Execute enemy attack in JRPG battle
    pub fn jrpg_enemy_attack(&mut self) {
        if let (Some(enemy), Some(hero)) = (&self.jrpg_enemy_combatant, &mut self.jrpg_hero_combatant) {
            // Generate random value for damage calculation
            let rng_value = (self.last_update_ms.wrapping_add(self.gif_animation_clock_ms * 2) % 255) as u8;

            // Calculate damage with variance, crits, lucky strikes, and miss chance
            let (damage, combat_result) = calculate_jrpg_damage(
                enemy.attack,
                enemy.luck,
                enemy.dexterity,
                hero.defense,
                hero.agility,
                rng_value,
            );

            // Store combat result for UI display
            self.jrpg_last_combat_result = combat_result;

            // Only apply damage if attack hit
            if combat_result != CombatResult::Miss {
                hero.hp = hero.hp.saturating_sub(damage);
                self.jrpg_damage_dealt = damage;

                // Hero hit animation (only when hit)
                self.hero_animation = HeroAnimation::Attacked;
                self.hero_animation_frame = 0;
                self.hero_animation_started_ms = self.gif_animation_clock_ms;
            } else {
                // Miss - no damage
                self.jrpg_damage_dealt = 0;
            }

            // Set damage animation position (near hero at x=240, y=150)
            self.jrpg_damage_animation_timer = 1000; // 1 second animation
            self.jrpg_damage_x = 240 + 32; // Center of hero GIF (64x64)
            self.jrpg_damage_y = 150 + 20; // Slightly below center

            let result_str = match combat_result {
                CombatResult::Critical => " CRITICAL!",
                CombatResult::Lucky => " LUCKY STRIKE!",
                CombatResult::Miss => " MISS!",
                CombatResult::Normal => "",
            };

            esp_println::println!("[JRPG] Enemy attack: {} damage{}. Hero HP: {}/{}",
                damage, result_str, hero.hp, hero.max_hp);

            // Set monster attack animation (always plays even on miss)
            self.monster_animation = MonsterAnimation::Attacking;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;

            self.needs_redraw = true;
        }
    }

    /// Execute player skill in JRPG battle
    pub fn jrpg_player_use_skill(&mut self, skill_index: usize) {
        // First, get skill and validate
        let skill = if let Some(hero) = &self.jrpg_hero_combatant {
            if skill_index >= hero.available_skills.len() {
                esp_println::println!("[JRPG] Invalid skill index");
                return;
            }

            let skill = hero.available_skills[skill_index];

            // Check SP cost
            if hero.sp < skill.sp_cost {
                esp_println::println!("[JRPG] Not enough SP! Need {}, have {}", skill.sp_cost, hero.sp);
                self.jrpg_battle_message = Some("Not enough SP!");
                self.jrpg_battle_message_timer = 2000;
                self.needs_redraw = true;
                return;
            }

            skill
        } else {
            return;
        };

        // Consume SP
        if let Some(hero_mut) = &mut self.jrpg_hero_combatant {
            hero_mut.sp = hero_mut.sp.saturating_sub(skill.sp_cost);
        }

        // Get hero stats needed for calculations (copied values)
        let (hero_attack, hero_luck, hero_intelligence) = if let Some(hero) = &self.jrpg_hero_combatant {
            (hero.attack, hero.luck, hero.intelligence)
        } else {
            return;
        };

        // Check if enemy exists
        if self.jrpg_enemy_combatant.is_none() {
            return;
        }

        // Generate random value for skill execution
        let rng_value = (self.last_update_ms.wrapping_add(self.gif_animation_clock_ms) % 255) as u8;

        esp_println::println!("[JRPG] Hero uses skill: {} (SP cost: {})", skill.name, skill.sp_cost);

        // Execute skill based on type
        match skill.skill_type {
            SkillType::Physical => {
                // Physical skill: use ATK with skill power multiplier
                let skill_damage = ((hero_attack as u32 * skill.power as u32) / 100) as u16;

                // Get enemy defense for damage calculation
                let enemy_def = if let Some(enemy) = &self.jrpg_enemy_combatant {
                    enemy.defense
                } else {
                    return;
                };

                // Skills never miss - calculate damage directly without miss check
                let base_damage = if skill_damage > enemy_def {
                    skill_damage - (enemy_def / 2)
                } else {
                    1
                };

                // Apply damage variance (±20%)
                let variance_percent = 80 + ((rng_value as u32 * 40) / 255) as u16;
                let varied_damage = ((base_damage as u32 * variance_percent as u32) / 100) as u16;

                // Calculate crit chance (skills can still crit)
                let crit_chance = 5 + (hero_luck / 20);
                let crit_roll = (rng_value as u16 * 100) / 255;

                let (damage, combat_result) = if crit_roll < 2 {
                    (skill_damage * 2, CombatResult::Lucky)
                } else if crit_roll < (2 + crit_chance) {
                    let crit_damage = ((skill_damage as u32 * 140) / 100) as u16;
                    (crit_damage, CombatResult::Critical)
                } else {
                    (varied_damage.max(1), CombatResult::Normal)
                };

                // Apply damage to enemy
                if let Some(enemy) = &mut self.jrpg_enemy_combatant {
                    enemy.hp = enemy.hp.saturating_sub(damage);
                    esp_println::println!("[JRPG] Skill dealt {} damage. Enemy HP: {}/{}", damage, enemy.hp, enemy.max_hp);
                }

                self.jrpg_damage_dealt = damage;
                self.jrpg_last_combat_result = combat_result;
            },
            SkillType::Magic => {
                // Magic skill: use INT with skill power multiplier (ignores DEF)
                let magic_damage = ((hero_intelligence as u32 * skill.power as u32) / 100) as u16;
                // Apply variance
                let variance_percent = 80 + ((rng_value as u32 * 40) / 255) as u16;
                let damage = ((magic_damage as u32 * variance_percent as u32) / 100) as u16;

                // Apply damage to enemy
                if let Some(enemy) = &mut self.jrpg_enemy_combatant {
                    enemy.hp = enemy.hp.saturating_sub(damage);
                    esp_println::println!("[JRPG] Magic dealt {} damage. Enemy HP: {}/{}", damage, enemy.hp, enemy.max_hp);
                }

                self.jrpg_damage_dealt = damage;
                self.jrpg_last_combat_result = CombatResult::Normal;
            },
            SkillType::Healing => {
                // Heal skill: restore HP
                if let Some(hero_mut) = &mut self.jrpg_hero_combatant {
                    let heal_amount = ((hero_intelligence as u32 * skill.power as u32) / 100) as u16;
                    let old_hp = hero_mut.hp;
                    hero_mut.hp = (hero_mut.hp + heal_amount).min(hero_mut.max_hp);
                    let actual_heal = hero_mut.hp - old_hp;

                    self.jrpg_damage_dealt = actual_heal;
                    self.jrpg_last_combat_result = CombatResult::Normal;

                    esp_println::println!("[JRPG] Healed {} HP. Hero HP: {}/{}", actual_heal, hero_mut.hp, hero_mut.max_hp);
                }
            },
            SkillType::Buff | SkillType::Debuff | SkillType::Utility => {
                // Apply effect (buffs/debuffs/utility)
                esp_println::println!("[JRPG] Skill effect applied: {:?}", skill.effect);
                self.jrpg_damage_dealt = 0;
                self.jrpg_last_combat_result = CombatResult::Normal;
            },
        }

        // Set damage animation position
        if skill.skill_type == SkillType::Healing {
            // Heal animation on hero
            self.jrpg_damage_x = 240 + 32;
            self.jrpg_damage_y = 150 + 20;
        } else {
            // Damage animation on enemy
            self.jrpg_damage_x = 80 + 32;
            self.jrpg_damage_y = 150 + 20;
        }
        self.jrpg_damage_animation_timer = 1000;

        // Set attack animation
        self.hero_animation = HeroAnimation::Attacking;
        self.hero_animation_frame = 0;
        self.hero_animation_started_ms = self.gif_animation_clock_ms;

        // Enemy hit animation (if damage skill)
        if skill.skill_type == SkillType::Physical || skill.skill_type == SkillType::Magic {
            self.monster_attacked_animation = MonsterAttackedAnimation::Attacked;
            self.monster_attacked_frame = 0;
            self.monster_attacked_started_ms = self.gif_animation_clock_ms;
        }

        self.needs_redraw = true;
    }

    /// Try to run from battle (50% chance)
    pub fn jrpg_try_run(&mut self) -> bool {
        let rng = (self.last_update_ms % 100) as u8;
        let success = rng < 50; // 50% chance

        if success {
            self.jrpg_battle_state = JrpgBattleState::Escaped;
            esp_println::println!("[JRPG] Escaped successfully");
        } else {
            esp_println::println!("[JRPG] Failed to escape");
        }

        self.needs_redraw = true;
        success
    }

    /// End JRPG battle and return to map
    pub fn end_jrpg_battle(&mut self) {
        // Sync hero HP/SP back to main hero
        if let Some(hero_combatant) = &self.jrpg_hero_combatant {
            self.hero.hp = hero_combatant.hp;
            self.hero.sp = hero_combatant.sp;
        }

        // Award rewards on victory
        if self.jrpg_battle_state == JrpgBattleState::Victory {
            // Extract enemy data before borrowing self mutably
            let (enemy_id, base_exp, zeny_earned) = if let Some(enemy) = &self.battle_enemy {
                (enemy.id, enemy.base_exp, enemy.zeny_reward)
            } else {
                (0, 0, 0)
            };

            if enemy_id > 0 {
                self.hero.add_exp(base_exp);
                self.hero.add_zeny(zeny_earned);

                // Update quest progress - monster killed
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::MonsterKilled { enemy_id },
                );

                // Update quest progress - battle completed
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::BattleCompleted,
                );

                // Update quest progress - zeny earned
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::ZenyEarned {
                        amount: zeny_earned,
                    },
                );

                // Roll for item drops
                let rng_value = (self.last_update_ms % 255) as u8;
                let drop_rate = 30; // 30% drop chance
                if rng_value < drop_rate {
                    if let Some((item_id, item_name)) = self.roll_item_drop(enemy_id, rng_value) {
                        let quantity = 1;
                        self.hero.add_item(item_id, item_name, quantity);
                    }
                }

                esp_println::println!(
                    "[JRPG] Victory! Gained {} EXP, {} Zeny",
                    base_exp, zeny_earned
                );
            }
        }

        // Clean up battle state
        self.jrpg_hero_combatant = None;
        self.jrpg_enemy_combatant = None;
        self.battle_enemy = None;
        self.jrpg_battle_message = None;
        self.jrpg_menu_selection = 0;

        // Return to map
        self.current_page = GamePage::Map;
        self.needs_redraw = true;
    }
}
