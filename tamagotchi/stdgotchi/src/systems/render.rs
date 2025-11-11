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
        AppMode::Menu | AppMode::Map | AppMode::Battle => {
            // Game-based rendering with GameManager
            if let Some(mut game_manager) = game_manager {
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
                        info!("Page completed, returning to map");
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
        AppMode::HeroOverview => {
            // Hero overview rendering - needs special handling for hero data
            if let Some(mut game_manager) = game_manager {
                // Update page logic
                let page_active = game_manager.hero_overview_page.update();

                // Check if page needs full redraw
                let full_redraw = game_manager.hero_overview_page.needs_full_redraw() || app_state.needs_redraw;

                // Draw the page with hero data using helper method
                if let Err(e) = game_manager.draw_hero_overview(display, full_redraw) {
                    log::error!("Failed to draw hero overview: {:?}", e);
                }

                // Reset needs_redraw flag
                if app_state.needs_redraw {
                    app_state.needs_redraw = false;
                }

                // Check if page is done
                if !page_active {
                    info!("Hero overview completed, returning to map");
                    app_state.current_mode = AppMode::Map;
                    app_state.needs_redraw = true;
                }
            }
        }
        AppMode::StatsAllocation => {
            // Stats allocation rendering - needs hero data
            if let Some(mut game_manager) = game_manager {
                let page_active = game_manager.stats_allocation_page.update();
                let full_redraw = game_manager.stats_allocation_page.needs_full_redraw() || app_state.needs_redraw;

                // Draw stats allocation with hero data
                if let Err(e) = game_manager.draw_stats_allocation(display, full_redraw) {
                    log::error!("Failed to draw stats allocation: {:?}", e);
                }

                if app_state.needs_redraw {
                    app_state.needs_redraw = false;
                }

                if !page_active {
                    info!("Stats allocation closed, returning to hero overview");
                    app_state.current_mode = AppMode::HeroOverview;
                    app_state.needs_redraw = true;
                }
            }
        }
        AppMode::Inventory => {
            // Inventory rendering - needs hero and game data
            if let Some(mut game_manager) = game_manager {
                let page_active = game_manager.inventory_page.update();
                let full_redraw = game_manager.inventory_page.needs_full_redraw() || app_state.needs_redraw;

                // Draw inventory with hero data
                if let Err(e) = game_manager.draw_inventory(display, full_redraw) {
                    log::error!("Failed to draw inventory: {:?}", e);
                }

                if app_state.needs_redraw {
                    app_state.needs_redraw = false;
                }

                if !page_active {
                    info!("Inventory closed, returning to menu");
                    app_state.current_mode = AppMode::Menu;
                    app_state.needs_redraw = true;
                }
            }
        }
        AppMode::Equipment => {
            // Equipment rendering - needs hero and game data
            if let Some(mut game_manager) = game_manager {
                let page_active = game_manager.equipment_page.update();
                let full_redraw = game_manager.equipment_page.needs_full_redraw() || app_state.needs_redraw;

                // Draw equipment with hero data
                if let Err(e) = game_manager.draw_equipment(display, full_redraw) {
                    log::error!("Failed to draw equipment: {:?}", e);
                }

                if app_state.needs_redraw {
                    app_state.needs_redraw = false;
                }

                if !page_active {
                    info!("Equipment closed, returning to menu");
                    app_state.current_mode = AppMode::Menu;
                    app_state.needs_redraw = true;
                }
            }
        }
        AppMode::Crafting => {
            // Crafting rendering - needs hero and game data
            if let Some(mut game_manager) = game_manager {
                let page_active = game_manager.crafting_page.update();
                let full_redraw = game_manager.crafting_page.needs_full_redraw() || app_state.needs_redraw;

                // Draw crafting with hero data
                if let Err(e) = game_manager.draw_crafting(display, full_redraw) {
                    log::error!("Failed to draw crafting: {:?}", e);
                }

                if app_state.needs_redraw {
                    app_state.needs_redraw = false;
                }

                if !page_active {
                    info!("Crafting closed, returning to map");
                    app_state.current_mode = AppMode::Map;
                    app_state.needs_redraw = true;
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
