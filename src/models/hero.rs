use serde::{Deserialize, Serialize};
use crate::models::context::LootItem;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Skill {
    pub name: String,
    pub icon: String,
    pub level: u32,
    pub mp_cost: u32,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Personnage {
    pub nom: String,
    pub classe: String,
    pub base_level: u32,
    pub base_exp: u32,
    pub job_level: u32,
    pub job_exp: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub inventaire: Vec<LootItem>,
    pub skills: Vec<Skill>,
}
