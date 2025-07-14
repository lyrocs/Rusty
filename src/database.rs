use crate::models::context::Context;
use crate::models::hero::Personnage;
use crate::models::hero::Skill;
use anyhow::Result;
use chrono::prelude::*;
use redb::{Database, TableDefinition};

const PERSONNAGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("personnages");
const CONTEXT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("context");

pub fn init_db(db: &Database) -> Result<()> {
    let read_txn = db.begin_read()?;
    let _table = match read_txn.open_table(PERSONNAGES_TABLE) {
        Ok(table) => table,
        Err(_) => {
            init_db_data(&db)?;
            return Ok(());
        }
    };

    Ok(())
}

pub fn init_db_data(db: &Database) -> Result<()> {
    let hero: Personnage = Personnage {
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
    };
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(PERSONNAGES_TABLE)?;

        // On convertit notre objet `hero` en bytes
        let hero_bytes = serde_json::to_vec(&hero)?;
        // On stocke les bytes dans la DB
        table.insert(hero.nom.as_str(), hero_bytes.as_slice())?;
        println!(
            "\n'{}' a été sérialisé et sauvegardé dans la base de données.",
            hero.nom
        );
    }
    write_txn.commit()?;

    let context: Context = Context {
        action: "overview".to_string(),
        last_action_time: Utc::now(),
    };
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(CONTEXT_TABLE)?;

        // On convertit notre objet `hero` en bytes
        let hero_bytes = serde_json::to_vec(&context)?;
        // On stocke les bytes dans la DB
        table.insert("context", hero_bytes.as_slice())?;
        println!(
            "\n'{}' a été sérialisé et sauvegardé dans la base de données.",
            "context"
        );
    }
    write_txn.commit()?;

    Ok(())
}

pub fn get_hero(db: &Database) -> Result<Personnage> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(PERSONNAGES_TABLE)?;
    if let Some(personnage_data) = table.get("Lyrocs")? {
        let personnage_bytes = personnage_data.value();
        let personnage_recupere: Personnage = serde_json::from_slice(personnage_bytes)?;
        Ok(personnage_recupere)
    } else {
        Err(anyhow::anyhow!("Personnage non trouvé"))
    }
}

pub fn update_hero(db: &Database, hero: Personnage) -> Result<Personnage> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(PERSONNAGES_TABLE)?;
    if let Some(personnage_data) = table.get("Lyrocs")? {
        let personnage_bytes = personnage_data.value();
        let personnage_recupere: Personnage = serde_json::from_slice(personnage_bytes)?;
        Ok(personnage_recupere)
    } else {
        Err(anyhow::anyhow!("Personnage non trouvé"))
    }
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

            // On convertit notre objet `hero` en bytes
            let hero_bytes = serde_json::to_vec(&context)?;
            // On stocke les bytes dans la DB
            table.insert("context", hero_bytes.as_slice())?;
            println!(
                "\n'{}' a été sérialisé et sauvegardé dans la base de données.",
                "context"
            );
        }
        write_txn.commit()?;
        Ok(context_recupere)
    } else {
        Err(anyhow::anyhow!("Context non trouvé"))
    }
}
