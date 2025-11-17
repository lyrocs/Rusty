//! Quest navigation system
//!
//! Handles quest list interactions like claiming rewards and closing.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::quest_list::QuestListAction;

/// System to handle quest list navigation
pub fn quest_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in QuestList mode
    if app_state.current_mode != AppMode::QuestList {
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
                log::info!("Quest list touch at ({}, {})", x, y);

                // Handle touch on quest list page
                if let Some(action) = game_manager
                    .quest_list_page
                    .handle_touch(*x as i32, *y as i32)
                {
                    match action {
                        QuestListAction::ClaimReward(quest_id) => {
                            log::info!("Claiming reward for quest ID: {}", quest_id);

                            // Get quest data first
                            let quest_data = game_manager.game_data.get_quest(quest_id).cloned();

                            if let Some(quest_data) = quest_data {
                                // Claim rewards
                                if let Some(rewards) =
                                    game_manager.quest_manager.claim_rewards(quest_id, &quest_data)
                                {
                                    // Apply EXP reward to active Rustymon
                                    if rewards.exp > 0 {
                                        let active_id = game_manager
                                            .rustymon_team
                                            .get_active_rustymon_id()
                                            .cloned();
                                        if let Some(id) = active_id {
                                            if let Some(rustymon) = game_manager
                                                .rustymon_collection
                                                .iter_mut()
                                                .find(|r| r.id == id)
                                            {
                                                let leveled_up =
                                                    rustymon.gain_exp(rewards.exp as u32);
                                                log::info!(
                                                    "Quest reward: {} gained {} EXP (Lv{})",
                                                    rustymon.name,
                                                    rewards.exp,
                                                    rustymon.level
                                                );
                                                if leveled_up {
                                                    log::info!(
                                                        "🎉 {} leveled up to {}!",
                                                        rustymon.name,
                                                        rustymon.level
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    // Apply fragment rewards
                                    for fragment_reward in &rewards.fragments {
                                        game_manager.fragment_collection.add_fragment(
                                            fragment_reward.monster_id,
                                            fragment_reward.amount,
                                        );
                                        log::info!(
                                            "Quest reward: {} fragments of monster #{}",
                                            fragment_reward.amount,
                                            fragment_reward.monster_id
                                        );
                                    }

                                    log::info!(
                                        "Quest {} rewards claimed successfully!",
                                        quest_data.name
                                    );
                                }
                            }

                            // Redraw to update quest list
                            game_manager.quest_list_page.mark_redraw();
                            app_state.needs_redraw = true;
                        }
                        QuestListAction::ScrollUp => {
                            game_manager.quest_list_page.scroll_up();
                            app_state.needs_redraw = true;
                        }
                        QuestListAction::ScrollDown => {
                            let total_quests = game_manager
                                .quest_manager
                                .active_quests
                                .iter()
                                .filter(|(id, _)| {
                                    game_manager
                                        .game_data
                                        .get_quest(**id)
                                        .map(|q| q.is_daily())
                                        .unwrap_or(false)
                                })
                                .count();
                            game_manager.quest_list_page.scroll_down(total_quests);
                            app_state.needs_redraw = true;
                        }
                        QuestListAction::Close => {
                            log::info!("Closing quest list, returning to menu");
                            app_state.current_mode = AppMode::Menu;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe right to go back to menu
                if *direction == SwipeDirection::Right {
                    log::info!("Swipe right: closing quest list, returning to menu");
                    app_state.current_mode = AppMode::Menu;
                    app_state.needs_redraw = true;
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
