use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::hero::Personnage;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Action {
    Overview,
    Battle,
    BattleSpell,
    BattleAuto,
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
    HeroOverview,
    // show inventory
    Inventory
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
    Result { rewards: Vec<LootItem> },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LootItem { 
    pub item_id: String, 
    pub quantity: u32 
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum AutoCombatState {
    /// Recherche un ennemi, avec un timer.
    Searching { end_time: DateTime<Utc> },
    /// Le combat automatique se déroule.
    Fighting,
    /// Affiche les récompenses du dernier combat avant de relancer la recherche.
    Result { rewards: Vec<LootItem> },
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
pub struct Context {
    pub action: Action,
    pub activity: Activity,
    pub last_action_time: DateTime<Utc>,
    pub hero: Personnage,
    pub battle: Battle,
    pub enemy: Enemy,
    pub location: Location,
    pub needs_redraw: bool,
}
