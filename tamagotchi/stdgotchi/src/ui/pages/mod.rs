//! Page Implementations
//!
//! Concrete page implementations for different game screens.

pub mod battle;
pub mod crafting;
pub mod death;
pub mod hero_overview;
pub mod map;
pub mod menu;
pub mod inventory;
pub mod equipment;
pub mod stats_allocation;
pub mod rustymon_list;
pub mod rustymon_detail;
pub mod fragment_collection_page;
pub mod rustymon_summon;

pub use battle::BattlePage;
pub use crafting::{CraftingPage, CraftingAction};
pub use death::DeathPage;
pub use hero_overview::HeroOverviewPage;
pub use map::{MapPage, TouchAction};
pub use menu::MenuPage;
pub use inventory::{InventoryPage, InventoryAction};
pub use equipment::{EquipmentPage, EquipmentAction};
pub use stats_allocation::StatsAllocationPage;
pub use rustymon_list::{RustymonListPage, RustymonListAction};
pub use rustymon_detail::{RustymonDetailPage, RustymonDetailAction};
pub use fragment_collection_page::{FragmentCollectionPage, FragmentCollectionAction};
pub use rustymon_summon::{RustymonSummonPage, RustymonSummonAction};
