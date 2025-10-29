/// Equipment data management
///
/// Loads and provides access to equipment data from JSON.
use heapless::Vec as HeaplessVec;
use serde::Deserialize;

use super::common::LazyData;
use crate::hero::equipment::{Equipment, EquipmentSlot, EquipmentType};

// Embed JSON file at compile time
const EQUIPMENTS_JSON: &str = include_str!("../../assets/data/equipments.json");

/// Equipment data structure (matches equipments.json)
#[derive(Debug, Deserialize, Clone)]
pub struct EquipmentData {
    pub id: u16,
    pub name: &'static str,
    pub equipment_type: &'static str,
    pub slot: &'static str,
    pub level_req: u16,
    pub job_req: Option<&'static str>,
    pub atk_bonus: u16,
    pub def_bonus: u16,
    pub hp_bonus: u16,
    pub sp_bonus: u16,
    pub str_bonus: i16,
    pub agi_bonus: i16,
    pub vit_bonus: i16,
    pub int_bonus: i16,
    pub dex_bonus: i16,
    pub luk_bonus: i16,
    pub crit_rate_bonus: u16,
    pub aspd_bonus: u16,
    pub max_refine: u8,
    pub can_upgrade: bool,
    pub upgrade_level_req: u16,
    pub upgrade_cost: u32,
    pub upgrades_to: Option<u16>,
}

// Static storage for parsed equipment data
static EQUIPMENTS: LazyData<HeaplessVec<EquipmentData, 32>> = LazyData::new();

/// Parse equipments from JSON (done once, cached)
fn parse_equipments() -> HeaplessVec<EquipmentData, 32> {
    esp_println::println!("[GAME_DATA] Parsing equipments.json...");

    match serde_json_core::from_str::<HeaplessVec<EquipmentData, 32>>(EQUIPMENTS_JSON) {
        Ok((equipments, _)) => {
            esp_println::println!("[GAME_DATA] Successfully parsed {} equipments", equipments.len());
            for equip in &equipments {
                esp_println::println!(
                    "  - {} (ID: {}, Type: {}, Slot: {})",
                    equip.name,
                    equip.id,
                    equip.equipment_type,
                    equip.slot
                );
            }
            equipments
        }
        Err(e) => {
            esp_println::println!("[ERROR] Failed to parse equipments.json: {:?}", e);
            HeaplessVec::new()
        }
    }
}

/// Parse equipment type string
fn parse_equipment_type(type_str: &str) -> EquipmentType {
    match type_str {
        "Knife" => EquipmentType::Knife,
        "Sword" => EquipmentType::Sword,
        "Spear" => EquipmentType::Spear,
        "Axe" => EquipmentType::Axe,
        "Bow" => EquipmentType::Bow,
        "Staff" => EquipmentType::Staff,
        "ClothArmor" => EquipmentType::ClothArmor,
        "LeatherArmor" => EquipmentType::LeatherArmor,
        "ChainMail" => EquipmentType::ChainMail,
        "PlateMail" => EquipmentType::PlateMail,
        "Ring" => EquipmentType::Ring,
        "Necklace" => EquipmentType::Necklace,
        "Gloves" => EquipmentType::Gloves,
        _ => EquipmentType::Knife, // Default fallback
    }
}

/// Parse equipment slot string
fn parse_equipment_slot(slot_str: &str) -> EquipmentSlot {
    match slot_str {
        "Weapon" => EquipmentSlot::Weapon,
        "Armor" => EquipmentSlot::Armor,
        "Accessory" => EquipmentSlot::Accessory,
        _ => EquipmentSlot::Weapon, // Default fallback
    }
}

/// Get equipment data by ID
pub fn get_equipment_by_id(id: u16) -> Option<Equipment> {
    let equipments = EQUIPMENTS.get_or_init(parse_equipments);

    equipments
        .iter()
        .find(|e| e.id == id)
        .map(|e| Equipment {
            id: e.id,
            name: e.name,
            equipment_type: parse_equipment_type(e.equipment_type),
            slot: parse_equipment_slot(e.slot),
            level_req: e.level_req,
            job_req: e.job_req,
            atk_bonus: e.atk_bonus,
            def_bonus: e.def_bonus,
            hp_bonus: e.hp_bonus,
            sp_bonus: e.sp_bonus,
            str_bonus: e.str_bonus,
            agi_bonus: e.agi_bonus,
            vit_bonus: e.vit_bonus,
            int_bonus: e.int_bonus,
            dex_bonus: e.dex_bonus,
            luk_bonus: e.luk_bonus,
            crit_rate_bonus: e.crit_rate_bonus,
            aspd_bonus: e.aspd_bonus,
            refine_level: 0, // Always start at +0
            max_refine: e.max_refine,
            can_upgrade: e.can_upgrade,
            upgrade_level_req: e.upgrade_level_req,
            upgrade_cost: e.upgrade_cost,
            upgrades_to: e.upgrades_to,
        })
}

/// Get all equipments
pub fn get_all_equipments() -> HeaplessVec<Equipment, 32> {
    let equipments = EQUIPMENTS.get_or_init(parse_equipments);
    let mut result = HeaplessVec::new();

    for equip_data in equipments.iter() {
        if let Some(equip) = get_equipment_by_id(equip_data.id) {
            result.push(equip).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}
