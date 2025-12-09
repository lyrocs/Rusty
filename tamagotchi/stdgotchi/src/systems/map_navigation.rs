//! Map navigation system (Stub)
//!
//! NOTE: Simplified for Phase 1 migration.
//! Will be replaced with proper navigation and battle initiation in Phase 2.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::game::core::MonsterStatus;
use crate::game::systems::combat::CombatState;
use crate::game::systems::dungeon::{DungeonRun, floor_stat_multiplier};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{DungeonCombatPage, ExpeditionTeamSelectPage, MonsterSelectData};

/// System to handle map navigation
pub fn map_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Map mode
    if app_state.current_mode != AppMode::Map {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Initialize game data if not already set (only once, not every frame)
    if !game_manager.map_page.has_game_data() {
        let zones: Vec<_> = game_manager.tamer_data.all_zones().cloned().collect();
        let maps: Vec<_> = game_manager.tamer_data.all_tamer_maps().cloned().collect();
        game_manager.map_page.set_game_data(zones, maps);
        log::info!("Map page game data initialized");
    }

    // Update dungeon progress on the map page (clone to avoid borrow conflict)
    let progress = game_manager.dungeon_progress.clone();
    game_manager.map_page.update_dungeon_progress(&progress);

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;
                log::info!("Touch at ({}, {})", x, y);

                // Handle touch on map page
                if let Some(action) = game_manager.map_page.handle_touch(x, y) {
                    use crate::ui::pages::TouchAction;
                    match action {
                        TouchAction::StartExpedition(map_id) => {
                            log::info!("Starting expedition on map: {}", map_id);
                            // Create expedition team selection page
                            if let Some(expedition_page) = create_expedition_team_page(
                                game_manager,
                                &map_id,
                            ) {
                                game_manager.expedition_team_page = Some(expedition_page);
                                game_manager.selected_expedition_map_id = Some(map_id);
                                app_state.current_mode = AppMode::ExpeditionTeamSelect;
                                app_state.needs_redraw = true;
                            }
                        }
                        TouchAction::StartDungeon { dungeon_id, start_floor } => {
                            log::info!("Map -> Dungeon Combat: {} from floor {}", dungeon_id, start_floor);
                            // Start dungeon combat
                            if let Some((combat_page, dungeon_run)) = create_dungeon_combat(
                                game_manager,
                                &dungeon_id,
                                start_floor,
                            ) {
                                game_manager.dungeon_combat_page = Some(combat_page);
                                game_manager.active_dungeon_run = Some(dungeon_run);
                                game_manager.selected_dungeon_id = Some(dungeon_id);
                                app_state.current_mode = AppMode::DungeonCombat;
                                app_state.needs_redraw = true;
                            } else {
                                log::warn!("Failed to create dungeon combat - no available monsters?");
                            }
                        }
                        TouchAction::BackToHome => {
                            log::info!("Map -> Home");
                            app_state.current_mode = AppMode::Home;
                            app_state.needs_redraw = true;
                        }
                        TouchAction::None => {
                            // Internal navigation, just redraw
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe right to go back
                if *direction == SwipeDirection::Right {
                    if game_manager.map_page.is_at_top_level() {
                        log::info!("Swipe right: returning to home");
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    } else {
                        // Go back within map navigation
                        game_manager.map_page.go_back();
                        app_state.needs_redraw = true;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Create dungeon combat with player team vs dungeon enemy
fn create_dungeon_combat(
    game_manager: &mut GameManager,
    dungeon_id: &str,
    start_floor: u16,
) -> Option<(DungeonCombatPage, DungeonRun)> {
    // Clone dungeon data to avoid borrow conflict
    let dungeon = game_manager.tamer_data.get_dungeon(dungeon_id)?.clone();
    let dungeon_name = dungeon.name.clone();

    // Get player's team monsters (must be alive, can be in expedition)
    let team_ids = game_manager.team.monster_ids().to_vec();
    let mut player_monsters: Vec<crate::game::core::Monster> = Vec::new();

    for monster_id in &team_ids {
        if let Some(monster) = game_manager.get_monster_mut(monster_id) {
            // Allow monsters in expedition to also run dungeons
            if monster.is_alive() {
                // Clone monster for combat and heal it
                let mut combat_monster = monster.clone();
                combat_monster.full_heal();
                player_monsters.push(combat_monster);
            }
        }
    }

    if player_monsters.is_empty() {
        log::warn!("No available monsters for dungeon combat");
        return None;
    }

    // Generate wave enemies for this floor
    let wave_enemies = generate_floor_waves(game_manager, &dungeon, start_floor)?;

    log::info!("Starting dungeon combat: {} monsters vs {} waves in {} floor {}",
        player_monsters.len(), wave_enemies.len(), dungeon_name, start_floor);

    // Create dungeon run
    let dungeon_run = DungeonRun::new(dungeon_id.to_string(), start_floor);

    // Create combat state with waves
    let combat_state = CombatState::with_waves(player_monsters, wave_enemies, start_floor);

    // Create combat page
    let combat_page = DungeonCombatPage::new(combat_state, dungeon_name);

    Some((combat_page, dungeon_run))
}

/// Generate enemy for a dungeon floor
fn generate_floor_enemy(
    game_manager: &GameManager,
    dungeon: &crate::game::core::Dungeon,
    floor: u16,
) -> Option<crate::game::core::Monster> {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    // Check if boss floor
    if dungeon.is_boss_floor(floor) {
        if let Some(boss_species) = dungeon.get_boss_species(floor) {
            // Boss level = floor + 5
            let boss_level = (floor + 5).min(99) as u8;
            if let Some(mut boss) = game_manager.tamer_data.create_monster_at_level(boss_species, boss_level) {
                // Scale boss stats based on floor
                let multiplier = floor_stat_multiplier(floor);
                boss.hp_max = (boss.hp_max as f32 * multiplier * 1.5) as u16; // Extra HP for bosses
                boss.hp_current = boss.hp_max;
                boss.atk = (boss.atk as f32 * multiplier * 1.2) as u16;
                boss.def = (boss.def as f32 * multiplier * 1.2) as u16;
                return Some(boss);
            }
        }
    }

    // Regular enemy - get enemy pool for this floor
    let pool = dungeon.get_enemy_pool(floor)?;
    if pool.species.is_empty() {
        return None;
    }

    // Pick random species from pool
    let species_idx = rng.gen_range(0..pool.species.len());
    let species_id = &pool.species[species_idx];

    // Enemy level based on floor
    let enemy_level = (floor.min(99)) as u8;

    // Create enemy
    let mut enemy = game_manager.tamer_data.create_monster_at_level(species_id, enemy_level)?;

    // Scale stats based on floor
    let multiplier = floor_stat_multiplier(floor);
    enemy.hp_max = (enemy.hp_max as f32 * multiplier) as u16;
    enemy.hp_current = enemy.hp_max;
    enemy.atk = (enemy.atk as f32 * multiplier) as u16;
    enemy.def = (enemy.def as f32 * multiplier) as u16;

    Some(enemy)
}

/// Generate all wave enemies for a dungeon floor
/// Boss floors (every 5th): 5 waves - 4 normal enemies + 1 boss
/// Normal floors: random 2-5 normal enemies
fn generate_floor_waves(
    game_manager: &GameManager,
    dungeon: &crate::game::core::Dungeon,
    floor: u16,
) -> Option<Vec<crate::game::core::Monster>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut waves = Vec::new();

    let is_boss = dungeon.is_boss_floor(floor);

    if is_boss {
        // Boss floor: 4 normal waves + 1 boss wave
        for _ in 0..4 {
            if let Some(enemy) = generate_regular_enemy(game_manager, dungeon, floor, &mut rng) {
                waves.push(enemy);
            }
        }
        // Add boss as final wave
        if let Some(boss) = generate_boss_enemy(game_manager, dungeon, floor) {
            waves.push(boss);
        }
    } else {
        // Normal floor: random 2-5 waves
        let wave_count = rng.gen_range(2..=5);
        for _ in 0..wave_count {
            if let Some(enemy) = generate_regular_enemy(game_manager, dungeon, floor, &mut rng) {
                waves.push(enemy);
            }
        }
    }

    if waves.is_empty() {
        None
    } else {
        log::info!("Generated {} waves for floor {} (boss: {})", waves.len(), floor, is_boss);
        Some(waves)
    }
}

/// Generate a regular (non-boss) enemy
fn generate_regular_enemy(
    game_manager: &GameManager,
    dungeon: &crate::game::core::Dungeon,
    floor: u16,
    rng: &mut impl rand::Rng,
) -> Option<crate::game::core::Monster> {
    let pool = dungeon.get_enemy_pool(floor)?;
    if pool.species.is_empty() {
        return None;
    }

    let species_idx = rng.gen_range(0..pool.species.len());
    let species_id = &pool.species[species_idx];
    let enemy_level = (floor.min(99)) as u8;

    let mut enemy = game_manager.tamer_data.create_monster_at_level(species_id, enemy_level)?;

    let multiplier = floor_stat_multiplier(floor);
    enemy.hp_max = (enemy.hp_max as f32 * multiplier) as u16;
    enemy.hp_current = enemy.hp_max;
    enemy.atk = (enemy.atk as f32 * multiplier) as u16;
    enemy.def = (enemy.def as f32 * multiplier) as u16;

    Some(enemy)
}

/// Generate a boss enemy
fn generate_boss_enemy(
    game_manager: &GameManager,
    dungeon: &crate::game::core::Dungeon,
    floor: u16,
) -> Option<crate::game::core::Monster> {
    let boss_species = dungeon.get_boss_species(floor)?;
    let boss_level = (floor + 5).min(99) as u8;
    let mut boss = game_manager.tamer_data.create_monster_at_level(boss_species, boss_level)?;

    let multiplier = floor_stat_multiplier(floor);
    boss.hp_max = (boss.hp_max as f32 * multiplier * 1.5) as u16;
    boss.hp_current = boss.hp_max;
    boss.atk = (boss.atk as f32 * multiplier * 1.2) as u16;
    boss.def = (boss.def as f32 * multiplier * 1.2) as u16;

    Some(boss)
}

/// Create expedition team selection page from map ID
pub fn create_expedition_team_page(
    game_manager: &GameManager,
    map_id: &str,
) -> Option<ExpeditionTeamSelectPage> {
    let tamer_map = game_manager.tamer_data.get_tamer_map(map_id)?;

    let monster_data: Vec<MonsterSelectData> = game_manager.monsters.iter()
        .map(|m| MonsterSelectData {
            id: m.id.clone(),
            name: m.name.clone(),
            level: m.level,
            element: m.element,
            is_available: m.status == MonsterStatus::Available,
            is_selected: false,
        })
        .collect();

    Some(ExpeditionTeamSelectPage::new(
        tamer_map.id.clone(),
        tamer_map.name.clone(),
        tamer_map.required_elements.clone(),
        monster_data,
    ))
}
