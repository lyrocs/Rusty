/// UI Pages Module
///
/// Page-specific rendering functions for different game screens.
/// Currently re-exports from tamagotchi/ui.rs for backward compatibility.
/// In future refactoring, pages will be extracted to individual modules.

// Re-export all page rendering functions from tamagotchi::ui
pub use crate::tamagotchi::ui::{
    draw_overview_page,
    draw_stats_page,
    draw_equipment_page,
    draw_farm_page,
    draw_rest_page,
    draw_battle_page,
    draw_jrpg_battle_page,
    draw_map_page,
    draw_menu,
    draw_inventory,
    draw_quests_page,
    draw_settings_page,
};
