use std::fs;
use anyhow::Result;
use serde::Deserialize;
use crate::models::context::Location;
use crate::models::enemy::Enemy;

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
    