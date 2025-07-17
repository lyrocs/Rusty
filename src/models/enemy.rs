use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DropItem {
    pub item: String,
    pub id: u32,
    pub chance: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Enemy {
    pub name: String,
    pub id: u32,
    pub level: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub base_exp: u32,
    pub job_exp: u32,
    pub drops: Vec<DropItem>,
}
