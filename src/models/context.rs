use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::hero::Personnage;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;
use crate::models::enemy::Loot;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Action {
    Map,
    HeroOverview,
    Inventory,
    FightManual,
    FightAuto,
    BackMap,
    BackManualFight,
    Wrap,
    Attack,
    SkillList,
    Skill,
    Equip,
    EquipmentPage,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Activity {
    //  show all direction or fight of current location
    BrowseLocation,
    // manual fight with multiple page (search enemy, battle overview, battle spell, battle resume)
    ManualCombat(ManualCombatState),
    // automatic fight with multiple page (search enemy, battle overview, battle resume, global resume)
    AutoCombat(AutoCombatState),
    // show hero stats
    HeroOverview(HeroOverviewState),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum HeroOverviewState {
    Overview,
    Inventory,
    Equipment,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ManualCombatState {
    /// Vue principale du combat, tour par tour.
    Overview,
    /// Le joueur est en train de choisir un sort dans une liste.
    SelectingSkill,
    /// Le joueur est en train de choisir un objet.
    SelectingItem,
    /// Affiche le résumé du combat qui vient de se terminer.
    Result { rewards: Vec<Loot> },
}



#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum AutoCombatState {
    /// Recherche un ennemi, avec un timer.
    Searching { end_time: DateTime<Utc> },
    /// Le combat automatique se déroule.
    Fighting,
    /// Affiche les récompenses du dernier combat avant de relancer la recherche.
    Result { rewards: Vec<Loot> },
    /// Le joueur est mort.
    Dead { end_time: DateTime<Utc> },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LocationConnection {
    pub label: String,
    pub target_id: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Location {
    pub name: String,
    pub id: u32,
    pub connections: Vec<LocationConnection>,
    pub enemies: Vec<EnemyShort>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EnemyShort {
    pub name: String,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CTA {
    pub label: String,
    pub action: Action,
    pub id: Option<i32>,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Context {
    pub activity: Activity,
    pub last_action_time: DateTime<Utc>,
    pub hero: Personnage,
    pub battle: Battle,
    pub enemy: Enemy,
    pub location: Location,
    pub needs_redraw: bool,
}
