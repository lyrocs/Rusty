//! Expedition Setup System
//!
//! Handles user interactions on the expedition setup page

use bevy_ecs::prelude::*;
use std::time::Instant;

use crate::ecs::resources::{AppMode, AppState, ExpeditionData, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::game::{calculate_expedition, calculate_drops, Card, ExpeditionSize};

/// System to handle expedition setup interactions
pub fn expedition_setup_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in ExpeditionSetup mode
    if app_state.current_mode != AppMode::ExpeditionSetup {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on expedition setup page
                if let Some(ref mut setup_page) = game_manager.expedition_setup_page {
                    // Check if user tapped a size button
                    if let Some(size) = setup_page.handle_touch(x, y) {
                        log::info!("Selected expedition size: {} monsters", size.count());
                        app_state.needs_redraw = true;
                    }

                    // Check if user tapped START (when size is selected)
                    if let Some(selected_size) = setup_page.get_selection() {
                        // Check if START button was tapped (100, 290, 168x45)
                        if x >= 100 && x <= 268 && y >= 290 && y <= 335 {
                            log::info!("🚀 Starting expedition with {} monsters!", selected_size.count());

                            // Get enemy from setup page (we'll need to store it)
                            // For now, we'll start the expedition directly
                            start_expedition(game_manager, selected_size, &mut app_state);
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe right to cancel and go back to map
                if *direction == SwipeDirection::Right {
                    log::info!("Cancelled expedition setup, returning to map");
                    game_manager.expedition_setup_page = None;
                    app_state.current_mode = AppMode::Map;
                    app_state.needs_redraw = true;
                }
            }
            _ => {}
        }
    }
}

/// Start an expedition with the selected size
fn start_expedition(
    game_manager: &mut GameManager,
    size: ExpeditionSize,
    app_state: &mut AppState,
) {
    // Get the setup page to extract enemy info
    let setup_page = match game_manager.expedition_setup_page.take() {
        Some(page) => page,
        None => {
            log::error!("No expedition setup page found!");
            return;
        }
    };

    // Extract enemy info from the page (we need to refactor ExpeditionSetupPage to expose this)
    // For now, we'll re-fetch from game_manager
    let map_id = game_manager.selected_map_id.unwrap_or(1);
    let location = game_manager.map_page.world_map().get_location(map_id);

    if let Some(location) = location {
        if !location.enemies.is_empty() {
            // Pick first enemy for simplicity (should be same as setup)
            let enemy_id = location.enemies[0];

            if let Some(enemy_data) = game_manager.game_data.get_enemy(enemy_id) {
                // Create enemy instance
                use crate::game::Enemy;
                let enemy = Enemy::from_data(
                    enemy_data.id,
                    enemy_data.name.clone(),
                    enemy_data.level,
                    enemy_data.hp,
                    enemy_data.attack,
                    enemy_data.defense,
                    enemy_data.hit,
                    enemy_data.flee,
                    enemy_data.base_exp,
                    enemy_data.get_element(),
                );

                // Calculate expedition results
                let result = calculate_expedition(&game_manager.hero, &enemy, size.count());

                log::info!("📊 Expedition calculated: {:.1}s duration, {} damage, survives: {}",
                    result.duration_seconds, result.total_damage as u32, result.survives);

                // Store expedition data
                let now = Instant::now();
                let duration = std::time::Duration::from_secs_f32(result.duration_seconds);
                let duration_seconds = result.duration_seconds; // Save before moving

                // Update hero state to OnExpedition
                let end_timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + duration.as_secs();
                let start_timestamp = end_timestamp - duration.as_secs();

                game_manager.expedition_data = Some(ExpeditionData {
                    enemy_id: enemy_data.id,
                    enemy_name: enemy_data.name.clone(),
                    target_kills: size.count(),
                    expedition_size: size,
                    start_time: now,
                    end_time: now + duration,
                    result,
                });

                game_manager.hero.state = crate::game::HeroState::OnExpedition {
                    end_time: end_timestamp,
                };

                // Create in-progress page
                match crate::ui::pages::ExpeditionInProgressPage::new(
                    enemy_data.name.clone(),
                    size.count(),
                    duration_seconds,
                    start_timestamp,
                    end_timestamp,
                ) {
                    Ok(progress_page) => {
                        game_manager.expedition_in_progress_page = Some(progress_page);
                        app_state.current_mode = AppMode::ExpeditionInProgress;
                        app_state.needs_redraw = true;
                        log::info!("Expedition started, showing progress page");
                    }
                    Err(e) => {
                        log::error!("Failed to create expedition progress page: {:?}", e);
                    }
                }
            }
        }
    }
}

/// Complete the expedition and show summary
fn complete_expedition(game_manager: &mut GameManager) {
    let expedition_data = match game_manager.expedition_data.take() {
        Some(data) => data,
        None => {
            log::error!("No expedition data found!");
            return;
        }
    };

    // Calculate results
    let survived = expedition_data.result.survives;
    let actual_kills = expedition_data.result.kills_completed;

    // Calculate EXP gained
    let exp_per_kill = expedition_data.result.kills_completed * 10; // Simple formula for now
    let total_exp = exp_per_kill * actual_kills;

    // Capture level BEFORE applying EXP
    let initial_level = game_manager.hero.level;

    // Apply EXP to hero
    let leveled_up = game_manager.hero.gain_experience(total_exp);
    if leveled_up {
        log::info!("🎉 LEVEL UP! Hero is now level {}", game_manager.hero.level);
    }

    // Calculate card drops if survived
    let cards_dropped = if survived {
        // Get enemy data for card info
        if let Some(enemy_data) = game_manager.game_data.get_enemy(expedition_data.enemy_id) {
            // Create card template
            let card_template = Card::new(
                enemy_data.id,
                enemy_data.card.name.clone(),
                enemy_data.card.rarity,
                enemy_data.card.atk_bonus,
                enemy_data.card.def_bonus,
            );

            // Calculate drops (uses Enemy instance but we'll fake it)
            let enemy = crate::game::Enemy::from_data(
                enemy_data.id,
                enemy_data.name.clone(),
                enemy_data.level,
                enemy_data.hp,
                enemy_data.attack,
                enemy_data.defense,
                enemy_data.hit,
                enemy_data.flee,
                enemy_data.base_exp,
                enemy_data.get_element(),
            );

            let drops = calculate_drops(
                &enemy,
                &card_template,
                enemy_data.drop_rate,
                actual_kills,
                expedition_data.expedition_size,
            );

            log::info!("💎 Cards dropped: {}", drops.len());
            drops
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Add cards to hero inventory
    for card in &cards_dropped {
        game_manager.hero.cards.push(card.clone());
    }

    // Apply damage to hero
    let damage_taken = if survived {
        expedition_data.result.total_damage as i32
    } else {
        game_manager.hero.current_health // All HP lost
    };
    game_manager.hero.take_damage(damage_taken);

    // Update hero state based on survival
    if survived {
        // Hero survived, set to Ready
        game_manager.hero.state = crate::game::HeroState::Ready;

        // Create success summary page
        match crate::ui::pages::ExpeditionSummaryPage::new_success(
            game_manager.hero.clone(),
            initial_level,
            actual_kills,
            total_exp,
            cards_dropped,
        ) {
            Ok(summary_page) => {
                game_manager.expedition_summary_page = Some(summary_page);
                log::info!("✅ Created success summary page");
            }
            Err(e) => {
                log::error!("❌ Failed to create summary page: {:?}", e);
            }
        }
    } else {
        // Hero died, set to KO for 10 minutes
        let recovery_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 600; // 10 minutes = 600 seconds

        game_manager.hero.state = crate::game::HeroState::KO {
            recovery_time: recovery_timestamp,
        };

        // Create failure summary page
        match crate::ui::pages::ExpeditionSummaryPage::new_failure(
            game_manager.hero.clone(),
            initial_level,
            expedition_data.target_kills,
            actual_kills,
            total_exp,
        ) {
            Ok(summary_page) => {
                game_manager.expedition_summary_page = Some(summary_page);
                log::info!("💀 Created failure summary page (KO for 10 minutes)");
            }
            Err(e) => {
                log::error!("❌ Failed to create summary page: {:?}", e);
            }
        }
    }
}
