//! Page Implementations
//!
//! Concrete page implementations for different game screens.

pub mod battle;
pub mod battle_result;
pub mod between_floors;
pub mod bonus_selection;
pub mod collection;
pub mod death;
pub mod dungeon_combat;
pub mod dungeon_defeat;
pub mod expedition_detail;
pub mod expedition_map;
pub mod expedition_result;
pub mod expedition_team_select;
pub mod home;
pub mod inventory;
pub mod map;
pub mod menu;
pub mod monster_detail;
pub mod monster_list;
pub mod monster_upgrade;

pub use battle::{BattlePage, BattleAction};
pub use battle_result::{BattleResultPage, BattleResultAction};
pub use between_floors::{BetweenFloorsPage, BetweenFloorsAction, MonsterStatusData};
pub use bonus_selection::{BonusSelectionPage, BonusSelectionAction};
pub use collection::{CollectionPage, CollectionAction, ZoneCollectionData, SpeciesCollectionData};

pub use death::DeathPage;
pub use dungeon_combat::{DungeonCombatPage, DungeonCombatAction};
pub use dungeon_defeat::{DungeonDefeatPage, DungeonDefeatAction};
pub use expedition_detail::{ExpeditionDetailPage, ExpeditionDetailAction, ExpeditionMonsterData};
pub use expedition_map::{ExpeditionMapPage, ExpeditionMapAction, ZoneDisplayData, MapDisplayData};
pub use expedition_result::{ExpeditionResultPage, ExpeditionResultAction, ExpeditionResultData};
pub use expedition_team_select::{ExpeditionTeamSelectPage, ExpeditionTeamAction, MonsterSelectData};
pub use home::{HomePage, HomeAction, ExpeditionSlotData, TeamMonsterData};
pub use inventory::{InventoryPage, InventoryAction};
pub use map::{MapPage, TouchAction};
pub use menu::MenuPage;
pub use monster_detail::{MonsterDetailPage, MonsterDetailAction};
pub use monster_list::{MonsterListPage, MonsterListAction};
pub use monster_upgrade::{MonsterUpgradePage, MonsterUpgradeAction};
