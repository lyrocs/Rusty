use crate::models::context::Context;
use crate::models::context::Action;
use crate::models::context::Activity;
use crate::models::hero::Personnage;
use crate::models::hero::Skill;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;
use crate::game_data;
use anyhow::Result;
use chrono::prelude::*;
use redb::{Database, TableDefinition};

const CONTEXT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("context");

pub fn init_db(db: &Database) -> Result<()> {
    let read_txn = db.begin_read()?;
    let _table = match read_txn.open_table(CONTEXT_TABLE) {
        Ok(table) => table,
        Err(_) => {
            init_db_data(&db)?;
            return Ok(());
        }
    };

    Ok(())
}

pub fn init_db_data(db: &Database) -> Result<()> {
    let locations = game_data::get_locations()?;
    let first_location = locations.first().unwrap();
    let context: Context = Context {
        action: Action::Overview,
        // activity: Activity::HeroOverview,
        activity: Activity::HeroOverview,
        last_action_time: Utc::now(),
        needs_redraw: true,
        hero: Personnage {
            nom: "Lyrocs".to_string(),
            classe: "Novice".to_string(),
            base_level: 1,
            base_exp: 0,
            job_level: 1,
            job_exp: 0,
            hp: 100,
            max_hp: 100,
            mp: 100,
            max_mp: 100,
            inventaire: Vec::new(),
            skills: vec![Skill {
                name: "Bash".to_string(),
                icon: "bash.bmp".to_string(),
                level: 1,
                mp_cost: 10,
                description: "ATK 110%".to_string(),
            }],
        },
        battle: Battle {
            turn: "".to_string(),
            status: "".to_string(),
            message: "".to_string(),
        },
        enemy: Enemy {
            name: "".to_string(),
            id: 0,
            level: 0,
            hp: 0,
            max_hp: 0,
            attack: 0,
            defense: 0,
            base_exp: 0,
            job_exp: 0,
            drops: Vec::new(),
        },
        location: first_location.clone(),
    };
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(CONTEXT_TABLE)?;
        let context_bytes = serde_json::to_vec(&context)?;
        table.insert("context", context_bytes.as_slice())?;
    }
    write_txn.commit()?;

    Ok(())
}


pub fn get_context(db: &Database) -> Result<Context> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(CONTEXT_TABLE)?;
    if let Some(context_data) = table.get("context")? {
        let context_bytes = context_data.value();
        let context_recupere: Context = serde_json::from_slice(context_bytes)?;
        Ok(context_recupere)
    } else {
        Err(anyhow::anyhow!("Context non trouvé"))
    }
}

pub fn update_context(db: &Database, context: Context) -> Result<Context> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(CONTEXT_TABLE)?;
    if let Some(context_data) = table.get("context")? {
        let context_bytes = context_data.value();
        let context_recupere: Context = serde_json::from_slice(context_bytes)?;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(CONTEXT_TABLE)?;
            let hero_bytes = serde_json::to_vec(&context)?;
            table.insert("context", hero_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(context_recupere)
    } else {
        Err(anyhow::anyhow!("Context non trouvé"))
    }
}
