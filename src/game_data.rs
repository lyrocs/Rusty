use std::fs;
use anyhow::Result;
use serde::Deserialize;
use crate::models::hero::Exp;
use crate::models::context::Location;
use crate::models::enemy::Enemy;
use crate::models::item::Equipment;

pub fn get_locations() -> Result<Vec<Location>> {
    let json_contenu = fs::read_to_string("data/maps.json")?;
    let game_data: Vec<Location> = serde_json::from_str(&json_contenu)?;

    Ok(game_data)
}

pub fn get_enemies() -> Result<Vec<Enemy>> {
    let json_contenu = fs::read_to_string("data/enemies.json")?;
    let game_data: Vec<Enemy> = serde_json::from_str(&json_contenu)?;

    Ok(game_data)
}

pub fn get_base_exp() -> Result<Vec<Exp>> {
    let json_contenu = fs::read_to_string("data/base_exp.json")?;
    let game_data: Vec<Exp> = serde_json::from_str(&json_contenu)?;

    Ok(game_data)
}

pub fn get_novice_exp() -> Result<Vec<Exp>> {
    let json_contenu = fs::read_to_string("data/novice_exp.json")?;
    let game_data: Vec<Exp> = serde_json::from_str(&json_contenu)?;

    Ok(game_data)
}

pub fn get_equipements() -> Result<Vec<Equipment>> {
    let json_contenu = fs::read_to_string("data/equipements.json")?;
    let game_data: Vec<Equipment> = serde_json::from_str(&json_contenu)?;

    Ok(game_data)
}