use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::hero::Personnage;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Context {
    pub action: String,
    pub last_action_time: DateTime<Utc>,
    pub hero: Personnage,
    pub battle: Option<Battle>,
    pub enemy: Option<Enemy>,
}

impl Context {
    pub fn update_battle(&mut self, turn: String, message: String) {
        if let Some(battle) = self.battle.as_mut() {
            battle.turn = turn;
            battle.message = message;
        }
    }
}
