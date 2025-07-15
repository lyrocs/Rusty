use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use crate::models::hero::Personnage;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Action {
    Overview,
    Battle,
    BattleSpell,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Context {
    pub action: Action,
    pub last_action_time: DateTime<Utc>,
    pub hero: Personnage,
    pub battle: Battle,
    pub enemy: Enemy,
}
