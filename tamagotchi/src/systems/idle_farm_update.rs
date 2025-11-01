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

            // === ENEMY DEATH ANIMATION ===
            if session.enemy_dying {
                if current_time >= session.enemy_death_complete_ms {
                    // Death animation complete, select new enemy and start spawning phase
                    session.enemy_dying = false;

                    // Randomly select new enemy from pool BEFORE starting walk-in animation
                    if !session.enemy_pool.is_empty() {
                        // Use current time as RNG seed to pick random enemy
                        let enemy_index = (current_time % session.enemy_pool.len() as u32) as usize;
                        let new_enemy_id = session.enemy_pool[enemy_index];

                        if let Some(new_enemy) = Enemy::from_id(new_enemy_id) {
                            // Update session with new enemy data
                            session.enemy_id = new_enemy_id;
                            session.enemy_max_hp = new_enemy.max_hp;
                            session.current_enemy_hp = new_enemy.max_hp;
                            session.enemy_level = new_enemy.level;

                            // Recalculate enemy attack delay for new enemy
                            session.enemy_attack_delay_ms = (5000 - (new_enemy.level as u32 * 30)).max(3000).min(5000);

                            esp_println::println!("[COMBAT] New enemy selected: {} (Level {}) - walking in...", new_enemy.name, new_enemy.level);
                        } else {
                            // Fallback: use existing enemy_id if new one is invalid
                            session.current_enemy_hp = session.enemy_max_hp;
                            esp_println::println!("[COMBAT] Enemy respawning - walking in...");
                        }
                    } else {
                        // Fallback: respawn same enemy if pool is empty
                        session.current_enemy_hp = session.enemy_max_hp;
                        esp_println::println!("[COMBAT] Enemy respawning - walking in...");
                    }

                    session.enemy_spawning = true;
                    session.enemy_spawn_complete_ms = current_time + 2000; // 2 second spawn delay
                    session.enemy_spawn_position_x = -64; // Start off-screen left for walk-in animation
                }
                return; // Skip combat while dying
            }

            // === ENEMY SPAWN HANDLING ===
            if session.enemy_spawning {
                // Animate enemy walking in from left side
                // Target position: 90, Start position: -64, Distance: 154 pixels over 2000ms
                const TARGET_X: i32 = 90;
                const SPAWN_DURATION_MS: u32 = 2000;

                let spawn_start_ms = session.enemy_spawn_complete_ms - SPAWN_DURATION_MS;
                let spawn_elapsed_ms = current_time.saturating_sub(spawn_start_ms);
                let spawn_progress = (spawn_elapsed_ms as f32 / SPAWN_DURATION_MS as f32).min(1.0);

                // Smooth walk-in animation
                session.enemy_spawn_position_x = -64 + ((TARGET_X - (-64)) as f32 * spawn_progress) as i32;

                // Trigger redraw for animation
                game_state.needs_redraw = true;

                if current_time >= session.enemy_spawn_complete_ms {
                    // Walk-in animation complete, enemy has reached battle position
                    session.enemy_spawning = false;
                    session.enemy_spawn_position_x = TARGET_X; // Ensure final position is exact

                    esp_println::println!("[COMBAT] Enemy reached battle position - combat starting!");

                    // Add 1-second delay before first attacks after spawn
                    // This creates a visible idle moment before combat resumes
                    session.next_hero_attack_ms = current_time + 1000;
                    session.next_enemy_attack_ms = current_time + session.enemy_attack_delay_ms + 1000;
                }
                return; // Skip combat while spawning
            }

            // === HERO ATTACK INITIATION ===
            // Start attack animation and calculate damage, but don't apply yet
            if current_time >= session.next_hero_attack_ms && !session.hero_attack_pending {
                if let Some(enemy) = Enemy::from_id(session.enemy_id) {
                    // Calculate hero stats
                    let hero_atk = game_state.hero.base_str * 2 + game_state.hero.equipped_weapon.atk_bonus;

                    // Check if attack misses (DEX + Level vs Enemy Level)
                    // Hit rate formula: 80% + (Hero DEX / 5) + (Hero Level - Enemy Level)
                    // Base hit rate: 80%
                    // DEX bonus: +1% hit per 5 DEX (so 50 DEX = +10% hit)
                    // Level difference: +1% hit per level above enemy, -1% per level below
                    // Final hit rate clamped between 20% and 95%
                    let base_hit_rate = 80.0;
                    let dex_bonus = session.hero_dex as f32 / 5.0;
                    let level_diff = session.hero_level as i32 - session.enemy_level as i32;
                    let hit_rate = (base_hit_rate + dex_bonus + level_diff as f32).max(20.0).min(95.0);
                    let miss_chance = 100.0 - hit_rate;

                    let hit_roll = (current_time % 100) as f32;
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
                            session.last_skill_use_ms = current_time;
                            session.next_skill_use_ms = current_time + session.skill_cooldown_ms;
                            session.last_skill_used = true;
                            esp_println::println!("[COMBAT] Hero starts SKILL attack!");
                            base_damage * 2
                        } else {
                            session.last_skill_used = false;
                            esp_println::println!("[COMBAT] Hero starts attack!");
                            base_damage
                        };

                        // Store damage to apply later (after animation plays)
                        session.pending_hero_damage = damage;
                        session.pending_hero_miss = false;
                    } else {
                        esp_println::println!("[COMBAT] Hero attack will MISS!");
                        session.pending_hero_damage = 0;
                        session.pending_hero_miss = true;
                        session.last_skill_used = false;
                    }

                    // Start attack animation, damage applies after 600ms windup
                    session.hero_attack_pending = true;
                    session.hero_damage_apply_ms = current_time + 600;
                    session.last_hero_attack_ms = current_time;
                    session.next_hero_attack_ms = current_time + session.hero_attack_delay_ms;
                }
            }

            // === HERO DAMAGE APPLICATION ===
            // Apply damage after attack animation has played
            if session.hero_attack_pending && current_time >= session.hero_damage_apply_ms {
                if let Some(enemy) = Enemy::from_id(session.enemy_id) {
                    session.hero_attack_pending = false;

                    if !session.pending_hero_miss {
                        let damage = session.pending_hero_damage;

                        // Update display tracking
                        session.last_hero_damage = damage;
                        session.hero_attack_missed = false;

                        esp_println::println!("[COMBAT] Hero attack lands! Damage: {}", damage);

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

                            // Start death animation phase
                            session.enemy_dying = true;
                            session.enemy_death_complete_ms = current_time + 1500;
                        }
                    } else {
                        // Attack missed
                        session.hero_attack_missed = true;
                        session.last_hero_damage = 0;
                        esp_println::println!("[COMBAT] Hero attack MISSED!");
                    }
                }
            }

            // === ENEMY ATTACK INITIATION ===
            // Start attack animation and calculate damage, but don't apply yet
            if !session.enemy_spawning && !session.enemy_dying
               && current_time >= session.next_enemy_attack_ms && !session.enemy_attack_pending {
                if let Some(enemy) = Enemy::from_id(session.enemy_id) {
                    // Calculate hero defense
                    let hero_def = (game_state.hero.base_vit / 2) +
                                   game_state.hero.equipped_armor.def_bonus +
                                   game_state.hero.equipped_garment.def_bonus +
                                   game_state.hero.equipped_shoes.def_bonus;

                    // Enemy miss chance
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

                        esp_println::println!("[COMBAT] Enemy starts attack!");
                        session.pending_enemy_damage = damage;
                        session.pending_enemy_miss = false;
                    } else {
                        esp_println::println!("[COMBAT] Enemy attack will MISS!");
                        session.pending_enemy_damage = 0;
                        session.pending_enemy_miss = true;
                    }

                    // Start attack animation, damage applies after 600ms windup
                    session.enemy_attack_pending = true;
                    session.enemy_damage_apply_ms = current_time + 600;
                    session.last_enemy_attack_ms = current_time;
                    session.next_enemy_attack_ms = current_time + session.enemy_attack_delay_ms;
                }
            }

            // === ENEMY DAMAGE APPLICATION ===
            // Apply damage after attack animation has played
            if session.enemy_attack_pending && current_time >= session.enemy_damage_apply_ms {
                if let Some(_enemy) = Enemy::from_id(session.enemy_id) {
                    session.enemy_attack_pending = false;

                    if !session.pending_enemy_miss {
                        let damage = session.pending_enemy_damage;

                        // Track damage for display
                        session.last_enemy_damage = damage;
                        session.enemy_attack_missed = false;

                        esp_println::println!("[COMBAT] Enemy attack lands! Damage: {}", damage);

                        // Apply damage to hero
                        if session.current_hp > damage {
                            session.current_hp -= damage;
                            game_state.hero.hp = session.current_hp;
                        } else {
                            // Hero died!
                            session.current_hp = 0;
                            game_state.hero.hp = 0;
                            session.state = IdleFarmState::Cooldown;
                            session.cooldown_end_ms = current_time + 60_000;

                            esp_println::println!("[COMBAT] Hero DIED!");

                            // Show results screen
                            use crate::core::GamePage;
                            game_state.current_page = GamePage::IdleFarmResult;
                            game_state.needs_redraw = true;
                            return;
                        }
                    } else {
                        // Attack missed
                        session.enemy_attack_missed = true;
                        session.last_enemy_damage = 0;
                        esp_println::println!("[COMBAT] Enemy attack MISSED!");
                    }
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
    // Get enemy pool from map
    use crate::world::MapHelper;
    let enemy_pool = MapHelper::enemies(map_id);

    if enemy_pool.is_empty() {
        esp_println::println!("[IDLE FARM] ERROR: No enemies found for map {}", map_id);
        return;
    }

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

        // Calculate total stats including equipment bonuses
        let total_vit = (game_state.hero.base_vit as i32
            + game_state.hero.equipped_armor.vit_bonus as i32
            + game_state.hero.equipped_garment.vit_bonus as i32
            + game_state.hero.equipped_shoes.vit_bonus as i32)
            .max(1) as u16;

        let total_dex = (game_state.hero.base_dex as i32
            + game_state.hero.equipped_weapon.dex_bonus as i32
            + game_state.hero.equipped_accessory1.dex_bonus as i32
            + game_state.hero.equipped_accessory2.dex_bonus as i32)
            .max(1) as u16;

        let session = IdleFarmSession::new(
            map_id,
            enemy_id,
            enemy_pool,
            game_state.last_update_ms,
            game_state.hero.hp,
            rates.kills_per_minute,
            rates.zeny_per_minute,
            rates.damage_per_minute,
            rates.regen_per_minute,
            game_state.hero.level,
            game_state.hero.base_agi,
            total_vit,  // VIT with equipment bonuses
            total_dex,  // DEX with equipment bonuses
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
