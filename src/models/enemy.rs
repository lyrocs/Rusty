use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DropItem {
    pub chance: f32,
    pub drop: Loot,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Loot {
    /// Un objet de base avec une quantité (ex: 3 Jellopy).
    Item { id: u32, name: String, quantity: u32 },
    
    /// Une pièce d'équipement (ex: une Dague).
    Equipment { id: u32, name: String },
    
    /// De la monnaie du jeu.
    Zeny { amount: u32 },
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
