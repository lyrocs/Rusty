//! Death System
//!
//! Handles hero death detection, death screen, and respawn

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to detect hero death in battle and switch to death screen
pub fn death_detection_system(
    mut app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Check in Battle or Battle3v3 mode
    let is_battle = app_state.current_mode == AppMode::Battle;
    let is_battle_3v3 = app_state.current_mode == AppMode::Battle3v3;

    if !is_battle && !is_battle_3v3 {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check for death or victory in 1v1 battle
    if is_battle {
        if let Some(ref battle_page) = game_manager.battle_page {
            if battle_page.hero_died() {
                log::info!("💀 Rustymon fainted! Switching to death screen...");

                // Sync battle state before switching
                game_manager.sync_battle_state();

                // Only create death page if it doesn't already exist (to preserve timer)
                if game_manager.death_page.is_none() {
                    // Create death page
                    match crate::ui::pages::DeathPage::new() {
                        Ok(death_page) => {
                            log::info!("Created new death page with 2-minute timer");
                            game_manager.death_page = Some(death_page);
                        }
                        Err(e) => {
                            log::error!("Failed to create death page: {:?}", e);
                            // Fallback: just reset Rustymon HP and continue
                            for rustymon in &mut game_manager.rustymon_collection {
                                rustymon.current_hp = rustymon.max_hp / 2;
                            }
                        }
                    }
                } else {
                    log::info!("Death page already exists, keeping existing timer");
                }

                // Switch to death mode
                app_state.current_mode = AppMode::Death;
                app_state.needs_redraw = true;
            }
        }
    }

    // Check for death or victory in 3v3 battle
    if is_battle_3v3 {
        if let Some(ref mut battle_3v3_page) = game_manager.battle_3v3_page {
            use crate::ui::pages::battle_3v3::BattleResult;

            let result = battle_3v3_page.get_result();

            // Debug: Log battle status every check
            if matches!(result, BattleResult::Ongoing) {
                // Only log occasionally to avoid spam
                static mut LAST_LOG: Option<std::time::Instant> = None;
                unsafe {
                    let now = std::time::Instant::now();
                    if LAST_LOG.map(|t| now.duration_since(t).as_secs() >= 2).unwrap_or(true) {
                        log::info!("⚔️ 3v3 Battle ongoing...");
                        LAST_LOG = Some(now);
                    }
                }
            }

            match result {
                BattleResult::Defeat => {
                    log::info!("💀 All Rustymon fainted in 3v3! Switching to death screen...");

                    // Sync battle state (update rustymon HP)
                    game_manager.rustymon_collection = battle_3v3_page.get_rustymon_collection();

                    // Create death page
                    if game_manager.death_page.is_none() {
                        match crate::ui::pages::DeathPage::new() {
                            Ok(death_page) => {
                                log::info!("Created new death page with 2-minute timer");
                                game_manager.death_page = Some(death_page);
                            }
                            Err(e) => {
                                log::error!("Failed to create death page: {:?}", e);
                                for rustymon in &mut game_manager.rustymon_collection {
                                    rustymon.current_hp = rustymon.max_hp / 2;
                                }
                            }
                        }
                    }

                    // Switch to death mode
                    app_state.current_mode = AppMode::Death;
                    app_state.needs_redraw = true;
                }
                BattleResult::Victory => {
                    log::info!("🎉 Victory in 3v3 battle!");

                    // Get data from battle before mutating game_manager
                    let updated_collection = battle_3v3_page.get_rustymon_collection();
                    let fragment_drops = battle_3v3_page.get_fragment_drops();

                    // Sync battle state (update rustymon HP)
                    game_manager.rustymon_collection = updated_collection;

                    // Apply fragment drops
                    if !fragment_drops.is_empty() {
                        log::info!("✨ Applying {} fragment drops", fragment_drops.len());
                        for (enemy_id, enemy_name) in fragment_drops {
                            game_manager.fragment_collection.add_fragment(enemy_id, 1);
                            log::info!("💎 Fragment added to collection: {} (ID: {})", enemy_name, enemy_id);
                        }
                    }

                    // Get rustymon that participated in battle (first 3 from team with HP > 0)
                    let mut battle_rustymon = Vec::new();
                    for slot in game_manager.rustymon_team.active_slots.iter().take(3) {
                        if let Some(rustymon_id) = slot {
                            if let Some(rustymon) = game_manager.rustymon_collection.iter().find(|r| &r.id == rustymon_id) {
                                log::info!("📝 Rustymon for result: {} - Level {}, EXP: {}",
                                    rustymon.name, rustymon.level, rustymon.exp);
                                battle_rustymon.push(rustymon.clone());
                            }
                        }
                        if battle_rustymon.len() >= 3 {
                            break;
                        }
                    }

                    // Calculate exp rewards (50 exp per rustymon for now)
                    let exp_rewards = vec![50; battle_rustymon.len()];
                    log::info!("🎁 Giving {} EXP to {} rustymon", exp_rewards[0], exp_rewards.len());

                    // Create battle result page
                    match crate::ui::pages::BattleResultPage::new(
                        battle_rustymon,
                        exp_rewards,
                        game_manager.map_page.world_map().game_data().clone(),
                    ) {
                        Ok(result_page) => {
                            log::info!("Created battle result page");
                            game_manager.battle_result_page = Some(result_page);

                            // Clear battle page
                            game_manager.battle_3v3_page = None;

                            // Switch to result screen
                            app_state.current_mode = AppMode::BattleResult;
                            app_state.needs_redraw = true;
                        }
                        Err(e) => {
                            log::error!("Failed to create battle result page: {:?}", e);
                            // Fallback: just go to map
                            game_manager.battle_3v3_page = None;
                            app_state.current_mode = AppMode::Map;
                            app_state.needs_redraw = true;
                        }
                    }
                }
                BattleResult::Ongoing => {
                    // Battle still ongoing
                }
            }
        }
    }
}

/// System to handle death screen interactions
pub fn death_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Death mode
    if app_state.current_mode != AppMode::Death {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Check if we can respawn
                if let Some(ref death_page) = game_manager.death_page {
                    if death_page.handle_touch(x, y) {
                        log::info!("✨ Respawning Rustymon team!");

                        // Restore all Rustymon HP to full
                        for rustymon in &mut game_manager.rustymon_collection {
                            rustymon.current_hp = rustymon.max_hp;
                            log::debug!("Restored {} HP to full ({}/{})",
                                rustymon.name, rustymon.current_hp, rustymon.max_hp);
                        }
                        log::info!("All Rustymon HP restored to full");

                        // Clear battle and death pages
                        game_manager.battle_page = None;
                        game_manager.battle_3v3_page = None;
                        game_manager.death_page = None;

                        // Return to map
                        app_state.current_mode = AppMode::Map;
                        app_state.needs_redraw = true;
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
