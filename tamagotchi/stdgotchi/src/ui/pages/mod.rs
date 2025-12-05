//! Page Implementations
//!
//! Concrete page implementations for different game screens.

pub mod afk_farm;
pub mod battle;
pub mod battle_result;
pub mod cards;
pub mod death;
pub mod map;
pub mod menu;
pub mod rest;
pub mod quest_list;
pub mod expedition_setup;
pub mod expedition_in_progress;
pub mod expedition_summary;
pub mod hero_info;
pub mod semi_active_battle;
pub mod skill_selection;
pub mod hunt_monster_list;
pub mod hunt_battle_result;

pub use afk_farm::AfkFarmPage;
pub use battle::BattlePage;
pub use battle_result::BattleResultPage;
pub use cards::CardsPage;
pub use death::DeathPage;
pub use map::{MapPage, TouchAction};
pub use menu::MenuPage;
pub use rest::RestPage;
pub use quest_list::{QuestListPage, QuestListAction};
pub use expedition_setup::ExpeditionSetupPage;
pub use expedition_in_progress::ExpeditionInProgressPage;
pub use expedition_summary::ExpeditionSummaryPage;
pub use hero_info::HeroInfoPage;
pub use semi_active_battle::{SemiActiveBattlePage, BattleResult};
pub use skill_selection::SkillSelectionPage;
pub use hunt_monster_list::{HuntMonsterListPage, HuntAction};
pub use hunt_battle_result::{HuntBattleResultPage, HuntResultAction};
