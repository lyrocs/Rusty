/// Update system - handles game logic updates (farming, SP regen, animations, etc.)

use bevy_ecs::prelude::*;

use crate::ecs::resources::RtcResource;
use crate::core::GameState;
use crate::combat::BattleState;
use crate::tamagotchi::models::{FarmState, GamePage, RestState};
use super::animations::{
    update_hero_animation, update_monster_animation,
};

pub fn tamagotchi_update_system(
    mut rtc_res: NonSendMut<RtcResource>,
    mut game_state: ResMut<GameState>,
) {
    // Get current CPU cycles for precise timing
    let current_cycles = esp_hal::xtensa_lx::timer::get_cycle_count();
    let cycles_elapsed = current_cycles.wrapping_sub(rtc_res.last_cycles);

    // Convert cycles to milliseconds (CPU freq is in MHz, cycles_elapsed is in cycles)
    // delta_ms = (cycles_elapsed / cycles_per_ms) = (cycles_elapsed / (cpu_freq_mhz * 1000))
    let delta_ms = (cycles_elapsed as u64 / (rtc_res.cpu_freq_mhz * 1000)) as u32;

    // Update last cycles for next frame
    rtc_res.last_cycles = current_cycles;

    // Update game time
    game_state.last_update_ms = game_state.last_update_ms.wrapping_add(delta_ms);

    // Update farm touch cooldown
    if game_state.farm_touch_cooldown > 0 {
        game_state.farm_touch_cooldown = game_state.farm_touch_cooldown.saturating_sub(delta_ms);
    }

    // Only update visual elements (FPS, animations) when screen is on
    if game_state.screen_on {
        // Update FPS counter every 2 seconds for less frequent updates
        game_state.frame_count += 1;
        let fps_elapsed = game_state
            .last_update_ms
            .wrapping_sub(game_state.last_fps_update_ms);
        if fps_elapsed >= 2000 {
            // Calculate FPS: frames / seconds
            game_state.fps = (game_state.frame_count * 1000) / fps_elapsed;
            game_state.frame_count = 0;
            game_state.last_fps_update_ms = game_state.last_update_ms;

            // Only redraw for FPS updates on pages where FPS changes matter (not during active gameplay)
            // During battle, we redraw based on game events (circles, timer, etc), not FPS counter
            if game_state.current_page != GamePage::Battle
                || game_state.battle_state != BattleState::Playing
            {
                game_state.needs_redraw = true; // Redraw when FPS updates
            }
        }

        // Update global GIF animation clock every 75ms for synchronized animations
        // 75ms is the GCD of frame durations (75ms for actions, 150ms for idle)
        // This ensures all GIF animations update at the same time, reducing redraws
        let gif_clock_elapsed = game_state
            .last_update_ms
            .wrapping_sub(game_state.gif_animation_last_update_ms);
        if gif_clock_elapsed >= 75 {
            game_state.gif_animation_clock_ms = game_state
                .gif_animation_clock_ms
                .wrapping_add(75);
            game_state.gif_animation_last_update_ms = game_state.last_update_ms;

            // Note: We don't set needs_redraw here - individual animation functions will do that
            // only if they actually change frames
        }
    }

    // Handle farm state transitions and animations
    if game_state.current_page == GamePage::Farm {
        match game_state.farm_state {
            FarmState::Idle => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Ensure animation is reset to Idle when on idle page
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Idle {
                        game_state.monster_animation = MonsterAnimation::Idle;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                }
            }
            FarmState::Fighting => {
                // Update farming progress (ALWAYS runs - game logic)
                let old_percent = (game_state.farm_progress * 100) / game_state.farm_duration_ms;
                game_state.update_farm_progress(delta_ms);
                let new_percent = (game_state.farm_progress * 100) / game_state.farm_duration_ms;
                // Only redraw if progress bar changes by at least 1% AND screen is on
                if new_percent != old_percent && game_state.screen_on {
                    game_state.needs_redraw = true;
                }

                // Only update animations when screen is on
                if game_state.screen_on {
                    use crate::tamagotchi::models::{
                        HeroAnimation, MonsterAnimation,
                    };

                    // Ensure hero is in Idle animation during fighting
                    if game_state.hero_animation != HeroAnimation::Idle
                        && game_state.hero_animation != HeroAnimation::Attacking
                        && game_state.hero_animation != HeroAnimation::Attacked
                    {
                        game_state.hero_animation = HeroAnimation::Idle;
                        game_state.hero_animation_frame = 0;
                        game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }

                    // Hero attacks monster every 4 seconds (trigger both hero attacking + monster attacked)
                    let time_since_last_hero_attack = game_state
                        .last_update_ms
                        .saturating_sub(game_state.last_hero_attack_ms);
                    if time_since_last_hero_attack >= 4000
                        && game_state.hero_animation == HeroAnimation::Idle
                        && game_state.monster_animation == MonsterAnimation::Idle
                    {
                        // Hero attacks!
                        game_state.hero_animation = HeroAnimation::Attacking;
                        game_state.hero_animation_frame = 0;
                        game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.last_hero_attack_ms = game_state.last_update_ms;

                        // Monster gets attacked!
                        game_state.monster_animation = MonsterAnimation::Attacked;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;

                        game_state.needs_redraw = true;
                    }

                    // Monster attacks hero every 6 seconds (trigger both monster attacking + hero attacked)
                    let time_since_last_monster_attack = game_state
                        .last_update_ms
                        .saturating_sub(game_state.last_attack_animation_ms);
                    if time_since_last_monster_attack >= 6000
                        && game_state.monster_animation == MonsterAnimation::Idle
                        && game_state.hero_animation == HeroAnimation::Idle
                    {
                        // Monster attacks!
                        game_state.monster_animation = MonsterAnimation::Attacking;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.last_attack_animation_ms = game_state.last_update_ms;

                        // Hero gets attacked!
                        game_state.hero_animation = HeroAnimation::Attacked;
                        game_state.hero_animation_frame = 0;
                        game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;

                        game_state.needs_redraw = true;
                    }

                    // Update all animations (get monster name from current enemy)
                    if let Some(enemy) = &game_state.current_enemy {
                        let monster_name = enemy.name;
                        update_monster_animation(&mut game_state, delta_ms, monster_name);
                    }
                    update_hero_animation(&mut game_state, delta_ms);
                }
            }
            FarmState::Victory => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Set to dying animation when entering victory
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Dying {
                        game_state.monster_animation = MonsterAnimation::Dying;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                    // Animate dying GIF (get monster name from current enemy)
                    let monster_name = game_state.current_enemy.as_ref().map(|e| e.name);
                    if let Some(name) = monster_name {
                        update_monster_animation(&mut game_state, delta_ms, name);
                    }
                }
            }
            FarmState::Defeat => {
                // No animation for defeat state
            }
        }
    }

    // Update rest progress (only redraw when HP or SP actually changes)
    if game_state.current_page == GamePage::Rest && game_state.rest_state == RestState::Resting {
        let old_sp = game_state.hero.sp;
        let old_hp = game_state.hero.hp;
        game_state.update_rest_progress(delta_ms);
        // Only redraw if HP or SP changed or state changed AND screen is on
        if (game_state.hero.sp != old_sp
            || game_state.hero.hp != old_hp
            || game_state.rest_state != RestState::Resting)
            && game_state.screen_on
        {
            game_state.needs_redraw = true;
        }
    }

    // Update hero animation on Rest page (only when screen is on)
    if game_state.current_page == GamePage::Rest && game_state.screen_on {
        use crate::tamagotchi::models::HeroAnimation;

        // Ensure hero is in Resting animation
        if game_state.hero_animation != HeroAnimation::Resting {
            game_state.hero_animation = HeroAnimation::Resting;
            game_state.hero_animation_frame = 0;
            game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            game_state.needs_redraw = true;
        }

        // Update resting animation
        update_hero_animation(&mut game_state, delta_ms);
    }

    // Update battle progress (spawn circles, check expiration, handle damage)
    if game_state.current_page == GamePage::Battle {
        match game_state.battle_state {
            BattleState::Idle => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Ensure animation is reset to Idle when on idle state
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Idle {
                        game_state.monster_animation = MonsterAnimation::Idle;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                }
            }
            BattleState::Playing => {
                // Update battle mechanics
                let old_score = game_state.battle_score;
                let old_missed = game_state.battle_missed;
                let old_state = game_state.battle_state;
                let old_time_sec = (game_state.battle_duration - game_state.battle_elapsed) / 1000;

                game_state.update_battle(delta_ms);

                let new_time_sec = (game_state.battle_duration - game_state.battle_elapsed) / 1000;

                // Redraw if score/missed/timer/state changed AND screen is on
                if (game_state.battle_score != old_score
                    || game_state.battle_missed != old_missed
                    || game_state.battle_state != old_state
                    || new_time_sec != old_time_sec)
                    && game_state.screen_on
                {
                    game_state.needs_redraw = true;
                }

                // Only update animation phases when screen is on
                if game_state.screen_on {
                    // Battle animation phase cycling
                    // Sequence: BothIdle (2s) -> MonsterAttacking (1s) -> BothIdle (2s) -> HeroAttacking (1s) -> repeat
                    use crate::tamagotchi::models::BattleAnimationPhase;
                    let time_in_phase = game_state
                        .last_update_ms
                        .saturating_sub(game_state.battle_animation_phase_started_ms);

                    let phase_changed = match game_state.battle_animation_phase {
                        BattleAnimationPhase::BothIdle => {
                            if time_in_phase >= 2000 {
                                // Alternate between monster attacking and hero attacking
                                // Use frame count to alternate
                                if (game_state.battle_elapsed / 6000) % 2 == 0 {
                                    game_state.battle_animation_phase =
                                        BattleAnimationPhase::MonsterAttacking;
                                } else {
                                    game_state.battle_animation_phase =
                                        BattleAnimationPhase::HeroAttacking;
                                }
                                game_state.battle_animation_phase_started_ms =
                                    game_state.last_update_ms;
                                true
                            } else {
                                false
                            }
                        }
                        BattleAnimationPhase::MonsterAttacking
                        | BattleAnimationPhase::HeroAttacking => {
                            if time_in_phase >= 1000 {
                                game_state.battle_animation_phase = BattleAnimationPhase::BothIdle;
                                game_state.battle_animation_phase_started_ms =
                                    game_state.last_update_ms;
                                true
                            } else {
                                false
                            }
                        }
                    };

                    // Animation phases and updates disabled during battle for performance
                    // GIFs are not rendered during manual battle gameplay anyway
                    if phase_changed {
                        // Don't set needs_redraw for animation phase changes since we don't render GIFs
                        // game_state.needs_redraw = true;
                    }

                    // Don't update animations during battle - GIFs are not rendered for performance
                    // let monster_name = game_state.battle_enemy.as_ref().map(|e| e.name);
                    // if let Some(name) = monster_name {
                    //     update_battle_animations(&mut game_state, delta_ms, name);
                    // }
                }
            }
            BattleState::Victory => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Set to dying animation when entering victory
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Dying {
                        game_state.monster_animation = MonsterAnimation::Dying;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                    // Animate dying GIF (get monster name from battle enemy)
                    let monster_name = game_state.battle_enemy.as_ref().map(|e| e.name);
                    if let Some(name) = monster_name {
                        update_monster_animation(&mut game_state, delta_ms, name);
                    }
                }
            }
            BattleState::Defeat => {
                // No animation for defeat state, keep it idle or stopped
            }
        }
    }

    // Handle JRPG battle updates
    if game_state.current_page == GamePage::JrpgBattle {
        use crate::tamagotchi::models::JrpgBattleState;

        // Only update visual timers when screen is on
        if game_state.screen_on {
            // Update battle message timer
            if game_state.jrpg_battle_message_timer > 0 {
                game_state.jrpg_battle_message_timer = game_state.jrpg_battle_message_timer.saturating_sub(delta_ms);
                if game_state.jrpg_battle_message_timer == 0 {
                    game_state.jrpg_battle_message = None;
                    game_state.needs_redraw = true;
                }
            }

            // Update damage animation timer (floats up and fades out over 1 second)
            if game_state.jrpg_damage_animation_timer > 0 {
                game_state.jrpg_damage_animation_timer = game_state.jrpg_damage_animation_timer.saturating_sub(delta_ms);
                if game_state.jrpg_damage_animation_timer == 0 {
                    game_state.jrpg_damage_dealt = 0;
                }
                game_state.needs_redraw = true; // Always redraw while animating
            }
        }

        // Update action animation timer and progress states (ALWAYS runs - game logic)
        if game_state.jrpg_action_animation_timer > 0 {
            game_state.jrpg_action_animation_timer = game_state.jrpg_action_animation_timer.saturating_sub(delta_ms);

            if game_state.jrpg_action_animation_timer == 0 {
                // Animation finished, progress to next state
                match game_state.jrpg_battle_state {
                    JrpgBattleState::PlayerAction => {
                        // Check if enemy defeated
                        if let Some(enemy) = &game_state.jrpg_enemy_combatant {
                            if enemy.hp == 0 {
                                // Transition to dying state to play death animation
                                game_state.jrpg_battle_state = JrpgBattleState::EnemyDying;
                                game_state.jrpg_action_animation_timer = 1200; // Duration for death animation

                                // Set monster dying animation (only when screen is on)
                                if game_state.screen_on {
                                    use crate::tamagotchi::models::MonsterAnimation;
                                    game_state.monster_animation = MonsterAnimation::Dying;
                                    game_state.monster_animation_frame = 0;
                                    game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                                }
                            } else {
                                // Enemy still alive, enemy's turn
                                game_state.jrpg_battle_state = JrpgBattleState::EnemyTurn;
                                game_state.jrpg_action_animation_timer = 500; // Brief pause
                            }
                        }
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    JrpgBattleState::EnemyDying => {
                        // Death animation finished, show victory
                        game_state.jrpg_battle_state = JrpgBattleState::Victory;
                        game_state.jrpg_battle_message = Some("Victory!");
                        game_state.jrpg_battle_message_timer = 0; // Don't auto-hide
                        // Set battle_end_time to prevent immediate accidental close
                        game_state.battle_end_time = game_state.last_update_ms;
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    JrpgBattleState::EnemyTurn => {
                        // Execute enemy action
                        game_state.jrpg_enemy_attack();
                        game_state.jrpg_battle_state = JrpgBattleState::EnemyAction;
                        game_state.jrpg_action_animation_timer = 1500;
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    JrpgBattleState::EnemyAction => {
                        // Check if hero defeated
                        if let Some(hero) = &game_state.jrpg_hero_combatant {
                            if hero.hp == 0 {
                                game_state.jrpg_battle_state = JrpgBattleState::Defeat;
                                game_state.jrpg_battle_message = Some("Defeat...");
                                game_state.jrpg_battle_message_timer = 0; // Don't auto-hide
                            } else {
                                // Hero still alive, back to player turn
                                game_state.jrpg_battle_state = JrpgBattleState::PlayerTurn;
                                game_state.jrpg_battle_message = None;
                            }
                        }
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Only update GIF animations when screen is on
        if game_state.screen_on {
            // Update GIF animations during JRPG battle
            update_hero_animation(&mut game_state, delta_ms);

            // Save enemy name to avoid borrow checker issues
            let enemy_name = game_state.jrpg_enemy_combatant.as_ref().map(|e| e.name);
            if let Some(name) = enemy_name {
                update_monster_animation(&mut game_state, delta_ms, name);
            }
        }
    }
}

