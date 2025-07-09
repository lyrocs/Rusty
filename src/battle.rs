use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Battle {
    pub turn: String,    // "hero" or "enemy"
    pub status: String,  // "ongoing" or "ended" or ""
    pub message: String, // "Enemy attacks you" or "You attack enemy" or ""
}
