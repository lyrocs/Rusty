//! Menu navigation system
//!
//! Handles menu interactions and navigation to different game modes.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle menu navigation
pub fn menu_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Menu mode
    if app_state.current_mode != AppMode::Menu {
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
                log::info!("Menu touch at ({}, {})", x, y);

                // Handle touch on menu page
                if let Some(action) = game_manager.menu_page.handle_touch(*x as i32, *y as i32) {
                    // Navigate based on selected action
                    use crate::ui::pages::menu::MenuAction;
                    match action {
                        MenuAction::Map => {
                            log::info!("Navigating to Map");
                            app_state.current_mode = AppMode::Map;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Battle => {
                            log::info!("Navigating to Battle");
                            // Only switch to battle if there's an active battle
                            if game_manager.battle_page.is_some() {
                                app_state.current_mode = AppMode::Battle;
                                app_state.needs_redraw = true;
                            } else {
                                log::warn!("No active battle");
                            }
                        }
                        MenuAction::Rest => {
                            log::info!("Navigating to Rest screen");
                            // Create rest page with hero
                            match crate::ui::pages::RestPage::new(game_manager.hero.clone()) {
                                Ok(rest_page) => {
                                    game_manager.rest_page = Some(rest_page);
                                    app_state.current_mode = AppMode::Rest;
                                    app_state.needs_redraw = true;
                                    log::info!("✅ Rest page created for {}", game_manager.hero.name);
                                }
                                Err(e) => {
                                    log::error!("Failed to create rest page: {:?}", e);
                                }
                            }
                        }
                        MenuAction::Hero => {
                            log::info!("Navigating to Hero Info");
                            // Create hero info page
                            match crate::ui::pages::HeroInfoPage::new(game_manager.hero.clone()) {
                                Ok(hero_page) => {
                                    game_manager.hero_info_page = Some(hero_page);
                                    app_state.current_mode = AppMode::HeroInfo;
                                    app_state.needs_redraw = true;
                                }
                                Err(e) => {
                                    log::error!("Failed to create hero info page: {:?}", e);
                                }
                            }
                        }
                        MenuAction::Quests => {
                            log::info!("Navigating to Quest List");
                            // Auto-start daily quests when opening quest page
                            game_manager.check_quest_resets();
                            game_manager.auto_start_daily_quests();
                            app_state.current_mode = AppMode::QuestList;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Cards => {
                            log::info!("Navigating to Card Collection");
                            // Create cards collection page with all game data and hero's owned cards
                            match crate::ui::pages::CardsPage::new(
                                game_manager.game_data.clone(),
                                game_manager.hero.cards.clone()
                            ) {
                                Ok(cards_page) => {
                                    game_manager.cards_page = Some(cards_page);
                                    app_state.current_mode = AppMode::CardCollection;
                                    app_state.needs_redraw = true;
                                }
                                Err(e) => {
                                    log::error!("Failed to create cards page: {:?}", e);
                                }
                            }
                        }
                        MenuAction::Pokemon => {
                            log::info!("Fetching Pokemon data from API...");
                            // Call Pokemon API
                            match crate::wifi::http_get("https://pokeapi.co/api/v2/pokemon/ditto") {
                                Ok(response) => {
                                    log::info!("Pokemon API response received ({} bytes)", response.len());
                                    // Parse JSON to extract name
                                    match serde_json::from_str::<serde_json::Value>(&response) {
                                        Ok(json) => {
                                            if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                                                log::info!("Pokemon name: {}", name);
                                            }
                                            // Store response in game manager for display
                                            game_manager.pokemon_api_response = Some(response);
                                            app_state.current_mode = AppMode::PokemonInfo;
                                            app_state.needs_redraw = true;
                                        }
                                        Err(e) => {
                                            log::error!("Failed to parse Pokemon JSON: {:?}", e);
                                            game_manager.pokemon_api_response = Some(format!("Error parsing JSON: {}", e));
                                            app_state.current_mode = AppMode::PokemonInfo;
                                            app_state.needs_redraw = true;
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to fetch Pokemon data: {:?}", e);
                                    game_manager.pokemon_api_response = Some(format!("Error: {:?}", e));
                                    app_state.current_mode = AppMode::PokemonInfo;
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Ignore other events in menu mode
            }
        }
    }
}
