//! Leveling System
//!
//! Handles XP gain and level up logic for monsters.

use crate::game::core::Monster;
use crate::game::calculations::{xp, stats};

/// Apply XP gain to a monster and handle level ups
/// Returns the number of levels gained
pub fn apply_xp_to_monster(monster: &mut Monster, xp_gained: u32) -> u8 {
    let (new_level, new_xp, new_xp_to_next, levels_gained) =
        xp::apply_xp_gain(monster.level, monster.xp, xp_gained);

    if levels_gained > 0 {
        // Update level and XP
        monster.level = new_level;
        monster.xp = new_xp;
        monster.xp_to_next = new_xp_to_next;

        // Recalculate stats based on new level
        // Note: This requires base stats from species, which we'll need to look up
        // For now, we apply a simple scaling
        recalculate_stats(monster);
    } else {
        monster.xp = new_xp;
    }

    levels_gained
}

/// Recalculate monster stats after level up
/// This should be called with base stats from species data
fn recalculate_stats(monster: &mut Monster) {
    // Simple scaling for now - proper implementation needs species base stats
    // This is a temporary implementation until data loader is complete
    let old_hp_percent = monster.hp_percentage();

    // Stats increase ~2% per level, HP increases ~3% per level
    // This is already applied by the base stat * level formula
    // For now, just restore HP percentage
    monster.hp_current = ((monster.hp_max as f32) * old_hp_percent).round() as u16;
}

/// Recalculate stats with base values from species
pub fn recalculate_stats_with_base(
    monster: &mut Monster,
    base_hp: u16,
    base_atk: u16,
    base_def: u16,
    base_spd: u16,
) {
    let old_hp_percent = monster.hp_percentage();

    monster.hp_max = stats::calculate_final_hp(base_hp, monster.level, monster.fusion_count);
    monster.atk = stats::calculate_final_stat(base_atk, monster.level, monster.fusion_count);
    monster.def = stats::calculate_final_stat(base_def, monster.level, monster.fusion_count);
    monster.spd = stats::calculate_final_stat(base_spd, monster.level, monster.fusion_count);

    // Restore HP percentage
    monster.hp_current = ((monster.hp_max as f32) * old_hp_percent).round() as u16;
}
