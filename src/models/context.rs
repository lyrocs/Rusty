use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Context {
    pub action: String,
    pub last_action_time: DateTime<Utc>,
}
