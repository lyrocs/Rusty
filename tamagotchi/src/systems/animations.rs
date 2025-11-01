/// Animation helper functions for battle and character animations
use crate::core::GameState;

/// Helper function to update monster GIF animation
/// Uses global animation clock for synchronized updates - only sets needs_redraw when frame changes
pub fn update_monster_animation(game_state: &mut GameState, _delta_ms: u32, monster_name: &str) {
    use crate::tamagotchi::models::MonsterAnimation;
    use embedded_graphics::pixelcolor::Rgb888;
    use tinygif::Gif;

    let gif_data = game_state.monster_animation.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");
    let total_frames = gif.frames().count();

    // Dynamic frame duration: 150ms for Idle, 100ms for actions
    // Slower action animations make them more visible and readable
    let frame_duration_ms = match game_state.monster_animation {
        MonsterAnimation::Idle => 150,
        MonsterAnimation::Attacking | MonsterAnimation::Attacked | MonsterAnimation::Dying => 100,
    };

    let elapsed_ms = game_state
        .gif_animation_clock_ms
        .wrapping_sub(game_state.monster_animation_started_ms);
    let target_frame = ((elapsed_ms / frame_duration_ms) as usize) % total_frames;

    // Only update and redraw if frame actually changed
    if game_state.monster_animation.should_loop() {
        // Loop animations (Idle)
        if game_state.monster_animation_frame != target_frame {
            game_state.monster_animation_frame = target_frame;
            game_state.needs_redraw = true;
        }
    } else {
        // Play-once animations (Attacking, Attacked, Dying)
        if game_state.monster_animation_frame < total_frames - 1 {
            if game_state.monster_animation_frame != target_frame {
                game_state.monster_animation_frame = target_frame.min(total_frames - 1);
                game_state.needs_redraw = true;
            }
        } else {
            // Animation finished
            // For Dying: stay on last frame (keep displaying dead monster)
            // For Attacking/Attacked: return to Idle
            if game_state.monster_animation == MonsterAnimation::Attacking
                || game_state.monster_animation == MonsterAnimation::Attacked
            {
                game_state.monster_animation = MonsterAnimation::Idle;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                game_state.needs_redraw = true;
            }
            // MonsterAnimation::Dying stays on last frame - no transition
        }
    }
}

/// Helper function to update hero GIF animation
/// Uses global animation clock for synchronized updates - only sets needs_redraw when frame changes
pub fn update_hero_animation(game_state: &mut GameState, _delta_ms: u32) {
    use crate::tamagotchi::models::HeroAnimation;
    use embedded_graphics::pixelcolor::Rgb888;
    use tinygif::Gif;

    let gif_data = game_state.hero_animation.gif_data(&game_state.hero.job);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse hero GIF");
    let total_frames = gif.frames().count();

    // Dynamic frame duration: 150ms for Idle/Resting, 100ms for actions
    // Slower action animations make them more visible and readable
    let frame_duration_ms = match game_state.hero_animation {
        HeroAnimation::Idle | HeroAnimation::Resting => 150,
        HeroAnimation::Attacking | HeroAnimation::Attacked => 100,
    };

    let elapsed_ms = game_state
        .gif_animation_clock_ms
        .wrapping_sub(game_state.hero_animation_started_ms);
    let target_frame = ((elapsed_ms / frame_duration_ms) as usize) % total_frames;

    // Only update and redraw if frame actually changed
    if game_state.hero_animation.should_loop() {
        // Loop animations (Resting, Idle)
        if game_state.hero_animation_frame != target_frame {
            game_state.hero_animation_frame = target_frame;
            game_state.needs_redraw = true;
        }
    } else {
        // Play-once animations (Attacking, Attacked)
        if game_state.hero_animation_frame < total_frames - 1 {
            if game_state.hero_animation_frame != target_frame {
                game_state.hero_animation_frame = target_frame.min(total_frames - 1);
                game_state.needs_redraw = true;
            }
        } else {
            // Animation finished - return to Idle
            if game_state.hero_animation == HeroAnimation::Attacking
                || game_state.hero_animation == HeroAnimation::Attacked
            {
                game_state.hero_animation = HeroAnimation::Idle;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
                game_state.needs_redraw = true;
            }
        }
    }
}

/// Update hero and monster animations for battle based on current animation phase
pub fn update_battle_animations(game_state: &mut GameState, delta_ms: u32, monster_name: &str) {
    use crate::tamagotchi::models::{BattleAnimationPhase, HeroAnimation, MonsterAnimation};

    // Set animations based on current phase
    match game_state.battle_animation_phase {
        BattleAnimationPhase::BothIdle => {
            // Both on idle animation
            if game_state.hero_animation != HeroAnimation::Idle {
                game_state.hero_animation = HeroAnimation::Idle;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            if game_state.monster_animation != MonsterAnimation::Idle {
                game_state.monster_animation = MonsterAnimation::Idle;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            // Update both idle animations
            update_hero_animation(game_state, delta_ms);
            update_monster_animation(game_state, delta_ms, monster_name);
        }
        BattleAnimationPhase::MonsterAttacking => {
            // Monster attacks (16.gif), hero gets hit (52.gif)
            if game_state.monster_animation != MonsterAnimation::Attacking {
                game_state.monster_animation = MonsterAnimation::Attacking;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            if game_state.hero_animation != HeroAnimation::Attacked {
                game_state.hero_animation = HeroAnimation::Attacked;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            update_hero_animation(game_state, delta_ms);
            update_monster_animation(game_state, delta_ms, monster_name);
        }
        BattleAnimationPhase::HeroAttacking => {
            // Hero attacks (84.gif), monster gets hit (24.gif)
            if game_state.hero_animation != HeroAnimation::Attacking {
                game_state.hero_animation = HeroAnimation::Attacking;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            // Set monster to Attacked animation (24.gif)
            if game_state.monster_animation != MonsterAnimation::Attacked {
                game_state.monster_animation = MonsterAnimation::Attacked;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            update_hero_animation(game_state, delta_ms);
            update_monster_animation(game_state, delta_ms, monster_name);
        }
    }
}
