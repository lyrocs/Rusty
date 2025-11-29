//! Page Implementations
//!
//! Concrete page implementations for different game screens.

pub mod afk_farm;
pub mod battle;
pub mod battle_result;
pub mod death;
pub mod map;
pub mod menu;
pub mod rest;
pub mod quest_list;
pub mod expedition_setup;

pub use afk_farm::AfkFarmPage;
pub use battle::BattlePage;
pub use battle_result::BattleResultPage;
pub use death::DeathPage;
pub use map::{MapPage, TouchAction};
pub use menu::MenuPage;
pub use rest::RestPage;
pub use quest_list::{QuestListPage, QuestListAction};
pub use expedition_setup::ExpeditionSetupPage;
