//! Rendering system
//!
//! Handles display updates based on app state.

use bevy_ecs::prelude::*;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
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
            // Turn display on
            if let Err(e) = display.display_on() {
                log::error!("Failed to turn display on: {:?}", e);
            }
        } else {
            // Turn display off
            if let Err(e) = display.display_off() {
                log::error!("Failed to turn display off: {:?}", e);
            }
            app_state.needs_redraw = false;
            return; // Skip rendering
        }
    }

    // Skip rendering if screen is off (but still update battle logic if in battle)
    if !app_state.screen_on {
        // Battle mode needs to continue updating even when screen is off
        if app_state.current_mode == AppMode::Battle {
            if let Some(mut game_manager) = game_manager {
                if let Some(page) = game_manager.get_current_page(AppMode::Battle) {
                    // Update battle logic (enemy AI, attacks, status effects)
                    let page_active = page.update();

                    // Check if battle is done
                    if !page_active {
                        info!("Battle completed while screen off, returning to map");
                        app_state.current_mode = AppMode::Map;
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        return;
    }

    match app_state.current_mode {
        AppMode::BattleLoading => {
            // Draw loading screen
            if app_state.needs_redraw {
                if let Err(e) = draw_loading_screen(display) {
                    log::error!("Failed to draw loading screen: {:?}", e);
                }
                app_state.needs_redraw = false;
            }
        }
        AppMode::Menu | AppMode::Map | AppMode::Battle | AppMode::BattleResult
        | AppMode::ExpeditionSetup | AppMode::ExpeditionInProgress | AppMode::ExpeditionSummary => {
            // Game-based rendering with GameManager
            if let Some(mut game_manager) = game_manager {
                // Update menu battle state if in Menu mode
                if app_state.current_mode == AppMode::Menu {
                    let has_battle = game_manager.battle_page.is_some();
                    game_manager.menu_page.set_has_active_battle(has_battle);
                }

                if let Some(page) = game_manager.get_current_page(app_state.current_mode) {
                    // Update page logic
                    let page_active = page.update();

                    // Check if page needs full redraw
                    let full_redraw = page.needs_full_redraw() || app_state.needs_redraw;

                    // Draw the page
                    if let Err(e) = page.draw(display, full_redraw) {
                        log::error!("Failed to draw page: {:?}", e);
                    }

                    // Reset needs_redraw flag
                    if app_state.needs_redraw {
                        app_state.needs_redraw = false;
                    }

                    // Check if page is done
                    if !page_active {
                        log::info!("Page completed, returning to map");
                        app_state.current_mode = AppMode::Map;
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        AppMode::Death => {
            // Death screen rendering
            if let Some(mut game_manager) = game_manager {
                if let Some(ref mut death_page) = game_manager.death_page {
                    let page_active = death_page.update();
                    let full_redraw = death_page.needs_full_redraw() || app_state.needs_redraw;

                    // Draw death page
                    if let Err(e) = death_page.draw_death_page(display, full_redraw) {
                        log::error!("Failed to draw death page: {:?}", e);
                    }

                    if app_state.needs_redraw {
                        app_state.needs_redraw = false;
                    }

                    if !page_active {
                        info!("Death page completed, returning to map");
                        app_state.current_mode = AppMode::Map;
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        AppMode::Rest => {
            // Rest screen rendering
            if let Some(mut game_manager) = game_manager {
                if let Some(ref mut rest_page) = game_manager.rest_page {
                    let page_active = rest_page.update();
                    let full_redraw = rest_page.needs_full_redraw() || app_state.needs_redraw;

                    // Draw rest page
                    if let Err(e) = rest_page.draw(display, full_redraw) {
                        log::error!("Failed to draw rest page: {:?}", e);
                    }

                    if app_state.needs_redraw {
                        app_state.needs_redraw = false;
                    }

                    if !page_active {
                        log::info!("Rest page completed, returning to menu");
                        app_state.current_mode = AppMode::Menu;
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        AppMode::AfkFarm => {
            // AFK Farm screen rendering
            if let Some(mut game_manager) = game_manager {
                if let Some(ref mut afk_farm_page) = game_manager.afk_farm_page {
                    let page_active = afk_farm_page.update();
                    let full_redraw = afk_farm_page.needs_full_redraw() || app_state.needs_redraw;

                    // Draw AFK farm page
                    if let Err(e) = afk_farm_page.draw(display, full_redraw) {
                        log::error!("Failed to draw AFK farm page: {:?}", e);
                    }

                    if app_state.needs_redraw {
                        app_state.needs_redraw = false;
                    }

                    if !page_active {
                        log::info!("AFK farming completed, returning to map");
                        app_state.current_mode = AppMode::Map;
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        // Rustymon-related modes removed in hero system migration
        // (RustymonList, RustymonDetail, RustymonSkills, FragmentCollection, RustymonSummon)
        AppMode::QuestList => {
            // Quest list rendering - needs quest manager and game data
            if let Some(mut game_manager) = game_manager {
                let page_active = game_manager.quest_list_page.update();
                let full_redraw = game_manager.quest_list_page.needs_full_redraw() || app_state.needs_redraw;

                // Draw quest list
                if let Err(e) = game_manager.draw_quest_list(display, full_redraw) {
                    log::error!("Failed to draw quest list: {:?}", e);
                }

                if app_state.needs_redraw {
                    app_state.needs_redraw = false;
                }

                if !page_active {
                    info!("Quest list closed, returning to menu");
                    app_state.current_mode = AppMode::Menu;
                    app_state.needs_redraw = true;
                }
            }
        }
        AppMode::PokemonInfo => {
            // Pokemon API info display
            if let Some(game_manager) = game_manager {
                if app_state.needs_redraw {
                    use embedded_graphics::{
                        mono_font::{MonoTextStyle, ascii::FONT_6X10},
                        pixelcolor::Rgb888,
                        prelude::*,
                        text::Text,
                    };

                    if let Err(e) = display.clear(Rgb888::new(15, 20, 30)) {
                        log::error!("Failed to clear display: {:?}", e);
                        return;
                    }

                    let title_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
                    let _ = Text::new("POKEMON API RESPONSE", Point::new(10, 20), title_style).draw(display);

                    if let Some(ref response) = game_manager.pokemon_api_response {
                        // Display Pokemon data
                        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 200, 200));

                        // Try to parse and pretty-print JSON
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
                            let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                            let id = json.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                            let height = json.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                            let weight = json.get("weight").and_then(|v| v.as_u64()).unwrap_or(0);

                            let mut y = 50;
                            let line_height = 15;

                            let _ = Text::new(&format!("Name: {}", name), Point::new(10, y), text_style).draw(display);
                            y += line_height;
                            let _ = Text::new(&format!("ID: {}", id), Point::new(10, y), text_style).draw(display);
                            y += line_height;
                            let _ = Text::new(&format!("Height: {}", height), Point::new(10, y), text_style).draw(display);
                            y += line_height;
                            let _ = Text::new(&format!("Weight: {}", weight), Point::new(10, y), text_style).draw(display);
                            y += line_height + 10;

                            // Show abilities
                            if let Some(abilities) = json.get("abilities").and_then(|v| v.as_array()) {
                                let _ = Text::new("Abilities:", Point::new(10, y), text_style).draw(display);
                                y += line_height;
                                for ability in abilities.iter().take(5) {
                                    if let Some(ability_name) = ability.get("ability")
                                        .and_then(|a| a.get("name"))
                                        .and_then(|n| n.as_str()) {
                                        let _ = Text::new(&format!("  - {}", ability_name), Point::new(10, y), text_style).draw(display);
                                        y += line_height;
                                    }
                                }
                            }
                        } else {
                            // Show raw response if parsing fails
                            let text_style_small = MonoTextStyle::new(&FONT_6X10, Rgb888::new(180, 180, 180));
                            let max_chars = 50;
                            for (i, line) in response.lines().take(20).enumerate() {
                                let truncated = if line.len() > max_chars {
                                    &line[..max_chars]
                                } else {
                                    line
                                };
                                let _ = Text::new(truncated, Point::new(10, 50 + (i as i32 * 12)), text_style_small).draw(display);
                            }
                        }
                    } else {
                        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(200, 100, 100));
                        let _ = Text::new("No data available", Point::new(10, 50), text_style).draw(display);
                    }

                    // Show hint
                    let hint_style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(100, 100, 100));
                    let _ = Text::new("Tap to return to menu", Point::new(10, 430), hint_style).draw(display);

                    if let Err(e) = display.flush() {
                        log::error!("Failed to flush display: {:?}", e);
                    }
                    app_state.needs_redraw = false;
                }
            }
        }
    }
}

/// Draw FPS text overlay
fn draw_fps_text(
    display: &mut crate::display::Sh8601Driver,
    fps: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    use core::fmt::Write;
    let mut fps_str = heapless::String::<16>::new();
    write!(fps_str, "FPS: {:.1}", fps).ok();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::YELLOW);
    Text::new(&fps_str, Point::new(10, 10), text_style).draw(display)?;
    Ok(())
}

/// Draw FPS overlay (small box with FPS counter)
fn draw_fps_overlay(
    display: &mut crate::display::Sh8601Driver,
    fps: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Draw semi-transparent background box for FPS
    Rectangle::new(Point::new(5, 2), Size::new(70, 15))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    draw_fps_text(display, fps)?;
    display.flush()?;
    Ok(())
}

/// Draw the initial welcome screen
fn draw_welcome_screen(
    display: &mut crate::display::Sh8601Driver,
    fps: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    display.clear(Rgb888::BLACK)?;

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::GREEN);
    Text::new("stdgotchi", Point::new(10, 30), text_style).draw(display)?;
    Text::new("ESP32-S3 AMOLED", Point::new(10, 50), text_style).draw(display)?;
    Text::new("Touch & Gestures!", Point::new(10, 70), text_style).draw(display)?;
    Text::new("Swipe down for GIF", Point::new(10, 90), text_style).draw(display)?;

    Circle::new(Point::new(50, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
        .draw(display)?;

    Circle::new(Point::new(100, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
        .draw(display)?;

    Circle::new(Point::new(150, 150), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::MAGENTA))
        .draw(display)?;

    draw_fps_text(display, fps)?;
    display.flush()?;
    Ok(())
}

/// Draw loading screen
fn draw_loading_screen(
    display: &mut crate::display::Sh8601Driver,
) -> Result<(), Box<dyn std::error::Error>> {
    display.clear(Rgb888::BLACK)?;

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::CYAN);
    Text::new("Loading Battle...", Point::new(120, 220), text_style).draw(display)?;

    display.flush()?;
    Ok(())
}
