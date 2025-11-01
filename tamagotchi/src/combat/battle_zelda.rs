/// Zelda-style timing battle game logic
///
/// Extension methods for GameState related to timing-based action battles.
/// Enemies spawn from the right and walk towards the hero in the center.
/// Player must tap at the right time when enemies enter the hit zone.

use crate::combat::{Enemy, HeroAnimation, MonsterAnimation, ZeldaBattleState, ZeldaEnemy};
use crate::core::{GamePage, GameState};
use crate::quest::QuestAction;

impl GameState {
    /// Start a Zelda-style battle
    pub fn start_zelda_battle(&mut self, enemy: Enemy) {
        if self.hero.use_sp(5) {
            // Battle costs 5 SP
            self.zelda_battle_enemy = Some(enemy);
            self.zelda_battle_state = ZeldaBattleState::Playing;
            self.zelda_battle_enemies.fill(None);
            self.zelda_battle_score = 0;
            self.zelda_battle_missed = 0;
            self.zelda_battle_combo = 0;
            self.zelda_battle_elapsed = 0;
            self.zelda_battle_next_spawn = self.last_update_ms + 1000; // First spawn in 1 second
            self.current_page = GamePage::ZeldaBattle; // Switch to Zelda battle page

            // Reset animations
            self.monster_animation = MonsterAnimation::Idle;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;
            self.hero_animation = HeroAnimation::Idle;
            self.hero_animation_frame = 0;
            self.hero_animation_started_ms = self.gif_animation_clock_ms;

            self.needs_redraw = true;
        }
    }

    /// Spawn a new enemy from the right side
    pub fn spawn_zelda_enemy(&mut self, rng_value: u8) {
        // Find empty slot
        for slot in &mut self.zelda_battle_enemies {
            if slot.is_none() {
                // Spawn from right edge of screen (x = 536)
                // Random Y position in middle area (avoid top/bottom edges)
                let y = 180 + ((rng_value as i32 * 7) % 120); // Y between 180-300

                // Get enemy HP from battle_enemy
                let enemy_hp = if let Some(ref enemy) = self.zelda_battle_enemy {
                    enemy.hp
                } else {
                    100 // Default HP
                };

                // Speed varies slightly (80-120 pixels per second)
                let speed = 80 + ((rng_value as i32 * 3) % 40);

                *slot = Some(ZeldaEnemy::new(
                    536,  // Start at right edge
                    y,
                    enemy_hp / 5, // Each small enemy has fraction of total HP
                    speed,
                    self.last_update_ms,
                ));
                break;
            }
        }
    }

    /// Update Zelda battle state
    pub fn update_zelda_battle(&mut self, delta_ms: u32) {
        if self.zelda_battle_state != ZeldaBattleState::Playing {
            return;
        }

        self.zelda_battle_elapsed += delta_ms;

        // Check if main battle enemy is defeated
        if let Some(enemy) = &self.zelda_battle_enemy {
            if enemy.hp == 0 {
                self.complete_zelda_battle();
                return;
            }
        }

        // Check if battle time is up (60 seconds)
        if self.zelda_battle_elapsed >= self.zelda_battle_duration {
            self.complete_zelda_battle();
            return;
        }

        // Spawn new enemies periodically
        if self.last_update_ms >= self.zelda_battle_next_spawn {
            let rng = (self.last_update_ms % 255) as u8;
            self.spawn_zelda_enemy(rng);
            self.zelda_battle_next_spawn = self.last_update_ms + self.zelda_battle_spawn_interval;
            self.needs_redraw = true;
        }

        // Update all active enemies
        let hero_x = 184; // Center X position where hero stands
        let hit_zone_width = 50; // Width of the hit zone around hero

        for enemy_slot in &mut self.zelda_battle_enemies {
            if let Some(enemy) = enemy_slot {
                // Update position
                enemy.update_position(delta_ms);

                // Check if in hit zone
                enemy.check_hit_zone(hero_x, hit_zone_width);

                // Check if enemy reached hero (player failed to hit)
                if enemy.has_reached_hero() && !enemy.is_hit {
                    // Enemy reached hero - player takes damage
                    let damage = if let Some(ref battle_enemy) = self.zelda_battle_enemy {
                        10 + battle_enemy.level
                    } else {
                        10
                    };
                    self.hero.take_damage(damage as u16);
                    self.zelda_battle_missed += 1;
                    self.zelda_battle_combo = 0; // Break combo

                    // Check if hero died
                    if self.hero.hp == 0 {
                        self.zelda_battle_state = ZeldaBattleState::Defeat;
                        self.needs_redraw = true;
                        return;
                    }

                    // Remove this enemy
                    *enemy_slot = None;
                    self.needs_redraw = true;
                }
            }
        }
    }

    /// Handle touch input during Zelda battle
    pub fn handle_zelda_battle_touch(&mut self, _x: i32, _y: i32) {
        if self.zelda_battle_state != ZeldaBattleState::Playing {
            return;
        }

        // Check if any enemy is in the hit zone
        let mut hit_enemy = false;
        let mut missed = false;

        for enemy_slot in &mut self.zelda_battle_enemies {
            if let Some(enemy) = enemy_slot {
                if !enemy.is_hit {
                    if enemy.is_in_hit_zone {
                        // Perfect timing! Hit the enemy
                        // Base damage: 5 + hero level
                        let damage = 5 + self.hero.level;

                        if enemy.hp > damage {
                            enemy.hp -= damage;
                        } else {
                            enemy.hp = 0;
                            enemy.is_hit = true;

                            // Award score and combo
                            self.zelda_battle_score += 1;
                            self.zelda_battle_combo += 1;

                            // Apply damage to main enemy
                            if let Some(ref mut battle_enemy) = self.zelda_battle_enemy {
                                let final_damage = damage * (1 + self.zelda_battle_combo / 5);
                                if battle_enemy.hp > final_damage {
                                    battle_enemy.hp -= final_damage;
                                } else {
                                    battle_enemy.hp = 0;
                                }
                            }

                            // Trigger hero attack animation
                            self.hero_animation = HeroAnimation::Attacking;
                            self.hero_animation_frame = 0;
                            self.hero_animation_started_ms = self.gif_animation_clock_ms;

                            // Mark hit for removal
                            *enemy_slot = None;
                        }

                        hit_enemy = true;
                        break;
                    }
                }
            }
        }

        // If tapped but no enemy in hit zone, it's a miss
        if !hit_enemy {
            missed = true;
            self.zelda_battle_missed += 1;
            self.zelda_battle_combo = 0; // Break combo
        }

        if hit_enemy || missed {
            self.needs_redraw = true;
        }
    }

    /// Complete the Zelda battle and award rewards
    pub fn complete_zelda_battle(&mut self) {
        if let Some(enemy) = &self.zelda_battle_enemy {
            let victory = enemy.hp == 0;

            if victory {
                self.zelda_battle_state = ZeldaBattleState::Victory;

                // Calculate rewards (score affects rewards)
                let base_exp = enemy.base_exp;
                let base_zeny = (enemy.level as u32) * 10;

                // Bonus based on combo
                let combo_multiplier = 1.0 + (self.zelda_battle_combo as f32 * 0.1);
                let exp = (base_exp as f32 * combo_multiplier) as u32;
                let zeny = (base_zeny as f32 * combo_multiplier) as u32;

                self.last_battle_exp = exp;
                self.last_battle_zeny = zeny;

                self.hero.add_exp(exp);
                self.hero.add_zeny(zeny);

                // Quest progress - monster killed
                let enemy_id = enemy.id;
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::MonsterKilled { enemy_id },
                );

                // Quest progress - battle completed
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::BattleCompleted,
                );

                // Quest progress - zeny earned
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::ZenyEarned { amount: zeny },
                );

                // Item drops (simple for now)
                // TODO: Implement item drop system
            } else {
                self.zelda_battle_state = ZeldaBattleState::Defeat;
            }

            self.battle_end_time = self.last_update_ms;
        }

        self.needs_redraw = true;
    }

    /// Return to map from Zelda battle
    pub fn exit_zelda_battle(&mut self) {
        self.zelda_battle_state = ZeldaBattleState::Idle;
        self.zelda_battle_enemy = None;
        self.zelda_battle_enemies.fill(None);
        self.current_page = GamePage::Map;
        self.needs_redraw = true;
    }
}
