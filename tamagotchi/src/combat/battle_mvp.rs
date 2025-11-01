/// MVP Battle system - Semi-active combat with auto-attacks and manual skill timing
///
/// Features:
/// - Auto-attacks run continuously
/// - Manual skills: Bash (3s CD), Provoke (8s CD), Potion (5s CD)
/// - Stagger system with critical windows (2x damage)
/// - Progressive phases based on boss HP
/// - Performance ranking system (S, A, B, C, D)

use crate::combat::{Enemy, MvpBattlePhase, MvpBattleRank, MvpBattleState};
use crate::core::GameState;
use crate::quest::QuestAction;

impl GameState {
    /// Start MVP battle against a boss
    pub fn start_mvp_battle(&mut self, mut enemy: Enemy) {
        if self.hero.use_sp(10) {
            // MVP battles cost 10 SP
            let enemy_name = enemy.name;

            // MVP bosses have 5x HP to make battles longer
            enemy.hp = enemy.hp.saturating_mul(5);
            enemy.max_hp = enemy.max_hp.saturating_mul(5);

            self.mvp_battle_enemy = Some(enemy);
            self.mvp_battle_state = MvpBattleState::Start;
            self.mvp_battle_phase = MvpBattlePhase::Phase1;

            // Initialize skill cooldowns (all ready at start)
            self.mvp_skill_cooldowns[0].last_used_ms = self.last_update_ms.saturating_sub(3_000);
            self.mvp_skill_cooldowns[1].last_used_ms = self.last_update_ms.saturating_sub(8_000);
            self.mvp_skill_cooldowns[2].last_used_ms = self.last_update_ms.saturating_sub(5_000);

            // Reset stagger system
            self.mvp_stagger_value = 0;
            self.mvp_critical_window_active = false;
            self.mvp_critical_window_end = 0;

            // Reset auto-attack timer
            self.mvp_auto_attack_next = self.last_update_ms + self.mvp_auto_attack_interval;

            // Reset battle stats
            self.mvp_battle_elapsed = 0;
            self.mvp_perfect_hits = 0;
            self.mvp_boss_def_debuff = 0;
            self.mvp_boss_def_debuff_end = 0;
            self.mvp_battle_rank = None;

            // Start battle and switch to MVP Battle page
            self.mvp_battle_state = MvpBattleState::Playing;
            self.current_page = crate::core::GamePage::MvpBattle;
            self.needs_redraw = true;

            esp_println::println!("[MVP] Battle started against {}", enemy_name);
        }
    }

    /// Update MVP battle state
    pub fn update_mvp_battle(&mut self, delta_ms: u32) {
        if self.mvp_battle_state != MvpBattleState::Playing {
            return;
        }

        self.mvp_battle_elapsed += delta_ms;

        // Check if enemy is defeated
        if let Some(enemy) = &self.mvp_battle_enemy {
            if enemy.hp == 0 {
                self.complete_mvp_battle(true);
                return;
            }

            // Update battle phase based on boss HP
            let new_phase = MvpBattlePhase::from_hp_percent(enemy.hp_percent());
            if new_phase != self.mvp_battle_phase {
                self.mvp_battle_phase = new_phase;
                esp_println::println!("[MVP] Phase transition: {:?}", new_phase);
                self.needs_redraw = true;
            }
        }

        // Check if hero is defeated
        if self.hero.hp == 0 {
            self.complete_mvp_battle(false);
            return;
        }

        // Auto-attack system
        if self.last_update_ms >= self.mvp_auto_attack_next {
            self.execute_mvp_auto_attack();
            // Adjust auto-attack speed based on phase
            let phase_speed_mult = match self.mvp_battle_phase {
                MvpBattlePhase::Phase1 => 1.0,
                MvpBattlePhase::Phase2 => 0.9,  // 10% faster
                MvpBattlePhase::Phase3 => 0.8,  // 20% faster
            };
            let interval = (self.mvp_auto_attack_interval as f32 * phase_speed_mult) as u32;
            self.mvp_auto_attack_next = self.last_update_ms + interval;
        }

        // Boss counter-attacks based on phase
        self.mvp_boss_counter_attack(delta_ms);

        // Update critical window
        if self.mvp_critical_window_active {
            if self.last_update_ms >= self.mvp_critical_window_end {
                self.mvp_critical_window_active = false;
                esp_println::println!("[MVP] Critical window ended");
                self.needs_redraw = true;
            }
        }

        // Update DEF debuff
        if self.mvp_boss_def_debuff > 0 && self.last_update_ms >= self.mvp_boss_def_debuff_end {
            self.mvp_boss_def_debuff = 0;
            esp_println::println!("[MVP] DEF debuff expired");
        }
    }

    /// Execute auto-attack
    fn execute_mvp_auto_attack(&mut self) {
        if let Some(enemy) = &mut self.mvp_battle_enemy {
            // Calculate hero attack (simplified from JRPG)
            let weapon = &self.hero.equipped_weapon;
            let accessory1 = &self.hero.equipped_accessory1;
            let accessory2 = &self.hero.equipped_accessory2;

            let total_str = self.hero.base_str as i16
                + weapon.str_bonus + accessory1.str_bonus + accessory2.str_bonus;
            let weapon_atk = weapon.total_atk();
            let accessory_atk = accessory1.atk_bonus + accessory2.atk_bonus;
            let hero_atk = 10 + (total_str.max(0) as u16 * 2) + weapon_atk + accessory_atk;

            // Base damage calculation (20% of ATK for auto-attacks - reduced for longer battles)
            let base_damage = (hero_atk as f32 * 0.2) as u16;

            // Apply DEF debuff if active
            let def_mult = if self.mvp_boss_def_debuff > 0 {
                1.0 + (self.mvp_boss_def_debuff as f32 / 100.0)
            } else {
                1.0
            };

            let damage = (base_damage as f32 * def_mult) as u16;
            enemy.hp = enemy.hp.saturating_sub(damage);

            // Increase stagger value (reduced gain for less frequent critical windows)
            let stagger_gain = 3; // Each auto-attack adds 3 stagger (was 5)
            self.mvp_stagger_value = (self.mvp_stagger_value + stagger_gain).min(self.mvp_stagger_max);

            // Check if stagger is full -> trigger critical window
            if self.mvp_stagger_value >= self.mvp_stagger_max && !self.mvp_critical_window_active {
                self.mvp_critical_window_active = true;
                self.mvp_critical_window_end = self.last_update_ms + 2_000; // 2 second window
                self.mvp_stagger_value = 0; // Reset stagger
                esp_println::println!("[MVP] CRITICAL WINDOW ACTIVATED!");
                self.needs_redraw = true;
            }

            esp_println::println!(
                "[MVP] Auto-attack: {} damage (stagger: {}/{})",
                damage, self.mvp_stagger_value, self.mvp_stagger_max
            );
        }
    }

    /// Boss counter-attacks based on current phase
    fn mvp_boss_counter_attack(&mut self, _delta_ms: u32) {
        if let Some(enemy) = &self.mvp_battle_enemy {
            // Boss attacks more frequently in later phases (balanced)
            let attack_chance = match self.mvp_battle_phase {
                MvpBattlePhase::Phase1 => 0.005,  // 0.5% chance per frame (~1 attack every 2 seconds at 60fps)
                MvpBattlePhase::Phase2 => 0.008,  // 0.8% chance per frame (~1 attack every 1.25 seconds)
                MvpBattlePhase::Phase3 => 0.012,  // 1.2% chance per frame (~1 attack every 0.8 seconds)
            };

            // Simple RNG based on time (avoid division by zero)
            let rng = (self.last_update_ms % 1000) as f32 / 1000.0;

            if rng < attack_chance {
                // Boss attacks hero (balanced damage: 60% of attack)
                let boss_damage = (enemy.attack as f32 * 0.6).max(1.0) as u16;
                self.hero.hp = self.hero.hp.saturating_sub(boss_damage);
                esp_println::println!("[MVP] Boss attacks for {} damage! Hero HP: {}", boss_damage, self.hero.hp);
                self.needs_redraw = true;
            }
        }
    }

    /// Use skill - Bash (quick tap for burst damage)
    pub fn mvp_use_bash(&mut self) {
        // Check cooldown
        if !self.mvp_skill_cooldowns[0].is_ready(self.last_update_ms) {
            return;
        }

        if let Some(enemy) = &mut self.mvp_battle_enemy {
            // Calculate hero attack
            let weapon = &self.hero.equipped_weapon;
            let accessory1 = &self.hero.equipped_accessory1;
            let accessory2 = &self.hero.equipped_accessory2;

            let total_str = self.hero.base_str as i16
                + weapon.str_bonus + accessory1.str_bonus + accessory2.str_bonus;
            let weapon_atk = weapon.total_atk();
            let accessory_atk = accessory1.atk_bonus + accessory2.atk_bonus;
            let hero_atk = 10 + (total_str.max(0) as u16 * 2) + weapon_atk + accessory_atk;

            // Bash does 120% ATK damage (reduced for balance)
            let base_damage = (hero_atk as f32 * 1.2) as u16;

            // Apply DEF debuff if active
            let def_mult = if self.mvp_boss_def_debuff > 0 {
                1.0 + (self.mvp_boss_def_debuff as f32 / 100.0)
            } else {
                1.0
            };

            // Apply critical window multiplier
            let crit_mult = if self.mvp_critical_window_active {
                self.mvp_perfect_hits += 1; // Track perfect hits
                2.0
            } else {
                1.0
            };

            let damage = (base_damage as f32 * def_mult * crit_mult) as u16;
            enemy.hp = enemy.hp.saturating_sub(damage);

            // Update cooldown
            self.mvp_skill_cooldowns[0].last_used_ms = self.last_update_ms;

            esp_println::println!(
                "[MVP] BASH! {} damage {}",
                damage,
                if crit_mult > 1.0 { "(CRITICAL!)" } else { "" }
            );
            self.needs_redraw = true;
        }
    }

    /// Use skill - Provoke (long press to reduce boss DEF)
    pub fn mvp_use_provoke(&mut self) {
        // Check cooldown
        if !self.mvp_skill_cooldowns[1].is_ready(self.last_update_ms) {
            return;
        }

        // Apply DEF debuff (30% for 10 seconds)
        self.mvp_boss_def_debuff = 30;
        self.mvp_boss_def_debuff_end = self.last_update_ms + 10_000;

        // Update cooldown
        self.mvp_skill_cooldowns[1].last_used_ms = self.last_update_ms;

        esp_println::println!("[MVP] PROVOKE! Boss DEF reduced by 30% for 10s");
        self.needs_redraw = true;
    }

    /// Use skill - Potion (swipe up for emergency heal)
    pub fn mvp_use_potion(&mut self) {
        // Check cooldown
        if !self.mvp_skill_cooldowns[2].is_ready(self.last_update_ms) {
            return;
        }

        // Heal 40% of max HP (balanced)
        let heal_amount = (self.hero.max_hp as f32 * 0.4) as u16;
        self.hero.hp = (self.hero.hp + heal_amount).min(self.hero.max_hp);

        // Update cooldown
        self.mvp_skill_cooldowns[2].last_used_ms = self.last_update_ms;

        esp_println::println!("[MVP] POTION! Healed {} HP", heal_amount);
        self.needs_redraw = true;
    }

    /// Complete MVP battle
    fn complete_mvp_battle(&mut self, victory: bool) {
        if let Some(enemy) = &self.mvp_battle_enemy {
            let enemy_id = enemy.id;

            if victory {
                self.mvp_battle_state = MvpBattleState::Victory;

                // Calculate rank
                let hero_hp_percent = ((self.hero.hp as u32 * 100) / self.hero.max_hp as u32) as u8;
                let rank = MvpBattleRank::calculate(
                    self.mvp_battle_elapsed,
                    self.mvp_perfect_hits,
                    hero_hp_percent
                );
                self.mvp_battle_rank = Some(rank);

                esp_println::println!(
                    "[MVP] Victory! Rank: {} | Time: {}s | Perfect hits: {} | HP: {}%",
                    rank.display_name(),
                    self.mvp_battle_elapsed / 1000,
                    self.mvp_perfect_hits,
                    hero_hp_percent
                );

                // Calculate rewards with rank multiplier
                let rank_mult = rank.reward_multiplier();
                let base_exp = enemy.base_exp as f32;
                let card_bonuses = self.hero.get_total_card_bonuses();
                let exp_with_bonus = (base_exp * rank_mult * (100.0 + card_bonuses.exp_bonus as f32) / 100.0) as u32;
                let zeny_earned = (enemy.zeny_reward as f32 * rank_mult) as u32;

                // Store rewards for display
                self.last_battle_exp = exp_with_bonus;
                self.last_battle_zeny = zeny_earned;

                // Award rewards
                self.hero.add_exp(exp_with_bonus);
                self.hero.add_zeny(zeny_earned);

                // Update quest progress
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::MonsterKilled { enemy_id },
                );
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::BattleCompleted,
                );
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::ZenyEarned { amount: zeny_earned },
                );

                // Roll for drops with rank bonus
                let rng_value = (self.last_update_ms % 255) as u8;
                let drops = crate::data::roll_drops(enemy_id, rng_value);

                self.last_drops.clear();
                for (item_id, item_name, quantity) in drops.iter() {
                    // Increase drop quantity based on rank
                    let bonus_quantity = match rank {
                        MvpBattleRank::S => 2,
                        MvpBattleRank::A => 1,
                        _ => 0,
                    };
                    let total_quantity = quantity + bonus_quantity;

                    if self.hero.add_item(*item_id, item_name, total_quantity) {
                        esp_println::println!("[MVP] Got {} x{}", item_name, total_quantity);
                        self.last_drops.push((*item_id, item_name, total_quantity)).ok();
                    }
                }
            } else {
                self.mvp_battle_state = MvpBattleState::Defeat;
                self.last_drops.clear();
                self.last_battle_exp = 0;
                self.last_battle_zeny = 0;
                esp_println::println!("[MVP] Defeat!");
            }
        }

        self.needs_redraw = true;
    }

    /// Reset MVP battle state
    pub fn reset_mvp_battle(&mut self) {
        self.mvp_battle_enemy = None;
        self.mvp_battle_state = MvpBattleState::Idle;
        self.mvp_battle_phase = MvpBattlePhase::Phase1;
        self.mvp_stagger_value = 0;
        self.mvp_critical_window_active = false;
        self.mvp_critical_window_end = 0;
        self.mvp_auto_attack_next = 0;
        self.mvp_battle_elapsed = 0;
        self.mvp_perfect_hits = 0;
        self.mvp_boss_def_debuff = 0;
        self.mvp_boss_def_debuff_end = 0;
        self.mvp_battle_rank = None;
    }
}
