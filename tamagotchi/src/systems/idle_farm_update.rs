/// IDLE farming update system
///
/// Handles background farming updates including HP regen, damage, kills, and rewards.

use crate::core::GameState;
use crate::combat::{Enemy, IdleFarmState};

/// Update IDLE farming session with real-time combat simulation (called every frame)
pub fn update_idle_farm_session(game_state: &mut GameState, delta_ms: u32) {
    // Check if there's an active farming session
    let session = match &mut game_state.idle_farm_session {
        Some(session) => session,
        None => return, // No active session
    };

    let current_time = game_state.last_update_ms;

    match session.state {
        IdleFarmState::Active => {
            session.last_update_ms = current_time;

            // === HP REGENERATION (passive, happens every second) ===
            let regen_elapsed_ms = current_time.saturating_sub(session.last_hp_regen_ms);
            if regen_elapsed_ms >= 1000 {
                let seconds_elapsed = regen_elapsed_ms as f32 / 1000.0;
                let hp_regen = (session.hp_regen_rate * seconds_elapsed) as u16;
                session.current_hp = (session.current_hp + hp_regen).min(game_state.hero.max_hp);
                game_state.hero.hp = session.current_hp;
                session.last_hp_regen_ms = current_time;
            }

            // === ENEMY SPAWN HANDLING ===
            if session.enemy_spawning {
                if current_time >= session.enemy_spawn_complete_ms {
                    // Spawn new enemy
                    session.enemy_spawning = false;
                    session.current_enemy_hp = session.enemy_max_hp;
                    session.next_enemy_attack_ms = current_time + session.enemy_attack_delay_ms;
                    esp_println::println!("[COMBAT] New enemy spawned!");
                }
                return; // Skip combat while spawning
            }

            // === HERO ATTACK ===
            if current_time >= session.next_hero_attack_ms {
                if let Some(enemy) = Enemy::from_id(session.enemy_id) {
                    // Calculate hero stats
                    let hero_atk = game_state.hero.base_str * 2 + game_state.hero.equipped_weapon.atk_bonus;
                    let hero_agi = game_state.hero.base_agi;

                    // Check if attack misses (AGI vs enemy level)
                    // Miss rate = max(5%, 30% - AGI)
                    let miss_chance = ((30.0 - hero_agi as f32).max(5.0)) as u8;
                    let hit_roll = (current_time % 100) as u8;
                    let is_miss = hit_roll < miss_chance;

                    if !is_miss {
                        // Calculate damage
                        let base_damage = if hero_atk > enemy.defense {
                            hero_atk - enemy.defense
                        } else {
                            1
                        };

                        // Check if skill is available for use
                        let use_skill = current_time >= session.next_skill_use_ms;
                        let damage = if use_skill {
                            // Skill does 2x damage
                            let skill_damage = base_damage * 2;
                            session.last_skill_use_ms = current_time;
                            session.next_skill_use_ms = current_time + session.skill_cooldown_ms;
                            esp_println::println!("[COMBAT] Hero uses SKILL! Damage: {}", skill_damage);
                            skill_damage
                        } else {
                            esp_println::println!("[COMBAT] Hero attacks! Damage: {}", base_damage);
                            base_damage
                        };

                        // Apply damage to enemy
                        if session.current_enemy_hp > damage {
                            session.current_enemy_hp -= damage;
                        } else {
                            // Enemy killed!
                            session.current_enemy_hp = 0;
                            session.monsters_killed += 1;

                            // Award rewards
                            let zeny_gain = enemy.zeny_reward;
                            let exp_gain = enemy.base_exp;
                            session.zeny_earned += zeny_gain;
                            session.exp_gained += exp_gain;
                            game_state.hero.zeny += zeny_gain;
                            game_state.hero.add_exp(exp_gain);

                            // Roll for item drops
                            use crate::data::roll_drops;
                            let rng_value = ((current_time % 255) + session.monsters_killed as u32) as u8;
                            let drops = roll_drops(enemy.id, rng_value);
                            for (item_id, item_name, quantity) in drops {
                                use crate::hero::inventory::InventoryExt;
                                game_state.hero.inventory.add_item(item_id, item_name, quantity);
                                session.items_collected += quantity;
                            }

                            esp_println::println!("[COMBAT] Enemy killed! Total: {}", session.monsters_killed);

                            // Start enemy respawn (death animation + spawn delay = 2 seconds)
                            session.enemy_spawning = true;
                            session.enemy_spawn_complete_ms = current_time + 2000;
                        }
                    } else {
                        esp_println::println!("[COMBAT] Hero attack MISSED!");
                    }

                    // Schedule next hero attack
                    session.last_hero_attack_ms = current_time;
                    session.next_hero_attack_ms = current_time + session.hero_attack_delay_ms;
                }
            }

            // === ENEMY ATTACK ===
            if !session.enemy_spawning && current_time >= session.next_enemy_attack_ms {
                if let Some(enemy) = Enemy::from_id(session.enemy_id) {
                    // Calculate hero defense
                    let hero_def = (game_state.hero.base_vit / 2) +
                                   game_state.hero.equipped_armor.def_bonus +
                                   game_state.hero.equipped_garment.def_bonus +
                                   game_state.hero.equipped_shoes.def_bonus;

                    // Enemy miss chance (lower than hero, ~10-20%)
                    let enemy_miss_chance = 15u8;
                    let enemy_hit_roll = ((current_time + 50) % 100) as u8;
                    let is_miss = enemy_hit_roll < enemy_miss_chance;

                    if !is_miss {
                        // Calculate enemy damage
                        let damage = if enemy.attack > hero_def {
                            enemy.attack - hero_def
                        } else {
                            1
                        };

                        esp_println::println!("[COMBAT] Enemy attacks! Damage: {}", damage);

                        // Apply damage to hero
                        if session.current_hp > damage {
                            session.current_hp -= damage;
                            game_state.hero.hp = session.current_hp;
                        } else {
                            // Hero died!
                            session.current_hp = 0;
                            game_state.hero.hp = 0;
                            session.state = IdleFarmState::Cooldown;
                            session.cooldown_end_ms = current_time + 60_000; // 60 second cooldown

                            esp_println::println!("[COMBAT] Hero DIED!");

                            // Show results screen
                            use crate::core::GamePage;
                            game_state.current_page = GamePage::IdleFarmResult;
                            game_state.needs_redraw = true;
                            return;
                        }
                    } else {
                        esp_println::println!("[COMBAT] Enemy attack MISSED!");
                    }

                    // Schedule next enemy attack
                    session.last_enemy_attack_ms = current_time;
                    session.next_enemy_attack_ms = current_time + session.enemy_attack_delay_ms;
                }
            }

            // Trigger periodic redraw
            if current_time % 1000 < delta_ms {
                game_state.needs_redraw = true;
            }
        }
        IdleFarmState::Cooldown => {
            // Check if cooldown is over
            if current_time >= session.cooldown_end_ms {
                esp_println::println!("[IDLE FARM] Cooldown complete - session ended");
            }
        }
        IdleFarmState::Idle => {
            // Session is idle, nothing to update
        }
    }
}

/// Start a new IDLE farming session
pub fn start_idle_farm_session(
    game_state: &mut GameState,
    map_id: u32,
    enemy_id: u32,
) {
    // Calculate farming rates
    if let Some(enemy) = Enemy::from_id(enemy_id) {
        use crate::combat::calculate_farming_rates;

        let rates = calculate_farming_rates(&game_state.hero, &enemy);

        esp_println::println!(
            "[IDLE FARM] Starting session - Kills/min: {:.1}, Zeny/min: {:.1}, Damage/min: {:.1}, Regen/min: {:.1}",
            rates.kills_per_minute,
            rates.zeny_per_minute,
            rates.damage_per_minute,
            rates.regen_per_minute
        );

        // Check if farming is even possible (net HP must be >= 0 for reasonable duration)
        if rates.net_hp_per_minute < -10.0 {
            // Hero will die very quickly
            esp_println::println!("[IDLE FARM] Warning: Hero will die quickly (net HP: {:.1}/min)", rates.net_hp_per_minute);
        }

        use crate::combat::IdleFarmSession;

        let session = IdleFarmSession::new(
            map_id,
            enemy_id,
            game_state.last_update_ms,
            game_state.hero.hp,
            rates.kills_per_minute,
            rates.zeny_per_minute,
            rates.damage_per_minute,
            rates.regen_per_minute,
            game_state.hero.base_agi,  // Total AGI
            game_state.hero.base_vit,  // Total VIT
            enemy.max_hp,
            enemy.level,
        );

        game_state.idle_farm_session = Some(session);
        game_state.needs_redraw = true;

        esp_println::println!("[IDLE FARM] Session started successfully");
    } else {
        esp_println::println!("[IDLE FARM] ERROR: Enemy not found with ID {}", enemy_id);
    }
}

/// Stop the current IDLE farming session
pub fn stop_idle_farm_session(game_state: &mut GameState) {
    if let Some(ref session) = game_state.idle_farm_session {
        esp_println::println!(
            "[IDLE FARM] Session stopped - Killed: {}, Zeny: {}, Exp: {}",
            session.monsters_killed,
            session.zeny_earned,
            session.exp_gained
        );

        // Show results screen if there's something to show
        if session.monsters_killed > 0 || session.duration_ms(game_state.last_update_ms) > 5000 {
            use crate::core::GamePage;
            game_state.current_page = GamePage::IdleFarmResult;
        }
    }

    game_state.needs_redraw = true;
}
