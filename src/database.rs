use crate::models::context::Context;
use crate::models::context::Action;
use crate::models::hero::Personnage;
use crate::models::hero::Skill;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;
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
    let context: Context = Context {
        action: Action::Overview,
        last_action_time: Utc::now(),
        hero: Personnage {
            nom: "Lyrocs".to_string(),
            classe: "Novice".to_string(),
            hp: 100,
            max_hp: 100,
            mp: 100,
            max_mp: 100,
            experience: 0,
            niveau: 1,
            inventaire: vec!["Épée".to_string(), "Arc".to_string(), "Herbes".to_string()],
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
            hp: 0,
            max_hp: 0,
            mp: 0,
            max_mp: 0,
        },
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
