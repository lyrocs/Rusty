//! Rendering system (Stub)
//!
//! NOTE: Simplified for Phase 1 migration.
//! Will be replaced with proper rendering in Phase 2.

use bevy_ecs::prelude::*;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X15},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};
use log::info;

use crate::ecs::resources::{AppMode, AppState, DisplayResource, GameManager, PageResource};
use crate::ui::page::Page;

/// System to render the display
pub fn render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut app_state: ResMut<AppState>,
    _page_res: Option<NonSendMut<PageResource>>,
    game_manager: Option<NonSendMut<GameManager>>,
) {
    let display = &mut display_res.display;

    // Handle screen on/off state changes
    static mut LAST_SCREEN_STATE: bool = true;
    let screen_state_changed = unsafe {
        let changed = LAST_SCREEN_STATE != app_state.screen_on;
        LAST_SCREEN_STATE = app_state.screen_on;
        changed
    };

    if screen_state_changed {
        if app_state.screen_on {
            if let Err(e) = display.display_on() {
                log::error!("Failed to turn display on: {:?}", e);
            }
        } else {
            if let Err(e) = display.display_off() {
                log::error!("Failed to turn display off: {:?}", e);
            }
            app_state.needs_redraw = false;
            return;
        }
    }

    // Skip rendering if screen is off
    if !app_state.screen_on {
        if app_state.current_mode == AppMode::Battle {
            if let Some(mut game_manager) = game_manager {
                if let Some(page) = game_manager.get_current_page(AppMode::Battle) {
                    let page_active = page.update();
                    if !page_active {
                        info!("Battle completed while screen off, returning to home");
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        return;
    }

    match app_state.current_mode {
        AppMode::BattleLoading => {
            if app_state.needs_redraw {
                if let Err(e) = draw_loading_screen(display) {
                    log::error!("Failed to draw loading screen: {:?}", e);
                }
                app_state.needs_redraw = false;
            }
        }
        AppMode::Home | AppMode::Menu | AppMode::Map | AppMode::Battle | AppMode::BattleResult
        | AppMode::MonsterList | AppMode::MonsterDetail | AppMode::MonsterUpgrade
        | AppMode::ExpeditionMap | AppMode::ExpeditionTeamSelect | AppMode::ExpeditionResult
        | AppMode::ExpeditionDetail | AppMode::Inventory | AppMode::Collection
        | AppMode::DungeonList | AppMode::DungeonInfo
        | AppMode::DungeonCombat | AppMode::BetweenFloors | AppMode::DungeonDefeat => {
            if let Some(mut game_manager) = game_manager {
                if app_state.current_mode == AppMode::Menu {
                    let has_battle = game_manager.battle_page.is_some();
                    game_manager.menu_page.set_has_active_battle(has_battle);
                }

                if let Some(page) = game_manager.get_current_page(app_state.current_mode) {
                    let page_active = page.update();
                    let full_redraw = page.needs_full_redraw() || app_state.needs_redraw;

                    if let Err(e) = page.draw(display, full_redraw) {
                        log::error!("Failed to draw page: {:?}", e);
                    }

                    if app_state.needs_redraw {
                        app_state.needs_redraw = false;
                    }

                    if !page_active {
                        // Special handling for different page types
                        if app_state.current_mode == AppMode::DungeonCombat {
                            // Get combat results before cleanup
                            if let Some(ref combat_page) = game_manager.dungeon_combat_page {
                                if let Some((victory, crystals, _xp)) = combat_page.combat_result() {
                                    if victory {
                                        game_manager.player.crystals += crystals;
                                        log::info!("Dungeon combat victory! +{} crystals", crystals);
                                    } else {
                                        log::info!("Dungeon combat defeat");
                                    }
                                }
                            }
                            game_manager.dungeon_combat_page = None;
                            app_state.current_mode = AppMode::Home;
                        } else {
                            log::info!("Page completed, returning to home");
                            app_state.current_mode = AppMode::Home;
                        }
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        AppMode::Death => {
            if let Some(mut game_manager) = game_manager {
                if let Some(ref mut death_page) = game_manager.death_page {
                    let page_active = death_page.update();
                    let full_redraw = death_page.needs_full_redraw() || app_state.needs_redraw;

                    if let Err(e) = death_page.draw_death_page(display, full_redraw) {
                        log::error!("Failed to draw death page: {:?}", e);
                    }

                    if app_state.needs_redraw {
                        app_state.needs_redraw = false;
                    }

                    if !page_active {
                        info!("Death page completed, returning to home");
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
    }
}

/// Draw loading screen
fn draw_loading_screen(
    display: &mut crate::display::St7789pDriver,
) -> Result<(), Box<dyn std::error::Error>> {
    display.clear(Rgb888::BLACK)?;

    let text_style = MonoTextStyle::new(&FONT_9X15, Rgb888::CYAN);
    Text::new("Loading Battle...", Point::new(120, 220), text_style).draw(display)?;

    display.flush()?;
    Ok(())
}
