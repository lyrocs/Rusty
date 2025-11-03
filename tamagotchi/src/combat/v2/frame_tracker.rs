/// Frame-based animation tracking for precise combat timing
///
/// Ensures animations complete all frames before transitioning phases

use crate::combat::animations::{HeroAnimation, MonsterAnimation};

/// Tracks animation frame progression and completion
#[derive(Debug, Clone)]
pub struct FrameTracker {
    pub animation_type: AnimationType,
    pub current_frame: u8,
    pub total_frames: u8,
    pub frame_duration_ms: u32,
    pub last_frame_change_ms: u32,
    pub animation_complete: bool,
    pub animation_started_ms: u32,
}

impl FrameTracker {
    /// Create a new frame tracker in idle state
    pub fn new(current_ms: u32) -> Self {
        Self {
            animation_type: AnimationType::Idle,
            current_frame: 0,
            total_frames: 4, // Idle default
            frame_duration_ms: 150, // Idle frame rate
            last_frame_change_ms: current_ms,
            animation_complete: false,
            animation_started_ms: current_ms,
        }
    }

    /// Start tracking a new animation
    pub fn start_animation(&mut self, anim_type: AnimationType, current_ms: u32) {
        self.start_animation_with_frames(anim_type, anim_type.frame_count(), current_ms);
    }

    /// Start tracking a new animation with custom frame count
    /// Use this for idle animations with variable frame counts
    pub fn start_animation_with_frames(&mut self, anim_type: AnimationType, frame_count: u8, current_ms: u32) {
        esp_println::println!(
            "[FRAME_TRACKER] Starting animation: {:?} with {} frames @ {}ms/frame",
            anim_type,
            frame_count,
            anim_type.frame_duration()
        );

        self.animation_type = anim_type;
        self.current_frame = 0;
        self.total_frames = frame_count;
        self.frame_duration_ms = anim_type.frame_duration();
        self.last_frame_change_ms = current_ms;
        self.animation_started_ms = current_ms;
        self.animation_complete = false;
    }

    /// Update animation frame based on elapsed time
    /// Returns true if animation just completed
    pub fn update(&mut self, current_ms: u32) -> bool {
        if self.animation_complete {
            return false; // Already complete
        }

        // Calculate which frame we should be on based on total elapsed time
        let total_elapsed = current_ms.saturating_sub(self.animation_started_ms);
        let target_frame = (total_elapsed / self.frame_duration_ms) as u8;

        // Check if we need to advance frames
        if target_frame != self.current_frame {
            self.current_frame = target_frame;

            // Log frame advance
            esp_println::println!(
                "[FRAME_TRACKER] {:?} advanced to frame {}/{}",
                self.animation_type,
                self.current_frame,
                self.total_frames
            );

            // Check if animation completed
            if self.current_frame >= self.total_frames {
                self.current_frame = self.total_frames.saturating_sub(1); // Stay on last frame
                self.animation_complete = true;

                esp_println::println!(
                    "[FRAME_TRACKER] {:?} animation COMPLETE (all {} frames shown)",
                    self.animation_type,
                    self.total_frames
                );

                return true; // Just completed
            }
        }

        false // Still running
    }

    /// Check if enough time has passed for a frame change
    pub fn should_advance_frame(&self, current_ms: u32) -> bool {
        if self.animation_complete {
            return false;
        }

        let elapsed = current_ms.saturating_sub(self.last_frame_change_ms);
        elapsed >= self.frame_duration_ms
    }

    /// Get current frame clamped to valid range
    pub fn get_display_frame(&self) -> u8 {
        self.current_frame.min(self.total_frames.saturating_sub(1))
    }

    /// Check if this is a looping animation
    pub fn is_looping(&self) -> bool {
        matches!(self.animation_type, AnimationType::Idle)
    }

    /// Reset to idle animation with default frame count
    pub fn reset_to_idle(&mut self, current_ms: u32) {
        self.start_animation(AnimationType::Idle, current_ms);
    }

    /// Reset to idle animation with actual GIF frame count
    pub fn reset_to_idle_with_frames(&mut self, frame_count: u8, current_ms: u32) {
        self.start_animation_with_frames(AnimationType::Idle, frame_count, current_ms);
    }
}

/// Animation types with frame data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationType {
    Idle,
    HeroAttacking,
    HeroAttacked,
    MonsterAttacking,
    MonsterAttacked,
    MonsterDying,
}

impl AnimationType {
    /// Get the number of frames for this animation
    pub fn frame_count(&self) -> u8 {
        match self {
            AnimationType::Idle => 4,              // 4-frame idle loop
            AnimationType::HeroAttacking => 8,     // 8-frame attack
            AnimationType::HeroAttacked => 3,      // 3-frame hit reaction
            AnimationType::MonsterAttacking => 8,  // 8-frame attack
            AnimationType::MonsterAttacked => 3,   // 3-frame hit reaction
            AnimationType::MonsterDying => 8,      // 8-frame death
        }
    }

    /// Get frame duration in milliseconds
    pub fn frame_duration(&self) -> u32 {
        match self {
            AnimationType::Idle => 150,  // Slower for idle
            _ => 100,                    // 100ms for all action animations
        }
    }

    /// Get total animation duration in milliseconds
    pub fn total_duration(&self) -> u32 {
        self.frame_count() as u32 * self.frame_duration()
    }

    /// Convert to HeroAnimation enum
    pub fn to_hero_animation(&self) -> HeroAnimation {
        match self {
            AnimationType::HeroAttacking => HeroAnimation::Attacking,
            AnimationType::HeroAttacked => HeroAnimation::Attacked,
            _ => HeroAnimation::Idle,
        }
    }

    /// Convert to MonsterAnimation enum
    pub fn to_monster_animation(&self) -> MonsterAnimation {
        match self {
            AnimationType::MonsterAttacking => MonsterAnimation::Attacking,
            AnimationType::MonsterAttacked => MonsterAnimation::Attacked,
            AnimationType::MonsterDying => MonsterAnimation::Dying,
            _ => MonsterAnimation::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_tracker_creation() {
        let tracker = FrameTracker::new(1000);
        assert_eq!(tracker.animation_type, AnimationType::Idle);
        assert_eq!(tracker.current_frame, 0);
        assert_eq!(tracker.total_frames, 4);
        assert!(!tracker.animation_complete);
    }

    #[test]
    fn test_animation_start() {
        let mut tracker = FrameTracker::new(1000);
        tracker.start_animation(AnimationType::HeroAttacking, 1000);

        assert_eq!(tracker.animation_type, AnimationType::HeroAttacking);
        assert_eq!(tracker.current_frame, 0);
        assert_eq!(tracker.total_frames, 8);
        assert_eq!(tracker.frame_duration_ms, 100);
        assert!(!tracker.animation_complete);
    }

    #[test]
    fn test_frame_advancement() {
        let mut tracker = FrameTracker::new(1000);
        tracker.start_animation(AnimationType::HeroAttacking, 1000);

        // Frame 0 at start
        assert_eq!(tracker.current_frame, 0);
        assert!(!tracker.update(1050)); // Not enough time for frame change

        // Frame 1 after 100ms
        assert!(!tracker.update(1100));
        assert_eq!(tracker.current_frame, 1);

        // Frame 7 after 700ms
        assert!(!tracker.update(1700));
        assert_eq!(tracker.current_frame, 7);

        // Complete after 800ms (8 frames)
        assert!(tracker.update(1800));
        assert_eq!(tracker.current_frame, 7); // Stays on last frame
        assert!(tracker.animation_complete);
    }

    #[test]
    fn test_animation_types() {
        assert_eq!(AnimationType::Idle.frame_count(), 4);
        assert_eq!(AnimationType::HeroAttacking.frame_count(), 8);
        assert_eq!(AnimationType::MonsterDying.frame_count(), 8);
        assert_eq!(AnimationType::HeroAttacked.frame_count(), 3);

        assert_eq!(AnimationType::Idle.frame_duration(), 150);
        assert_eq!(AnimationType::HeroAttacking.frame_duration(), 100);

        assert_eq!(AnimationType::HeroAttacking.total_duration(), 800);
        assert_eq!(AnimationType::HeroAttacked.total_duration(), 300);
    }
}