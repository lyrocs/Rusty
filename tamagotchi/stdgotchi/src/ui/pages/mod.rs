//! Page Implementations
//!
//! Concrete page implementations for different game screens.

pub mod battle;
pub mod death;
pub mod map;
pub mod menu;
pub mod rustymon_list;
pub mod rustymon_detail;
pub mod rustymon_skills;
pub mod fragment_collection_page;
pub mod rustymon_summon;

pub use battle::BattlePage;
pub use death::DeathPage;
pub use map::{MapPage, TouchAction};
pub use menu::MenuPage;
pub use rustymon_list::{RustymonListPage, RustymonListAction};
pub use rustymon_detail::{RustymonDetailPage, RustymonDetailAction};
pub use rustymon_skills::{RustymonSkillsPage, RustymonSkillsAction};
pub use fragment_collection_page::{FragmentCollectionPage, FragmentCollectionAction};
pub use rustymon_summon::{RustymonSummonPage, RustymonSummonAction};
