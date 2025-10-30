/// Card data management
///
/// Loads and provides access to card data from JSON.
use heapless::Vec as HeaplessVec;
use serde::Deserialize;

use super::common::LazyData;
use crate::hero::equipment::{Card, CardEffect, EquipmentSlot};

// Embed JSON file at compile time
const CARDS_JSON: &str = include_str!("../../assets/data/cards.json");

/// Card data structure (matches cards.json)
#[derive(Debug, Deserialize, Clone)]
pub struct CardData {
    pub id: u16,
    pub name: &'static str,
    pub allowed_slots: HeaplessVec<&'static str, 4>,
    pub effects: CardEffectData,
    pub drop_from: &'static str,
    pub drop_rate: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CardEffectData {
    #[serde(default)]
    pub exp_bonus: u8,
    #[serde(default)]
    pub sp_regen: u8,
    #[serde(default)]
    pub aspd_bonus: u8,
    #[serde(default)]
    pub hp_bonus: u16,
    #[serde(default)]
    pub vit_bonus: u8,
    #[serde(default)]
    pub description: &'static str,
}

// Static storage for parsed card data
static CARDS: LazyData<HeaplessVec<CardData, 32>> = LazyData::new();

/// Parse cards from JSON (done once, cached)
fn parse_cards() -> HeaplessVec<CardData, 32> {
    esp_println::println!("[GAME_DATA] Parsing cards.json...");

    match serde_json_core::from_str::<HeaplessVec<CardData, 32>>(CARDS_JSON) {
        Ok((cards, _)) => {
            esp_println::println!("[GAME_DATA] Successfully parsed {} cards", cards.len());
            for card in &cards {
                esp_println::println!(
                    "  - {} (ID: {}, Drops from: {})",
                    card.name,
                    card.id,
                    card.drop_from
                );
            }
            cards
        }
        Err(e) => {
            esp_println::println!("[ERROR] Failed to parse cards.json: {:?}", e);
            HeaplessVec::new()
        }
    }
}

/// Parse card slot string to EquipmentSlot
fn parse_card_slot(slot_str: &str) -> EquipmentSlot {
    match slot_str {
        "Weapon" => EquipmentSlot::Weapon,
        "Armor" => EquipmentSlot::Armor,
        "Shoes" => EquipmentSlot::Shoes,
        "Garment" => EquipmentSlot::Garment,
        "Accessory" => EquipmentSlot::Accessory1,
        _ => EquipmentSlot::Weapon, // Default fallback
    }
}

/// Get card data by ID
pub fn get_card_by_id(id: u16) -> Option<Card> {
    let cards = CARDS.get_or_init(parse_cards);

    cards
        .iter()
        .find(|c| c.id == id)
        .map(|c| {
            // Use first allowed slot (cards typically have one allowed slot type)
            let allowed_slot = c.allowed_slots.get(0)
                .map(|s| parse_card_slot(s))
                .unwrap_or(EquipmentSlot::Weapon);

            Card {
                id: c.id,
                name: c.name,
                allowed_slot,
                effects: CardEffect {
                    exp_bonus: c.effects.exp_bonus,
                    sp_regen: c.effects.sp_regen,
                    aspd_bonus: c.effects.aspd_bonus,
                    hp_bonus: c.effects.hp_bonus,
                    vit_bonus: c.effects.vit_bonus,
                },
            }
        })
}

/// Get all cards
pub fn get_all_cards() -> HeaplessVec<Card, 32> {
    let cards = CARDS.get_or_init(parse_cards);
    let mut result = HeaplessVec::new();

    for card_data in cards.iter() {
        if let Some(card) = get_card_by_id(card_data.id) {
            result.push(card).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}

/// Check if a card can be socketed into a specific equipment slot
pub fn can_socket_card(card_id: u16, equipment_slot: EquipmentSlot) -> bool {
    if let Some(card) = get_card_by_id(card_id) {
        // Cards can go in either accessory slot
        if equipment_slot == EquipmentSlot::Accessory1 || equipment_slot == EquipmentSlot::Accessory2 {
            return card.allowed_slot == EquipmentSlot::Accessory1;
        }
        card.allowed_slot == equipment_slot
    } else {
        false
    }
}
