//! Page Implementations
//!
//! Concrete page implementations for different game screens.

pub mod battle;
pub mod crafting;
pub mod hero_overview;
pub mod map;
pub mod menu;
pub mod inventory;
pub mod equipment;

pub use battle::BattlePage;
pub use crafting::{CraftingPage, CraftingAction};
pub use hero_overview::HeroOverviewPage;
pub use map::{MapPage, TouchAction};
pub use menu::MenuPage;
pub use inventory::{InventoryPage, InventoryAction};
pub use equipment::{EquipmentPage, EquipmentAction};
