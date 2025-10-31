use crate::combat::{BattleState, Circle, CircleType, Enemy, MonsterAnimation};
/// Manual battle (Whac-A-Mole) game logic
///
/// Extension methods for GameState related to manual click-based battles.
use crate::core::{GamePage, GameState};
use crate::quest::QuestAction;

impl GameState {
    pub fn start_battle(&mut self, enemy: Enemy) {
        if self.hero.use_sp(5) {
            // Battle costs 5 SP (more than farming)
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

                *slot = Some(Circle::new(
                    x,
                    y,
                    30,
                    circle_type,
                    self.last_update_ms,
                    2000,
                ));
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
                // Award rewards based on score with card EXP bonus and level penalty
                let exp_mult = (self.battle_score as u32).max(1);

                // Apply level difference penalty (manual battles get full rewards, not 1/10)
                let level_penalty =
                    crate::combat::calculate_level_penalty(self.hero.level, enemy.level);

                // Calculate EXP with score multiplier and level penalty
                let base_exp = (enemy.base_exp as f32) * (exp_mult as f32) / 5.0 * level_penalty;
                let card_bonuses = self.hero.get_total_card_bonuses();
                let exp_with_bonus =
                    (base_exp * (100.0 + card_bonuses.exp_bonus as f32) / 100.0 + 0.5) as u32;

                // Calculate Zeny with score multiplier (no level penalty for zeny)
                let zeny_earned = enemy.zeny_reward * exp_mult / 5;

                // Store rewards for display
                self.last_battle_exp = exp_with_bonus;
                self.last_battle_zeny = zeny_earned;

                // Award rewards
                self.hero.add_exp(exp_with_bonus);
                self.hero.add_zeny(zeny_earned);

                // Roll for item drops
                let rng_value = (self.last_update_ms % 255) as u8;
                let drops = crate::data::roll_drops(enemy.id, rng_value);

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
                // Clear drops and rewards on defeat
                self.last_drops.clear();
                self.last_battle_exp = 0;
                self.last_battle_zeny = 0;
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
        self.last_battle_exp = 0;
        self.last_battle_zeny = 0;

        // Reset animation to Idle
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }
}
