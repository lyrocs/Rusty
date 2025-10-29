/// Farming game logic
///
/// Extension methods for GameState related to farming mechanics.

use crate::core::{GameState, GamePage};
use crate::combat::{Enemy, FarmState, MonsterAnimation};
use crate::quest::QuestAction;

impl GameState {
    pub fn roll_item_drop(&self, enemy_id: u32, _rng_value: u8) -> Option<(u32, &'static str)> {
        // Simple item drop table based on enemy
        match enemy_id {
            1002 => Some((512, "Apple")),           // Poring drops Apple
            1007 => Some((705, "Clover")),          // Fabre drops Clover
            1004 => Some((518, "Honey")),           // Hornet drops Honey
            1051 => Some((955, "Worm Peeling")),   // Thief Bug drops Worm Peeling
            _ => None,
        }
    }

    /// Start farming with a new enemy
    pub fn start_farming(&mut self, enemy: Enemy) {
        if self.hero.use_sp(20) {
            self.current_enemy = Some(enemy);
            self.farm_state = FarmState::Fighting;
            self.farm_progress = 0;
            self.current_page = GamePage::Farm;
            // Reset animation to Idle when starting new farm
            self.monster_animation = MonsterAnimation::Idle;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;
            self.needs_redraw = true;
        }
    }

    /// Update farming progress
    pub fn update_farm_progress(&mut self, delta_ms: u32) {
        if self.farm_state == FarmState::Fighting {
            self.farm_progress += delta_ms;

            if self.farm_progress >= self.farm_duration_ms {
                self.complete_farming();
            }
        }
    }

    /// Complete farming and award rewards
    fn complete_farming(&mut self) {
        if let Some(enemy) = &self.current_enemy {
            let enemy_id = enemy.id;
            let zeny_earned = enemy.zeny_reward;

            self.hero.add_exp(enemy.base_exp);
            self.hero.add_zeny(zeny_earned);
            self.farm_state = FarmState::Victory;

            // Update quest progress - monster killed
            crate::tamagotchi::quest_system::update_quest_progress(
                self,
                QuestAction::MonsterKilled { enemy_id },
            );

            // Update quest progress - zeny earned
            crate::tamagotchi::quest_system::update_quest_progress(
                self,
                QuestAction::ZenyEarned {
                    amount: zeny_earned,
                },
            );

            // Roll for item drops
            let rng_value = (self.last_update_ms % 255) as u8;
            let drops = crate::data::roll_drops(enemy_id, rng_value);

            // Clear previous drops and store new ones
            self.last_drops.clear();

            for (item_id, item_name, quantity) in drops.iter() {
                if self.hero.add_item(*item_id, item_name, *quantity) {
                    esp_println::println!("[DROPS] Got {} x{}", item_name, quantity);
                    self.last_drops.push((*item_id, item_name, *quantity)).ok();
                } else {
                    esp_println::println!("[DROPS] Inventory full! Lost {}", item_name);
                }
            }
        }
    }

    /// Reset farming state
    pub fn reset_farming(&mut self) {
        self.current_enemy = None;
        self.farm_state = FarmState::Idle;
        self.farm_progress = 0;
        // Set cooldown to prevent immediate re-touch (300ms)
        self.farm_touch_cooldown = 300;
        // Reset animation to Idle
        self.monster_animation = MonsterAnimation::Idle;
        self.monster_animation_frame = 0;
        self.monster_animation_started_ms = self.gif_animation_clock_ms;
    }
}
