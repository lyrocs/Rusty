//! Expedition State
//!
//! Manages expedition instances and their lifecycle.

use serde::{Deserialize, Serialize};

/// Expedition durations in minutes
/// NOTE: Development values (1-4 min). Change to 20/60/240/480 for production.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpeditionDuration {
    Short = 1,       // 1 minute (dev) - was 20 minutes
    Medium = 2,      // 2 minutes (dev) - was 1 hour
    Long = 3,        // 3 minutes (dev) - was 4 hours
    Overnight = 4,   // 4 minutes (dev) - was 8 hours
}

impl ExpeditionDuration {
    pub fn minutes(&self) -> u32 {
        *self as u32
    }

    pub fn seconds(&self) -> u64 {
        (self.minutes() as u64) * 60
    }

    /// Convert seconds back to duration enum
    pub fn from_seconds(secs: u64) -> Self {
        match secs {
            0..=90 => ExpeditionDuration::Short,      // ~1 min
            91..=150 => ExpeditionDuration::Medium,   // ~2 min
            151..=210 => ExpeditionDuration::Long,    // ~3 min
            _ => ExpeditionDuration::Overnight,       // 4+ min
        }
    }
}

/// An active expedition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expedition {
    /// Unique expedition ID
    pub id: String,
    /// Map ID being explored
    pub map_id: String,
    /// Monster IDs in the expedition (1-3)
    pub monster_ids: Vec<String>,
    /// Duration of the expedition
    pub duration: ExpeditionDuration,
    /// Unix timestamp when expedition started
    pub started_at: u64,
    /// Whether expedition is complete
    pub completed: bool,
}

impl Expedition {
    /// Create a new expedition
    pub fn new(
        id: String,
        map_id: String,
        monster_ids: Vec<String>,
        duration: ExpeditionDuration,
        started_at: u64,
    ) -> Self {
        Self {
            id,
            map_id,
            monster_ids,
            duration,
            started_at,
            completed: false,
        }
    }

    /// Check if expedition is complete based on current time
    pub fn is_complete(&self, current_time: u64) -> bool {
        current_time >= self.started_at + self.duration.seconds()
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self, current_time: u64) -> u64 {
        let end_time = self.started_at + self.duration.seconds();
        if current_time >= end_time {
            0
        } else {
            end_time - current_time
        }
    }

    /// Get progress percentage (0.0 to 1.0)
    pub fn progress(&self, current_time: u64) -> f32 {
        let elapsed = current_time.saturating_sub(self.started_at);
        let total = self.duration.seconds();
        (elapsed as f32 / total as f32).min(1.0)
    }
}
