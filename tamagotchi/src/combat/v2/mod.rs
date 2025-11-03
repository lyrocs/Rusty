/// Combat system v2 - Flexible stats-driven combat with smooth animations
///
/// This is a complete rework of the idle farm combat system with:
/// - Stats-driven timing (ASPD based on AGI)
/// - Clean state machine architecture
/// - Animation preloading for smooth transitions
/// - Fixed 100ms update loop
/// - Separated concerns (combat logic, animations, calculations)

pub mod state;
pub mod animation;
pub mod calculator;
pub mod engine;
pub mod frame_tracker;

pub use state::{CombatPhase, CombatState};
pub use animation::AnimationController;
pub use calculator::{CombatCalculator, calculate_attack_speed_ms};
pub use engine::{CombatEngine, update_idle_farm_v2};
pub use frame_tracker::{FrameTracker, AnimationType};
