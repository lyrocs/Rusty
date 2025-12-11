//! Dungeon Navigation System
//!
//! Handles dungeon combat flow: combat -> between floors -> next combat or exit.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents, SdCardWrapper};
use crate::game::systems::combat::CombatState;
use crate::game::systems::dungeon::{DungeonRun, floor_stat_multiplier};
use crate::game::systems::progression::leveling::apply_xp_to_monster;
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{BetweenFloorsPage, BetweenFloorsAction, MonsterStatusData, DungeonCombatPage, DungeonDefeatPage, DungeonDefeatAction};

/// System to handle dungeon combat navigation
pub fn dungeon_combat_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    mut sd_card_res: Option<NonSendMut<SdCardWrapper>>,
) {
    // Only process in DungeonCombat mode
    if app_state.current_mode != AppMode::DungeonCombat {
        return;
    }

    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check if animations need loading from SD card
    if let (Some(ref mut combat_page), Some(ref mut sd_card)) =
        (&mut game_manager.dungeon_combat_page, &mut sd_card_res)
    {
        // First check: initial loading (shows "Loading..." screen first)
        if combat_page.needs_initial_load() {
            combat_page.load_initial_animations(sd_card);
        }
        // Load pending animations (attack, hurt, death queued by combat events)
        if combat_page.has_pending_animations() {
            combat_page.load_pending_animations(sd_card);
        }
        // Reload if enemy species changed (new wave)
        if combat_page.needs_enemy_reload() {
            combat_page.reload_enemy_species(sd_card);
        }
        // Reload current frames for streaming playback (only when frames advance)
        if combat_page.needs_frame_reload() {
            combat_page.reload_needed_frames(sd_card);
        }
    }

    // Check if combat ended
    let combat_ended = game_manager.dungeon_combat_page
        .as_ref()
        .map(|p| p.combat_result().is_some())
        .unwrap_or(false);

    if !combat_ended {
        // Handle touch events for combat
        for event in pending_events.events.iter() {
            if let InputEvent::Touch { x, y } = event {
                if let Some(ref mut combat_page) = game_manager.dungeon_combat_page {
                    combat_page.handle_touch(*x as i32, *y as i32);
                }
            }
        }
        return;
    }

    // Combat ended - get results
    let (victory, crystals, xp) = game_manager.dungeon_combat_page
        .as_ref()
        .and_then(|p| p.combat_result())
        .unwrap_or((false, 0, 0));

    // Get team monsters from combat state
    let team_status: Vec<MonsterStatusData> = game_manager.dungeon_combat_page
        .as_ref()
        .map(|p| {
            p.combat_state().player_monsters.iter().map(|m| MonsterStatusData {
                name: m.name.clone(),
                element: m.element,
                level: m.level,
                hp_current: m.hp_current,
                hp_max: m.hp_max,
                is_alive: m.is_alive(),
            }).collect()
        })
        .unwrap_or_default();

    // Capture skill bar progress for persistence between floors
    let skill_bar_progress = game_manager.dungeon_combat_page
        .as_ref()
        .map(|p| p.combat_state().player_skl_bar)
        .unwrap_or(0.0);

    if victory {
        // Player won - transition to between floors
        let current_floor = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.current_floor)
            .unwrap_or(1);

        let dungeon_name = game_manager.selected_dungeon_id
            .as_ref()
            .and_then(|id| game_manager.tamer_data.get_dungeon(id))
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Unknown Dungeon".to_string());

        // Update dungeon run and save skill bar progress
        if let Some(ref mut run) = game_manager.active_dungeon_run {
            run.advance_floor(crystals, xp);
            run.persistent_skill_bar = skill_bar_progress;
        }

        let floors_cleared = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.floors_cleared())
            .unwrap_or(0);

        let total_crystals = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.crystals_earned)
            .unwrap_or(crystals);

        let total_xp = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.xp_earned)
            .unwrap_or(xp);

        // Create between floors page
        game_manager.between_floors_page = Some(BetweenFloorsPage::new(
            dungeon_name,
            current_floor,
            floors_cleared,
            total_crystals,
            total_xp,
            team_status,
        ));

        // Clean up combat page
        game_manager.dungeon_combat_page = None;

        // Transition to between floors
        app_state.current_mode = AppMode::BetweenFloors;
        app_state.needs_redraw = true;

        log::info!("Combat victory! Floor {} cleared. Total: +{} crystals, +{} XP",
            current_floor, total_crystals, total_xp);
    } else {
        // Player lost - show defeat page
        let current_floor = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.current_floor)
            .unwrap_or(1);

        let dungeon_id = game_manager.selected_dungeon_id
            .clone()
            .unwrap_or_default();

        let dungeon_name = game_manager.tamer_data.get_dungeon(&dungeon_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Unknown Dungeon".to_string());

        let total_crystals = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.crystals_earned)
            .unwrap_or(0);

        let total_xp = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.xp_earned)
            .unwrap_or(0);

        let previous_record = game_manager.dungeon_progress
            .get(&dungeon_id)
            .copied()
            .unwrap_or(0);

        // Find last checkpoint (highest checkpoint <= current floor)
        let last_checkpoint = game_manager.tamer_data.get_dungeon(&dungeon_id)
            .map(|d| {
                d.checkpoints.iter()
                    .filter(|&&cp| cp <= current_floor)
                    .max()
                    .copied()
                    .unwrap_or(1)
            })
            .unwrap_or(1);

        log::info!("Dungeon run ended: defeat on floor {}. Rewards: +{} crystals, +{} XP",
            current_floor, total_crystals, total_xp);

        // Create defeat page
        game_manager.dungeon_defeat_page = Some(DungeonDefeatPage::new(
            dungeon_id,
            dungeon_name,
            current_floor,
            total_crystals,
            total_xp,
            previous_record,
            last_checkpoint,
        ));

        // Clean up combat page but keep dungeon run data for potential retry
        game_manager.dungeon_combat_page = None;

        // Transition to defeat screen
        app_state.current_mode = AppMode::DungeonDefeat;
        app_state.needs_redraw = true;
    }
}

/// System to handle between floors navigation
pub fn between_floors_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    mut sd_card_res: Option<NonSendMut<SdCardWrapper>>,
) {
    // Only process in BetweenFloors mode
    if app_state.current_mode != AppMode::BetweenFloors {
        return;
    }

    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let action = game_manager.between_floors_page
                    .as_ref()
                    .map(|p| p.handle_touch(*x as i32, *y as i32))
                    .unwrap_or(BetweenFloorsAction::None);

                match action {
                    BetweenFloorsAction::Continue => {
                        // Continue to next floor
                        if let Some(dungeon_id) = &game_manager.selected_dungeon_id.clone() {
                            let next_floor = game_manager.active_dungeon_run
                                .as_ref()
                                .map(|r| r.current_floor)
                                .unwrap_or(1);

                            // Get team status from between floors page for HP tracking
                            let team_hp: Vec<(u16, u16)> = game_manager.between_floors_page
                                .as_ref()
                                .map(|p| p.team_status.iter().map(|m| (m.hp_current, m.hp_max)).collect())
                                .unwrap_or_default();

                            // Create next combat (starts in loading state)
                            if let Some((combat_page, _)) = create_next_floor_combat(
                                game_manager,
                                dungeon_id,
                                next_floor,
                                &team_hp,
                            ) {
                                game_manager.dungeon_combat_page = Some(combat_page);
                                game_manager.between_floors_page = None;
                                app_state.current_mode = AppMode::DungeonCombat;
                                app_state.needs_redraw = true;
                                log::info!("Continuing to floor {}", next_floor);
                            } else {
                                log::warn!("Failed to create combat for floor {}", next_floor);
                                end_dungeon_run(game_manager);
                                app_state.current_mode = AppMode::Home;
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                    BetweenFloorsAction::Abandon => {
                        // Abandon run - keep rewards
                        log::info!("Dungeon run abandoned by player");
                        end_dungeon_run(game_manager);
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    }
                    BetweenFloorsAction::None => {}
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe right to abandon
                if *direction == SwipeDirection::Right {
                    log::info!("Dungeon run abandoned (swipe)");
                    end_dungeon_run(game_manager);
                    app_state.current_mode = AppMode::Home;
                    app_state.needs_redraw = true;
                }
            }
            _ => {}
        }
    }
}

/// System to handle dungeon defeat page navigation
pub fn dungeon_defeat_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    mut sd_card_res: Option<NonSendMut<SdCardWrapper>>,
) {
    // Only process in DungeonDefeat mode
    if app_state.current_mode != AppMode::DungeonDefeat {
        return;
    }

    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let action = game_manager.dungeon_defeat_page
                    .as_ref()
                    .map(|p| p.handle_touch(*x as i32, *y as i32))
                    .unwrap_or(DungeonDefeatAction::None);

                match action {
                    DungeonDefeatAction::Retry => {
                        // Get retry info from defeat page before cleaning it up
                        let (dungeon_id, checkpoint, floor_reached) = game_manager.dungeon_defeat_page
                            .as_ref()
                            .map(|p| (p.dungeon_id().to_string(), p.retry_checkpoint(), p.floor_reached()))
                            .unwrap_or_default();

                        // Apply rewards from this run before retrying
                        let crystals = game_manager.dungeon_defeat_page
                            .as_ref()
                            .map(|p| p.crystals_earned())
                            .unwrap_or(0);
                        let xp_earned = game_manager.dungeon_defeat_page
                            .as_ref()
                            .map(|p| p.xp_earned())
                            .unwrap_or(0);

                        game_manager.player.crystals += crystals;

                        // Apply XP to team monsters before retrying
                        let team_ids = game_manager.team.monster_ids().to_vec();
                        for monster_id in team_ids {
                            if let Some(monster) = game_manager.monsters.iter_mut()
                                .find(|m| m.id == monster_id)
                            {
                                let levels_gained = apply_xp_to_monster(monster, xp_earned);
                                if levels_gained > 0 {
                                    log::info!("{} gained {} XP and leveled up to level {}!",
                                        monster.name, xp_earned, monster.level);
                                }
                            }
                        }

                        // Update record if we beat our previous best
                        let is_new_record = game_manager.dungeon_defeat_page
                            .as_ref()
                            .map(|p| p.is_new_record())
                            .unwrap_or(false);

                        if is_new_record {
                            game_manager.dungeon_progress.insert(dungeon_id.clone(), floor_reached);
                        }

                        log::info!("Retrying dungeon {} from checkpoint floor {}", dungeon_id, checkpoint);

                        // Clean up defeat page
                        game_manager.dungeon_defeat_page = None;
                        game_manager.active_dungeon_run = None;

                        // Create new dungeon run from checkpoint
                        if let Some(dungeon) = game_manager.tamer_data.get_dungeon(&dungeon_id).cloned() {
                            // Create a new dungeon run starting from checkpoint
                            game_manager.active_dungeon_run = Some(DungeonRun::new(dungeon_id.clone(), checkpoint));
                            game_manager.selected_dungeon_id = Some(dungeon_id.clone());

                            // Get player team with full HP (retry = fresh start)
                            let team_ids = game_manager.team.monster_ids().to_vec();
                            let mut player_monsters: Vec<crate::game::core::Monster> = Vec::new();

                            for monster_id in team_ids.iter() {
                                if let Some(monster) = game_manager.get_monster_mut(monster_id) {
                                    // Allow monsters in expedition to also run dungeons
                                    let mut combat_monster = monster.clone();
                                    // Full HP on retry
                                    combat_monster.hp_current = combat_monster.hp_max;
                                    player_monsters.push(combat_monster);
                                }
                            }

                            if !player_monsters.is_empty() {
                                // Generate wave enemies for checkpoint floor
                                if let Some(wave_enemies) = generate_floor_waves(&*game_manager, &dungeon, checkpoint) {
                                    let combat_state = CombatState::with_waves(player_monsters, wave_enemies, checkpoint);
                                    game_manager.dungeon_combat_page = Some(DungeonCombatPage::new(combat_state, dungeon.name.clone()));
                                    app_state.current_mode = AppMode::DungeonCombat;
                                    app_state.needs_redraw = true;
                                } else {
                                    log::warn!("Failed to generate enemies for retry");
                                    app_state.current_mode = AppMode::Home;
                                    app_state.needs_redraw = true;
                                }
                            } else {
                                log::warn!("No available monsters for retry");
                                app_state.current_mode = AppMode::Home;
                                app_state.needs_redraw = true;
                            }
                        } else {
                            log::warn!("Dungeon not found for retry: {}", dungeon_id);
                            app_state.current_mode = AppMode::Home;
                            app_state.needs_redraw = true;
                        }
                    }
                    DungeonDefeatAction::Quit => {
                        log::info!("Player quit dungeon");
                        // Apply rewards before quitting
                        end_dungeon_run(game_manager);
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    }
                    DungeonDefeatAction::None => {}
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe right to quit
                if *direction == SwipeDirection::Right {
                    log::info!("Player quit dungeon (swipe)");
                    end_dungeon_run(game_manager);
                    app_state.current_mode = AppMode::Home;
                    app_state.needs_redraw = true;
                }
            }
            _ => {}
        }
    }
}

/// End dungeon run and apply rewards
fn end_dungeon_run(game_manager: &mut GameManager) {
    if let Some(ref run) = game_manager.active_dungeon_run {
        // Apply crystal rewards to player
        game_manager.player.crystals += run.crystals_earned;

        // Apply XP rewards to team monsters
        let xp_earned = run.xp_earned;
        let team_ids = game_manager.team.monster_ids().to_vec();

        for monster_id in team_ids {
            if let Some(monster) = game_manager.monsters.iter_mut()
                .find(|m| m.id == monster_id)
            {
                let levels_gained = apply_xp_to_monster(monster, xp_earned);
                if levels_gained > 0 {
                    log::info!("{} gained {} XP and leveled up {} times to level {}!",
                        monster.name, xp_earned, levels_gained, monster.level);
                } else {
                    log::info!("{} gained {} XP ({}/{})",
                        monster.name, xp_earned, monster.xp, monster.xp_to_next);
                }
            }
        }

        // Update dungeon progress if we set a new record
        let dungeon_id = run.dungeon_id.clone();
        let highest_floor = run.current_floor;
        let current_record = game_manager.dungeon_progress.get(&dungeon_id).copied().unwrap_or(0);
        if highest_floor > current_record {
            game_manager.dungeon_progress.insert(dungeon_id.clone(), highest_floor);
            log::info!("New dungeon record! {} floor {}", dungeon_id, highest_floor);
        }

        log::info!("Dungeon run ended: +{} crystals, +{} XP, floor {} reached",
            run.crystals_earned, xp_earned, run.current_floor);
    }

    // Clean up
    game_manager.active_dungeon_run = None;
    game_manager.dungeon_combat_page = None;
    game_manager.between_floors_page = None;
    game_manager.dungeon_defeat_page = None;
    game_manager.selected_dungeon_id = None;
}

/// Create combat for next floor, preserving team HP and skill bar
fn create_next_floor_combat(
    game_manager: &mut GameManager,
    dungeon_id: &str,
    floor: u16,
    team_hp: &[(u16, u16)],
) -> Option<(DungeonCombatPage, ())> {
    use rand::Rng;

    // Clone dungeon data
    let dungeon = game_manager.tamer_data.get_dungeon(dungeon_id)?.clone();
    let dungeon_name = dungeon.name.clone();

    // Get persistent skill bar from dungeon run
    let persistent_skill_bar = game_manager.active_dungeon_run
        .as_ref()
        .map(|r| r.persistent_skill_bar)
        .unwrap_or(0.0);

    // Get player's team monsters
    let team_ids = game_manager.team.monster_ids().to_vec();
    let mut player_monsters: Vec<crate::game::core::Monster> = Vec::new();

    for (i, monster_id) in team_ids.iter().enumerate() {
        if let Some(monster) = game_manager.get_monster_mut(monster_id) {
            // Allow monsters in expedition to also run dungeons
            let mut combat_monster = monster.clone();
            // Restore HP from previous combat if available
            if i < team_hp.len() {
                combat_monster.hp_current = team_hp[i].0;
                combat_monster.hp_max = team_hp[i].1;
            }
            player_monsters.push(combat_monster);
        }
    }

    // Only include alive monsters
    player_monsters.retain(|m| m.is_alive());

    if player_monsters.is_empty() {
        log::warn!("No alive monsters for next floor");
        return None;
    }

    // Generate wave enemies for this floor
    let wave_enemies = generate_floor_waves(game_manager, &dungeon, floor)?;

    log::info!("Next floor combat: {} monsters vs {} waves on floor {}, skill bar: {:.0}%",
        player_monsters.len(), wave_enemies.len(), floor, persistent_skill_bar * 100.0);

    // Create combat state with waves and preserved skill bar
    let combat_state = CombatState::with_waves_and_skill_bar(
        player_monsters,
        wave_enemies,
        floor,
        persistent_skill_bar,
    );

    // Create combat page (starts in loading state, animations loaded by navigation system)
    let combat_page = DungeonCombatPage::new(combat_state, dungeon_name);

    Some((combat_page, ()))
}

/// Generate enemy for a dungeon floor (single enemy - for backwards compatibility)
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
            let boss_level = (floor + 5).min(99) as u8;
            if let Some(mut boss) = game_manager.tamer_data.create_monster_at_level(boss_species, boss_level) {
                let multiplier = floor_stat_multiplier(floor);
                boss.hp_max = (boss.hp_max as f32 * multiplier * 1.5) as u16;
                boss.hp_current = boss.hp_max;
                boss.atk = (boss.atk as f32 * multiplier * 1.2) as u16;
                boss.def = (boss.def as f32 * multiplier * 1.2) as u16;
                return Some(boss);
            }
        }
    }

    // Regular enemy
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
