/// Farming game logic
///
/// Extension methods for GameState related to farming mechanics.

use crate::core::{GameState, GamePage};
use crate::combat::{Enemy, FarmState, MonsterAnimation, FarmDuration, calculate_kill_tick_interval};
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

    /// Start farming with efficiency system (new method)
    pub fn start_farming_with_efficiency(&mut self, enemy: Enemy, duration: FarmDuration) {
        let sp_cost = duration.sp_cost();

        if self.hero.use_sp(sp_cost) {
            // Set duration
            self.farm_duration_ms = duration.duration_ms();

            // Reset kill tracking
            self.farm_kills_count = 0;
            self.farm_last_kill_time = self.last_update_ms;

            // Start farming
            self.current_enemy = Some(enemy);
            self.farm_state = FarmState::Fighting;
            self.farm_progress = 0;
            self.current_page = GamePage::Farm;

            // Reset animation to Idle when starting new farm
            self.monster_animation = MonsterAnimation::Idle;
            self.monster_animation_frame = 0;
            self.monster_animation_started_ms = self.gif_animation_clock_ms;
            self.needs_redraw = true;

            esp_println::println!(
                "[FARM] Started {} farming with {} expected kills",
                duration.display_name(),
                self.farm_expected_kills
            );
        }
    }

    /// Update farming progress
    pub fn update_farm_progress(&mut self, delta_ms: u32) {
        if self.farm_state == FarmState::Fighting {
            self.farm_progress += delta_ms;

            // Handle kill ticks for efficiency system
            if self.farm_expected_kills > 0 {
                let kill_tick_interval = calculate_kill_tick_interval(
                    self.farm_expected_kills,
                    self.farm_duration_ms,
                );

                // Check if enough time has passed for a kill tick
                let time_since_last_kill = self.last_update_ms.saturating_sub(self.farm_last_kill_time);
                if time_since_last_kill >= kill_tick_interval && self.farm_kills_count < self.farm_expected_kills {
                    self.farm_kills_count += 1;
                    self.farm_last_kill_time = self.last_update_ms;
                    self.needs_redraw = true;

                    esp_println::println!(
                        "[FARM] Kill tick! {}/{}",
                        self.farm_kills_count,
                        self.farm_expected_kills
                    );
                }
            }

            if self.farm_progress >= self.farm_duration_ms {
                self.complete_farming();
            }
        }
    }

    /// Complete farming and award rewards
    fn complete_farming(&mut self) {
        if let Some(enemy) = &self.current_enemy {
            let enemy_id = enemy.id;

            // Use actual kills count (or 1 if efficiency system not used)
            let actual_kills = if self.farm_expected_kills > 0 {
                self.farm_kills_count
            } else {
                1 // Fallback for old farming system
            };

            // Calculate rewards with level penalty and auto-farm rate (1/10)
            let (total_exp, total_zeny) = crate::combat::calculate_farm_rewards(
                enemy,
                actual_kills,
                self.hero.level,
            );

            self.hero.add_exp(total_exp);
            self.hero.add_zeny(total_zeny);
            self.farm_state = FarmState::Victory;
            self.victory_state_entered_ms = self.last_update_ms; // Track when victory started for animation delay

            esp_println::println!(
                "[FARM] Completed! {} kills, +{} EXP, +{} Zeny",
                actual_kills,
                total_exp,
                total_zeny
            );

            // Update quest progress - monsters killed (multiple)
            for _ in 0..actual_kills {
                crate::tamagotchi::quest_system::update_quest_progress(
                    self,
                    QuestAction::MonsterKilled { enemy_id },
                );
            }

            // Update quest progress - zeny earned
            crate::tamagotchi::quest_system::update_quest_progress(
                self,
                QuestAction::ZenyEarned {
                    amount: total_zeny,
                },
            );

            // Roll for item drops (once per kill, with 1/10 chance for auto-farm)
            self.last_drops.clear();

            for kill_index in 0..actual_kills {
                let rng_value = ((self.last_update_ms + kill_index as u32) % 100) as u8;

                // Auto-farm drop rate: 10% (1/10)
                if rng_value < 10 {
                    let drop_rng = ((self.last_update_ms + kill_index as u32) % 255) as u8;
                    let drops = crate::data::roll_drops(enemy_id, drop_rng);

                    for (item_id, item_name, quantity) in drops.iter() {
                        if self.hero.add_item(*item_id, item_name, *quantity) {
                            esp_println::println!("[DROPS] Got {} x{}", item_name, quantity);
                            // Store only the first few drops to avoid overflow
                            if self.last_drops.len() < 4 {
                                self.last_drops.push((*item_id, item_name, *quantity)).ok();
                            }
                        } else {
                            esp_println::println!("[DROPS] Inventory full! Lost {}", item_name);
                        }
                    }
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
