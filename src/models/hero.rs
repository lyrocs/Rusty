use serde::{Deserialize, Serialize};

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
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub niveau: u8,
    pub experience: u32,
    pub inventaire: Vec<String>,
    pub skills: Vec<Skill>,
}
