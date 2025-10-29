/// Rest/recovery game logic
///
/// Extension methods for GameState related to rest and HP/SP recovery.

use crate::core::GameState;
use crate::combat::RestState;

impl GameState {
    pub fn init_rest_state(&mut self) {
        // Check if already fully recovered
        if self.hero.sp >= self.hero.max_sp && self.hero.hp >= self.hero.max_hp {
            self.rest_state = RestState::FullSP;
        } else {
            self.rest_state = RestState::Resting;
        }
        self.rest_progress = 0;
    }

    /// Update rest progress
    pub fn update_rest_progress(&mut self, delta_ms: u32) {
        if self.rest_state == RestState::Resting {
            self.rest_progress += delta_ms;

            // Regenerate SP and HP every second
            if self.rest_progress >= 1000 {
                let seconds = self.rest_progress / 1000;

                // Regenerate SP (5 SP per second by default)
                self.hero
                    .regenerate_sp((seconds as u16) * self.sp_regen_rate);

                // Regenerate HP (10 HP per second)
                let hp_regen_rate = 10u16;
                self.hero.heal((seconds as u16) * hp_regen_rate);

                self.rest_progress %= 1000;

                // Check if both SP and HP are full
                if self.hero.sp >= self.hero.max_sp && self.hero.hp >= self.hero.max_hp {
                    self.rest_state = RestState::FullSP;
                }
            }
        }
    }

    /// Get farm progress percentage
    pub fn farm_progress_percent(&self) -> u8 {
        ((self.farm_progress as u64 * 100) / self.farm_duration_ms as u64) as u8
    }
}
