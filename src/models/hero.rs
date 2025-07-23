use serde::{Deserialize, Serialize};
use crate::models::item::LootItem;
use crate::models::item::Equipment;


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Exp {
    pub level: u32,
    pub exp: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Skill {
    pub name: String,
    pub id: u32,
    pub icon: String,
    pub level: u32,
    pub mp_cost: u32,
    pub description: String,
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum InventoryItem {
    // Pour les objets uniques comme les armes, armures...
    Equipment(Equipment),

    // Pour les objets qui s'empilent (potions, matériaux de craft...)
    Stackable(LootItem),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Personnage {
    pub nom: String,
    pub classe: String,
    pub base_level: u32,
    pub base_exp: u32,
    pub base_exp_next: u32,
    pub job_level: u32,
    pub job_exp: u32,
    pub job_exp_next: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub inventaire: Vec<InventoryItem>,
    pub skills: Vec<Skill>,
    pub weapon: Option<Equipment>,
    pub armor: Option<Equipment>,
    pub accessory: Option<Equipment>,
}
