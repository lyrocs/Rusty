//! Expedition In Progress System
//!
//! Monitors ongoing expedition and transitions to summary when complete

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};

/// System to monitor expedition progress
pub fn expedition_in_progress_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in ExpeditionInProgress mode
    if app_state.current_mode != AppMode::ExpeditionInProgress {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check if expedition is complete
    if let Some(ref progress_page) = game_manager.expedition_in_progress_page {
        if progress_page.is_complete() {
            log::info!("Expedition complete! Transitioning to summary...");

            // Complete the expedition
            complete_expedition(game_manager);

            // Transition to summary mode
            app_state.current_mode = AppMode::ExpeditionSummary;
            app_state.needs_redraw = true;
        }
    }

    // Handle input events (swipe right to return to map while expedition continues)
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Swipe { direction } => {
                if *direction == SwipeDirection::Right {
                    log::info!("Returning to map while expedition continues in background");
                    app_state.current_mode = AppMode::Map;
                    app_state.needs_redraw = true;
                }
            }
            _ => {
                // Ignore other inputs
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
    let result = &expedition_data.result;
    let survived = result.survives;
    let kills = if survived {
        expedition_data.target_kills
    } else {
        result.kills_completed
    };

    log::info!("Expedition result: survived={}, kills={}/{}",
        survived, kills, expedition_data.target_kills);

    // Calculate cards dropped
    let cards = if survived {
        use crate::game::expedition::calculate_drops;
        use crate::game::{Enemy, Card};

        // Get enemy data
        if let Some(enemy_data) = game_manager.game_data.get_enemy(expedition_data.enemy_id) {
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

            // Create card template
            let card_template = Card {
                monster_id: enemy_data.id,
                name: enemy_data.card.name.clone(),
                rarity: enemy_data.card.rarity,
                atk_bonus: enemy_data.card.atk_bonus,
                def_bonus: enemy_data.card.def_bonus,
            };

            calculate_drops(&enemy, &card_template, enemy_data.drop_rate, kills, expedition_data.expedition_size)
        } else {
            Vec::new()
        }
    } else {
        Vec::new() // No loot on death
    };

    // Calculate experience gained (base_exp * kills)
    let base_exp = if let Some(enemy_data) = game_manager.game_data.get_enemy(expedition_data.enemy_id) {
        enemy_data.base_exp
    } else {
        0
    };
    let exp_gained = (base_exp as u32) * kills;

    // Update hero
    let mut hero = game_manager.hero.clone();
    let initial_level = hero.level; // Capture level BEFORE applying experience

    if survived {
        // Add experience
        hero.gain_experience(exp_gained);

        // Add cards to collection
        for card in &cards {
            hero.cards.push(card.clone());
        }

        // Deduct health
        hero.current_health = (hero.current_health as f32 - result.total_damage) as i32;
        hero.current_health = hero.current_health.max(1); // Never go to 0 if survived

        // Hero is ready for next expedition
        hero.state = crate::game::HeroState::Ready;
    } else {
        // Hero died - set to KO state (10 minute recovery)
        use std::time::{SystemTime, UNIX_EPOCH};
        let recovery_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + (10 * 60); // 10 minutes

        hero.state = crate::game::HeroState::KO { recovery_time };

        // Still gain some experience
        hero.gain_experience(exp_gained);

        // Reset health to 1
        hero.current_health = 1;
    }

    // Update game manager
    game_manager.hero = hero.clone();

    // Create summary page
    let summary_page = if survived {
        crate::ui::pages::ExpeditionSummaryPage::new_success(
            hero,
            initial_level,
            kills,
            exp_gained,
            cards,
        )
    } else {
        crate::ui::pages::ExpeditionSummaryPage::new_failure(
            hero,
            initial_level,
            expedition_data.target_kills,
            kills,
            exp_gained,
        )
    };

    match summary_page {
        Ok(page) => {
            game_manager.expedition_summary_page = Some(page);
        }
        Err(e) => {
            log::error!("Failed to create expedition summary page: {:?}", e);
        }
    }
}
