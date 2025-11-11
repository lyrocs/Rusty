//! Stats Allocation System
//!
//! Handles stats allocation page interactions

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, InputEventChannel};
use crate::input_thread::InputEvent;

/// Stat type for allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatType {
    Str,
    Agi,
    Vit,
    Int,
    Dex,
    Luk,
}

/// System to handle stats allocation interactions
pub fn stats_allocation_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in StatsAllocation mode
    if app_state.current_mode != AppMode::StatsAllocation {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = x as i32;
                let y = y as i32;

                // Handle touch on stats allocation page
                if let Some(action) = game_manager.handle_stats_allocation_touch(x, y) {
                    use crate::ui::pages::stats_allocation::ButtonAction;
                    match action {
                        ButtonAction::IncreaseStr => allocate_point(StatType::Str, game_manager),
                        ButtonAction::DecreaseStr => deallocate_point(StatType::Str, game_manager),
                        ButtonAction::IncreaseAgi => allocate_point(StatType::Agi, game_manager),
                        ButtonAction::DecreaseAgi => deallocate_point(StatType::Agi, game_manager),
                        ButtonAction::IncreaseVit => allocate_point(StatType::Vit, game_manager),
                        ButtonAction::DecreaseVit => deallocate_point(StatType::Vit, game_manager),
                        ButtonAction::IncreaseInt => allocate_point(StatType::Int, game_manager),
                        ButtonAction::DecreaseInt => deallocate_point(StatType::Int, game_manager),
                        ButtonAction::IncreaseDex => allocate_point(StatType::Dex, game_manager),
                        ButtonAction::DecreaseDex => deallocate_point(StatType::Dex, game_manager),
                        ButtonAction::IncreaseLuk => allocate_point(StatType::Luk, game_manager),
                        ButtonAction::DecreaseLuk => deallocate_point(StatType::Luk, game_manager),
                        ButtonAction::ResetStats => reset_stats(game_manager),
                        ButtonAction::Close => {
                            log::info!("Closing stats allocation - returning to overview");
                            app_state.current_mode = AppMode::HeroOverview;
                            app_state.needs_redraw = true;
                        }
                    }
                    app_state.needs_redraw = true;
                }
            }
            InputEvent::BootPressed => {
                // Boot button closes stats allocation
                log::info!("Boot button pressed - returning to overview");
                app_state.current_mode = AppMode::HeroOverview;
                app_state.needs_redraw = true;
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// Allocate a stat point
fn allocate_point(stat: StatType, game_manager: &mut GameManager) {
    let hero = &mut game_manager.hero;

    // Check if we have points available
    if hero.stat_points == 0 {
        log::warn!("No stat points available");
        return;
    }

    // Check max stat limit (99)
    let current_value = match stat {
        StatType::Str => hero.stats.str,
        StatType::Agi => hero.stats.agi,
        StatType::Vit => hero.stats.vit,
        StatType::Int => hero.stats.int,
        StatType::Dex => hero.stats.dex,
        StatType::Luk => hero.stats.luk,
    };

    if current_value >= 99 {
        log::warn!("Stat already at maximum (99)");
        return;
    }

    // Allocate the point
    match stat {
        StatType::Str => hero.stats.str += 1,
        StatType::Agi => hero.stats.agi += 1,
        StatType::Vit => hero.stats.vit += 1,
        StatType::Int => hero.stats.int += 1,
        StatType::Dex => hero.stats.dex += 1,
        StatType::Luk => hero.stats.luk += 1,
    }

    hero.stat_points -= 1;

    // Update hero HP/SP based on new stats
    hero.recalculate_max_hp_sp();

    log::info!("Allocated point to {:?}, remaining: {}", stat, hero.stat_points);
}

/// Deallocate a stat point (undo allocation)
fn deallocate_point(stat: StatType, game_manager: &mut GameManager) {
    let hero = &mut game_manager.hero;

    // Get base stat from job
    let base_value = match stat {
        StatType::Str => hero.job.base_stats().str,
        StatType::Agi => hero.job.base_stats().agi,
        StatType::Vit => hero.job.base_stats().vit,
        StatType::Int => hero.job.base_stats().int,
        StatType::Dex => hero.job.base_stats().dex,
        StatType::Luk => hero.job.base_stats().luk,
    };

    // Check if we can decrease (can't go below base)
    let current_value = match stat {
        StatType::Str => hero.stats.str,
        StatType::Agi => hero.stats.agi,
        StatType::Vit => hero.stats.vit,
        StatType::Int => hero.stats.int,
        StatType::Dex => hero.stats.dex,
        StatType::Luk => hero.stats.luk,
    };

    if current_value <= base_value {
        log::warn!("Cannot decrease stat below base value");
        return;
    }

    // Deallocate the point
    match stat {
        StatType::Str => hero.stats.str -= 1,
        StatType::Agi => hero.stats.agi -= 1,
        StatType::Vit => hero.stats.vit -= 1,
        StatType::Int => hero.stats.int -= 1,
        StatType::Dex => hero.stats.dex -= 1,
        StatType::Luk => hero.stats.luk -= 1,
    }

    hero.stat_points += 1;

    // Update hero HP/SP
    hero.recalculate_max_hp_sp();

    log::info!("Deallocated point from {:?}, remaining: {}", stat, hero.stat_points);
}

/// Reset all stat allocations
fn reset_stats(game_manager: &mut GameManager) {
    let hero = &mut game_manager.hero;

    log::info!("Resetting stats to base values");

    // Reset to base stats
    hero.stats = hero.job.base_stats();

    // Restore all stat points (3 per level, adjusted for base level 1)
    hero.stat_points = (hero.level.saturating_sub(1)) * 3;

    // Recalculate HP/SP
    hero.recalculate_max_hp_sp();

    log::info!("Stats reset, {} points available", hero.stat_points);
}
