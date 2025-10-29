
// Re-export core types for backward compatibility
pub use crate::core::{GamePage, GameState, MapId};

// Re-export hero types for backward compatibility
pub use crate::hero::{
    Equipment, EquipmentSlot, EquipmentType, Hero, Inventory, InventoryExt, Item,
};

// Re-export quest types for backward compatibility
pub use crate::quest::{ActiveQuest, QuestAction, QuestData, QuestObjective, QuestReward, QuestType};

// Re-export combat types for backward compatibility
pub use crate::combat::{
    ActiveStatusEffect, BattleAnimationPhase, BattleState, Circle, CircleType, CombatResult,
    Enemy, FarmState, HeroAnimation, JrpgBattleAction, JrpgBattleMenu, JrpgBattleState,
    JrpgCombatant, JrpgSkill, MonsterAnimation, MonsterAttackedAnimation, RestState,
    SkillEffect, SkillType, StatusEffectType, calculate_jrpg_damage, get_map_background,
    get_monster_attacked_gif,
};

// Game data functions are re-exported from tamagotchi::game_data
use crate::tamagotchi::{
    get_city_npcs, get_map_connections,
    get_map_enemies, get_map_name, is_city,
};

/// Location type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    City,  // Cities with NPCs (Prontera, etc)
    Field, // Monster fields for hunting
}

/// Map helper functions (uses generated data from maps.json)
pub struct MapHelper;

impl MapHelper {
    pub fn name(map_id: MapId) -> &'static str {
        get_map_name(map_id)
    }

    pub fn location_type(map_id: MapId) -> LocationType {
        if is_city(map_id) {
            LocationType::City
        } else {
            LocationType::Field
        }
    }

    /// Get available exits from this location (from maps.json)
    pub fn exits(map_id: MapId) -> heapless::Vec<MapExit, 4> {
        let (north, south, east, west) = get_map_connections(map_id);
        let mut exits = heapless::Vec::new();

        if let Some(dest) = north {
            exits
                .push(MapExit {
                    direction: "North",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = south {
            exits
                .push(MapExit {
                    direction: "South",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = east {
            exits
                .push(MapExit {
                    direction: "East",
                    destination: dest,
                })
                .ok();
        }
        if let Some(dest) = west {
            exits
                .push(MapExit {
                    direction: "West",
                    destination: dest,
                })
                .ok();
        }

        exits
    }

    /// Get enemy IDs for a map (from maps.json)
    pub fn enemies(map_id: MapId) -> heapless::Vec<u32, 8> {
        get_map_enemies(map_id)
    }

    /// Get NPCs for city locations (from maps.json)
    pub fn npcs(map_id: MapId) -> heapless::Vec<&'static str, 8> {
        get_city_npcs(map_id)
    }
}

/// Exit from a location
#[derive(Debug, Clone, Copy)]
pub struct MapExit {
    pub direction: &'static str,
    pub destination: MapId,
}


/// Calculate damage for JRPG battles with variance, crits, lucky strikes, and miss chance

impl GameState {
    /// Roll for item drop based on enemy ID
    fn roll_item_drop(&self, enemy_id: u32, _rng_value: u8) -> Option<(u32, &'static str)> {
        // Simple item drop table based on enemy
        match enemy_id {
            1002 => Some((512, "Apple")),           // Poring drops Apple
            1007 => Some((705, "Clover")),          // Fabre drops Clover
            1004 => Some((518, "Honey")),           // Hornet drops Honey
            1051 => Some((955, "Worm Peeling")),   // Thief Bug drops Worm Peeling
            _ => None,
        }
    }

    /// Start farming with a new enemy
    pub fn start_farming(&mut self, enemy: Enemy) {
        if self.hero.use_sp(20) {
            self.current_enemy = Some(enemy);
            self.farm_state = FarmState::Fighting;
            self.farm_progress = 0;
            self.current_page = GamePage::Farm;
            // Reset animation to Idle when starting new farm
            self.monster_animation = MonsterAnimation::Idle;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;
            self.needs_redraw = true;
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
            let enemy_id = enemy.id;
            let zeny_earned = enemy.zeny_reward;

            self.hero.add_exp(enemy.base_exp);
            self.hero.add_zeny(zeny_earned);
            self.farm_state = FarmState::Victory;

            // Update quest progress - monster killed
            crate::tamagotchi::quest_system::update_quest_progress(
                self,
                QuestAction::MonsterKilled { enemy_id },
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
            let drops = crate::tamagotchi::game_data::roll_drops(enemy_id, rng_value);

            // Clear previous drops and store new ones
            self.last_drops.clear();

            for (item_id, item_name, quantity) in drops.iter() {
                if self.hero.add_item(*item_id, item_name, *quantity) {
                    esp_println::println!("[DROPS] Got {} x{}", item_name, quantity);
                    self.last_drops.push((*item_id, item_name, *quantity)).ok();
                } else {
                    esp_println::println!("[DROPS] Inventory full! Lost {}", item_name);
                }
            }
        }
    }

    /// Reset farming state
    pub fn reset_farming(&mut self) {
        self.current_enemy = None;
        self.farm_state = FarmState::Idle;
        self.farm_progress = 0;
        // Set cooldown to prevent immediate re-touch (300ms)
        self.farm_touch_cooldown = 300;
        // Reset animation to Idle
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }

    /// Initialize rest state based on current HP/SP
    pub fn init_rest_state(&mut self) {
        // Check if already fully recovered
        if self.hero.sp >= self.hero.max_sp && self.hero.hp >= self.hero.max_hp {
            self.rest_state = RestState::FullSP;
        } else {
            self.rest_state = RestState::Resting;
        }
        self.rest_progress = 0;
    }

    /// Update rest progress
    pub fn update_rest_progress(&mut self, delta_ms: u32) {
        if self.rest_state == RestState::Resting {
            self.rest_progress += delta_ms;

            // Regenerate SP and HP every second
            if self.rest_progress >= 1000 {
                let seconds = self.rest_progress / 1000;

                // Regenerate SP (5 SP per second by default)
                self.hero
                    .regenerate_sp((seconds as u16) * self.sp_regen_rate);

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
        if self.hero.use_sp(30) {
            // Battle costs 30 SP (more than farming)
            self.battle_enemy = Some(enemy);
            self.battle_state = BattleState::Playing;
            self.battle_circles = [None, None, None, None];
            self.battle_score = 0;
            self.battle_missed = 0;
            self.battle_combo = 0;
            self.battle_elapsed = 0;
            self.battle_next_spawn = self.last_update_ms + 500; // First spawn in 500ms
            self.current_page = GamePage::Battle; // Switch to battle page

            // Reset animation to Idle when starting new battle
            self.monster_animation = MonsterAnimation::Idle;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;
            self.needs_redraw = true;
        }
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

                *slot = Some(Circle::new(x, y, 30, circle_type, self.last_update_ms, 2000));
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
            self.needs_redraw = true; // Redraw when new circle spawns
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
                        self.battle_missed += 1;
                        // Reset combo on missing red circle
                        self.battle_combo = 0;
                    } else {
                        // Missed green circle - counts as miss and resets combo
                        self.battle_missed += 1;
                        self.battle_combo = 0;
                    }
                    *circle = None;
                    self.needs_redraw = true; // Redraw when circle expires

                    // Check for defeat (hero HP reaches 0)
                    if self.hero.hp == 0 {
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
                            // Increase combo on green hit
                            self.battle_combo += 1;
                            self.battle_score += 1;

                            if let Some(enemy) = &mut self.battle_enemy {
                                // Base damage: 5 + hero level
                                let base_damage = 5 + self.hero.level;

                                // Combo multiplier: 1.0x at combo 1, increases by 0.2x per combo
                                // Caps at 3.0x (combo 11+)
                                let combo_multiplier =
                                    (1.0 + (self.battle_combo - 1) as f32 * 0.2).min(3.0);
                                let damage = (base_damage as f32 * combo_multiplier) as u16;

                                enemy.hp = enemy.hp.saturating_sub(damage);
                                esp_println::println!(
                                    "[BATTLE] Hit green! Combo: {}x ({}x multiplier) Dealt {} damage. Enemy HP: {}",
                                    self.battle_combo,
                                    combo_multiplier,
                                    damage,
                                    enemy.hp
                                );

                                // Check if enemy is defeated
                                if enemy.hp == 0 {
                                    enemy_defeated = true;
                                }
                            }
                        }
                        CircleType::BadTarget => {
                            // Blocked enemy attack - doesn't increase or decrease combo
                            self.battle_score += 1;
                            esp_println::println!(
                                "[BATTLE] Blocked red attack! Combo maintained at {}",
                                self.battle_combo
                            );
                        }
                    }
                    *circle = None;
                    self.needs_redraw = true; // Redraw when circle is clicked

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
            // Win only if enemy HP is 0 (defeated before timeout)
            if enemy.hp == 0 {
                self.battle_state = BattleState::Victory;
                // Award rewards based on score
                let exp_mult = (self.battle_score as u32).max(1);
                self.hero.add_exp(enemy.base_exp * exp_mult / 5);
                self.hero.add_zeny(enemy.zeny_reward * exp_mult / 5);

                // Roll for item drops
                let rng_value = (self.last_update_ms % 255) as u8;
                let drops = crate::tamagotchi::game_data::roll_drops(enemy.id, rng_value);

                // Clear previous drops and store new ones
                self.last_drops.clear();

                for (item_id, item_name, quantity) in drops.iter() {
                    if self.hero.add_item(*item_id, item_name, *quantity) {
                        esp_println::println!("[DROPS] Got {} x{}", item_name, quantity);
                        self.last_drops.push((*item_id, item_name, *quantity)).ok();
                    } else {
                        esp_println::println!("[DROPS] Inventory full! Lost {}", item_name);
                    }
                }
            } else {
                self.battle_state = BattleState::Defeat;
                // Clear drops on defeat
                self.last_drops.clear();
            }
            // Record when battle ended to prevent accidental clicks
            self.battle_end_time = self.last_update_ms;
        }
    }

    /// Reset battle state
    pub fn reset_battle(&mut self) {
        self.battle_enemy = None;
        self.battle_state = BattleState::Idle;
        self.battle_circles = [None, None, None, None];
        self.battle_score = 0;
        self.battle_missed = 0;
        self.battle_combo = 0;
        self.battle_elapsed = 0;
        self.battle_last_touch_x = 0;
        self.battle_last_touch_y = 0;
        self.battle_last_touch_time = 0;
        self.battle_end_time = 0;

        // Reset animation to Idle
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }

    /// Start JRPG battle with an enemy
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

            // Update combo counter
            if combat_result != CombatResult::Miss {
                self.jrpg_combo_count = self.jrpg_combo_count.saturating_add(1);
                if self.jrpg_combo_count >= 3 {
                    self.jrpg_combo_ready = true;
                }
            } else {
                self.jrpg_combo_count = 0;
                self.jrpg_combo_ready = false;
            }

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

            esp_println::println!("[JRPG] Hero dealt {} damage{} (Combo: {}). Enemy HP: {}/{}",
                damage, result_str, self.jrpg_combo_count, enemy.hp, enemy.max_hp);

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
                let rng_value2 = (rng_value.wrapping_add(self.jrpg_combo_count)) % 255;
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

            // Only apply damage and reset combo if attack hit
            if combat_result != CombatResult::Miss {
                hero.hp = hero.hp.saturating_sub(damage);
                self.jrpg_damage_dealt = damage;

                // Reset combo on player damage
                self.jrpg_combo_count = 0;
                self.jrpg_combo_ready = false;

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

                // Update combo
                if combat_result != CombatResult::Miss {
                    self.jrpg_combo_count = self.jrpg_combo_count.saturating_add(1);
                    if self.jrpg_combo_count >= 3 {
                        self.jrpg_combo_ready = true;
                    }
                } else {
                    self.jrpg_combo_count = 0;
                    self.jrpg_combo_ready = false;
                }
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
