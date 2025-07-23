use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LootItem { 
    pub id: u32, 
    pub name: String,
    pub quantity: u32 
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EquipmentBonus {
    pub label: String,
    pub value: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum EquipmentCategory {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Equipment {
    pub name: String,
    pub id: u32,
    pub icon: String,
    pub level: u32,
    pub attack: u32,
    pub defense: u32,
    pub description: String,
    pub bonus: Vec<EquipmentBonus>,
    pub category: EquipmentCategory,
}