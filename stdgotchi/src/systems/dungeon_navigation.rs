//! Dungeon Navigation System
//!
//! Handles dungeon combat flow: combat -> between floors -> next combat or exit.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents, SdCardWrapper};
use crate::game::systems::combat::CombatState;
use crate::game::systems::dungeon::{DungeonRun, floor_stat_multiplier};
use crate::game::systems::progression::leveling::apply_xp_to_monster;
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{BetweenFloorsPage, BetweenFloorsAction, MonsterStatusData, DungeonCombatPage, DungeonDefeatPage, DungeonDefeatAction, DungeonListAction, DungeonInfoPage, DungeonInfoAction, MonsterDisplayInfo};

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
        // Reload if player species changed (swap to uncached monster)
        if combat_page.needs_player_reload() {
            combat_page.reload_player_species(sd_card);
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
                xp_current: m.xp,
                xp_to_next: m.xp_to_next,
                xp_gained: 0, // Will be updated after combat
            }).collect()
        })
        .unwrap_or_default();

    // Capture skill bar progress for persistence between floors
    let skill_bar_progress = game_manager.dungeon_combat_page
        .as_ref()
        .map(|p| p.combat_state().player_skl_bar)
        .unwrap_or(0.0);

    if victory {
        // Player won
        let current_floor = game_manager.active_dungeon_run
            .as_ref()
            .map(|r| r.current_floor)
            .unwrap_or(1);

        let dungeon_name = game_manager.selected_dungeon_id
            .as_ref()
            .and_then(|id| game_manager.tamer_data.get_dungeon(id))
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "Unknown Dungeon".to_string());

        // Check if this was a boss floor
        let is_boss_floor = game_manager.dungeon_combat_page
            .as_ref()
            .map(|p| p.combat_state().is_boss_floor)
            .unwrap_or(false);

        // Update dungeon run and save skill bar progress
        if let Some(ref mut run) = game_manager.active_dungeon_run {
            run.advance_floor(crystals, xp);
            run.persistent_skill_bar = skill_bar_progress;
        }

        // Apply XP to team monsters
        let team_ids = game_manager.team.monster_ids().to_vec();
        let alive_monsters: Vec<_> = team_ids.iter()
            .filter(|id| game_manager.get_monster(id).map(|m| m.is_alive()).unwrap_or(false))
            .cloned()
            .collect();
        let alive_count = alive_monsters.len();
        let xp_per_monster = if alive_count > 0 { xp / alive_count as u32 } else { 0 };

        for monster_id in &alive_monsters {
            if let Some(monster) = game_manager.get_monster_mut(monster_id) {
                let levels_gained = apply_xp_to_monster(monster, xp_per_monster);
                if levels_gained > 0 {
                    log::info!("{} gained {} XP and leveled up {} times to Lv.{}!",
                        monster.name, xp_per_monster, levels_gained, monster.level);
                } else {
                    log::info!("{} gained {} XP ({}/{})",
                        monster.name, xp_per_monster, monster.xp, monster.xp_to_next);
                }
            }
        }

        // Clean up combat page
        game_manager.dungeon_combat_page = None;

        if is_boss_floor {
            // Boss floor: Full heal team and go to BetweenFloors (skip bonus selection)
            log::info!("Boss floor {} cleared! Full heal applied", current_floor);

            // Full heal all team monsters
            let team_ids = game_manager.team.monster_ids().to_vec();
            for monster_id in &team_ids {
                if let Some(monster) = game_manager.get_monster_mut(monster_id) {
                    monster.hp_current = monster.hp_max;
                }
            }

            // Calculate XP per monster (divide among alive monsters)
            let alive_count = team_ids.iter()
                .filter_map(|id| game_manager.get_monster(id))
                .filter(|m| m.is_alive())
                .count();
            let xp_per_monster = if alive_count > 0 { xp / alive_count as u32 } else { 0 };

            // Get updated team status after heal
            let team_status_healed: Vec<MonsterStatusData> = team_ids.iter()
                .filter_map(|id| game_manager.get_monster(id))
                .map(|m| MonsterStatusData {
                    name: m.name.clone(),
                    level: m.level,
                    hp_current: m.hp_current,
                    hp_max: m.hp_max,
                    element: m.element,
                    is_alive: m.is_alive(),
                    xp_current: m.xp,
                    xp_to_next: m.xp_to_next,
                    xp_gained: if m.is_alive() { xp_per_monster } else { 0 },
                })
                .collect();

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

            let active_bonuses = game_manager.active_dungeon_run
                .as_ref()
                .map(|r| r.active_bonuses.clone())
                .unwrap_or_default();

            // Create between floors page
            game_manager.between_floors_page = Some(BetweenFloorsPage::new(
                dungeon_name,
                current_floor,
                floors_cleared,
                total_crystals,
                total_xp,
                team_status_healed,
                active_bonuses,
            ));

            app_state.current_mode = AppMode::BetweenFloors;
            app_state.needs_redraw = true;
        } else {
            // Non-boss floor: Go directly to BetweenFloors
            log::info!("Floor {} cleared! Floor XP: {}", current_floor, xp);

            // Get team status for BetweenFloors page
            let team_ids = game_manager.team.monster_ids().to_vec();
            let alive_count = team_ids.iter()
                .filter_map(|id| game_manager.get_monster(id))
                .filter(|m| m.is_alive())
                .count();
            let xp_per_monster = if alive_count > 0 { xp / alive_count as u32 } else { 0 };

            let team_status: Vec<MonsterStatusData> = team_ids.iter()
                .filter_map(|id| game_manager.get_monster(id))
                .map(|m| MonsterStatusData {
                    name: m.name.clone(),
                    level: m.level,
                    hp_current: m.hp_current,
                    hp_max: m.hp_max,
                    element: m.element,
                    is_alive: m.is_alive(),
                    xp_current: m.xp,
                    xp_to_next: m.xp_to_next,
                    xp_gained: if m.is_alive() { xp_per_monster } else { 0 },
                })
                .collect();

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

            let active_bonuses = game_manager.active_dungeon_run
                .as_ref()
                .map(|r| r.active_bonuses.clone())
                .unwrap_or_default();

            // Create between floors page
            game_manager.between_floors_page = Some(BetweenFloorsPage::new(
                dungeon_name,
                current_floor,
                floors_cleared,
                total_crystals,
                total_xp,
                team_status,
                active_bonuses,
            ));

            app_state.current_mode = AppMode::BetweenFloors;
            app_state.needs_redraw = true;
        }
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

                        // Apply crystal rewards from this run before retrying
                        // (XP was already applied per floor during combat victories)
                        let crystals = game_manager.dungeon_defeat_page
                            .as_ref()
                            .map(|p| p.crystals_earned())
                            .unwrap_or(0);

                        game_manager.player.crystals += crystals;

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
                                // Generate enemy for checkpoint floor
                                if let Some(enemy) = generate_floor_enemy(&*game_manager, &dungeon, checkpoint) {
                                    let is_boss = dungeon.is_boss_floor(checkpoint);
                                    let combat_state = CombatState::for_floor(player_monsters, enemy, checkpoint, is_boss);
                                    game_manager.dungeon_combat_page = Some(DungeonCombatPage::new(combat_state, dungeon.name.clone()));
                                    app_state.current_mode = AppMode::DungeonCombat;
                                    app_state.needs_redraw = true;
                                } else {
                                    log::warn!("Failed to generate enemy for retry");
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
/// Note: XP is now applied per floor during combat, so we only apply crystals here
fn end_dungeon_run(game_manager: &mut GameManager) {
    if let Some(ref run) = game_manager.active_dungeon_run {
        // Apply crystal rewards to player
        game_manager.player.crystals += run.crystals_earned;

        // XP was already applied per floor during combat victories
        // No need to apply again here

        // Update dungeon progress if we set a new record
        let dungeon_id = run.dungeon_id.clone();
        let highest_floor = run.current_floor;
        let current_record = game_manager.dungeon_progress.get(&dungeon_id).copied().unwrap_or(0);
        if highest_floor > current_record {
            game_manager.dungeon_progress.insert(dungeon_id.clone(), highest_floor);
            log::info!("New dungeon record! {} floor {}", dungeon_id, highest_floor);
        }

        log::info!("Dungeon run ended: +{} crystals, +{} XP total, floor {} reached",
            run.crystals_earned, run.xp_earned, run.current_floor);
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

    // Get stat boosts from active dungeon run
    use crate::game::core::bonus::StatBoostType;
    let (atk_boost, def_boost, spd_boost) = game_manager.active_dungeon_run
        .as_ref()
        .map(|run| (
            run.get_stat_boost(StatBoostType::Atk),
            run.get_stat_boost(StatBoostType::Def),
            run.get_stat_boost(StatBoostType::Spd),
        ))
        .unwrap_or((0.0, 0.0, 0.0));

    let has_boosts = atk_boost > 0.0 || def_boost > 0.0 || spd_boost > 0.0;
    if has_boosts {
        log::info!("Applying stat boosts: ATK +{}%, DEF +{}%, SPD +{}%",
            (atk_boost * 100.0) as u8, (def_boost * 100.0) as u8, (spd_boost * 100.0) as u8);
    }

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
            // Apply stat boosts from active bonuses
            if atk_boost > 0.0 {
                combat_monster.atk = (combat_monster.atk as f32 * (1.0 + atk_boost)) as u16;
            }
            if def_boost > 0.0 {
                combat_monster.def = (combat_monster.def as f32 * (1.0 + def_boost)) as u16;
            }
            if spd_boost > 0.0 {
                combat_monster.spd = (combat_monster.spd as f32 * (1.0 + spd_boost)) as u16;
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

    // Generate enemy for this floor
    let enemy = generate_floor_enemy(game_manager, &dungeon, floor)?;
    let is_boss = dungeon.is_boss_floor(floor);

    log::info!("Next floor combat: {} monsters vs enemy on floor {} (boss: {})",
        player_monsters.len(), floor, is_boss);

    // Create combat state for floor
    let combat_state = CombatState::for_floor(
        player_monsters,
        enemy,
        floor,
        is_boss,
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

/// System to handle dungeon list navigation
pub fn dungeon_list_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in DungeonList mode
    if app_state.current_mode != AppMode::DungeonList {
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
                let action = game_manager.dungeon_list_page
                    .as_ref()
                    .map(|p| p.handle_touch(*x as i32, *y as i32))
                    .unwrap_or(DungeonListAction::None);

                match action {
                    DungeonListAction::Back => {
                        log::info!("DungeonList -> Home");
                        game_manager.dungeon_list_page = None;
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    }
                    DungeonListAction::SelectDungeon(dungeon_id) => {
                        log::info!("Selected dungeon: {}", dungeon_id);
                        // Create dungeon info page
                        if let Some(info_page) = create_dungeon_info_page(game_manager, &dungeon_id) {
                            game_manager.dungeon_info_page = Some(info_page);
                            game_manager.selected_dungeon_id = Some(dungeon_id);
                            app_state.current_mode = AppMode::DungeonInfo;
                            app_state.needs_redraw = true;
                        }
                    }
                    DungeonListAction::None => {}
                }
            }
            InputEvent::Swipe { direction } => {
                match direction {
                    SwipeDirection::Right => {
                        // Swipe right -> back to home
                        log::info!("Swipe right: DungeonList -> Home");
                        game_manager.dungeon_list_page = None;
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    }
                    SwipeDirection::Up | SwipeDirection::Down => {
                        // Handle scrolling
                        if let Some(ref mut page) = game_manager.dungeon_list_page {
                            page.handle_swipe(*direction == SwipeDirection::Up);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// System to handle dungeon info navigation
pub fn dungeon_info_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in DungeonInfo mode
    if app_state.current_mode != AppMode::DungeonInfo {
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
                let action = game_manager.dungeon_info_page
                    .as_mut()
                    .map(|p| p.handle_touch(*x as i32, *y as i32))
                    .unwrap_or(DungeonInfoAction::None);

                match action {
                    DungeonInfoAction::Back => {
                        log::info!("DungeonInfo -> DungeonList");
                        game_manager.dungeon_info_page = None;
                        app_state.current_mode = AppMode::DungeonList;
                        app_state.needs_redraw = true;
                    }
                    DungeonInfoAction::StartDungeon { checkpoint } => {
                        log::info!("Starting dungeon from checkpoint floor {}", checkpoint);
                        // Start dungeon run
                        if let Some(dungeon_id) = game_manager.selected_dungeon_id.clone() {
                            if let Some(dungeon) = game_manager.tamer_data.get_dungeon(&dungeon_id).cloned() {
                                // Create dungeon run
                                game_manager.active_dungeon_run = Some(DungeonRun::new(dungeon_id.clone(), checkpoint));

                                // Get player team with full HP
                                let team_ids = game_manager.team.monster_ids().to_vec();
                                let mut player_monsters: Vec<crate::game::core::Monster> = Vec::new();

                                for monster_id in team_ids.iter() {
                                    if let Some(monster) = game_manager.get_monster_mut(monster_id) {
                                        let mut combat_monster = monster.clone();
                                        // Full HP on start
                                        combat_monster.hp_current = combat_monster.hp_max;
                                        player_monsters.push(combat_monster);
                                    }
                                }

                                if !player_monsters.is_empty() {
                                    // Generate enemy for checkpoint floor
                                    if let Some(enemy) = generate_floor_enemy(&*game_manager, &dungeon, checkpoint) {
                                        let is_boss = dungeon.is_boss_floor(checkpoint);
                                        let combat_state = CombatState::for_floor(player_monsters, enemy, checkpoint, is_boss);
                                        game_manager.dungeon_combat_page = Some(DungeonCombatPage::new(combat_state, dungeon.name.clone()));
                                        game_manager.dungeon_info_page = None;
                                        game_manager.dungeon_list_page = None;
                                        app_state.current_mode = AppMode::DungeonCombat;
                                        app_state.needs_redraw = true;
                                    } else {
                                        log::warn!("Failed to generate enemy for floor {}", checkpoint);
                                    }
                                } else {
                                    log::warn!("No available monsters for dungeon");
                                }
                            }
                        }
                    }
                    DungeonInfoAction::None => {}
                }
            }
            InputEvent::Swipe { direction } => {
                match direction {
                    SwipeDirection::Right => {
                        // Swipe right -> back to dungeon list
                        log::info!("Swipe right: DungeonInfo -> DungeonList");
                        game_manager.dungeon_info_page = None;
                        app_state.current_mode = AppMode::DungeonList;
                        app_state.needs_redraw = true;
                    }
                    SwipeDirection::Up | SwipeDirection::Down => {
                        // Handle scrolling monster list
                        if let Some(ref mut page) = game_manager.dungeon_info_page {
                            page.handle_swipe(*direction == SwipeDirection::Up);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Create dungeon info page from dungeon ID
fn create_dungeon_info_page(game_manager: &GameManager, dungeon_id: &str) -> Option<DungeonInfoPage> {
    let dungeon = game_manager.tamer_data.get_dungeon(dungeon_id)?;

    // Get monsters from all enemy pools
    let mut monsters: Vec<MonsterDisplayInfo> = Vec::new();
    let mut seen_species: std::collections::HashSet<String> = std::collections::HashSet::new();

    for pool in &dungeon.enemy_pools {
        for species_id in &pool.species {
            if seen_species.contains(species_id) {
                continue;
            }
            seen_species.insert(species_id.clone());

            if let Some(species) = game_manager.tamer_data.species.get(species_id) {
                monsters.push(MonsterDisplayInfo {
                    name: species.name.clone(),
                    element: species.element,
                    is_boss: false,
                });
            }
        }
    }

    // Get boss names
    let mut boss_names: Vec<String> = Vec::new();
    for (_, boss_species_id) in &dungeon.bosses {
        if let Some(species) = game_manager.tamer_data.species.get(boss_species_id) {
            boss_names.push(species.name.clone());
            // Also add to monsters list as boss
            if !seen_species.contains(boss_species_id) {
                monsters.push(MonsterDisplayInfo {
                    name: species.name.clone(),
                    element: species.element,
                    is_boss: true,
                });
            }
        }
    }

    // Calculate level range from species base levels
    let level_min = dungeon.enemy_pools.iter()
        .flat_map(|pool| pool.species.iter())
        .filter_map(|species_id| game_manager.tamer_data.species.get(species_id))
        .map(|species| species.base_level)
        .min()
        .unwrap_or(1);

    let level_max = dungeon.enemy_pools.iter()
        .flat_map(|pool| pool.species.iter())
        .filter_map(|species_id| game_manager.tamer_data.species.get(species_id))
        .map(|species| species.base_level)
        .max()
        .unwrap_or(99);

    // Get highest floor reached
    let highest_floor = game_manager.dungeon_progress.get(dungeon_id).copied().unwrap_or(0);

    Some(DungeonInfoPage::new(
        dungeon,
        monsters,
        boss_names,
        level_min,
        level_max,
        highest_floor,
    ))
}
