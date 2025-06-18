use text_io::read;
use redb::{Database, Error, TableDefinition, ReadableTable};
use serde::{Serialize, Deserialize};
use serde_json; // On importe serde_json

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Personnage {
    nom: String,
    classe: String,
    points_de_vie: u32,
    niveau: u8,
    inventaire: Vec<String>,
}


const PERSONNAGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("personnages");

fn main() -> Result<(), Box<dyn std::error::Error>> {


     let db = Database::create("mon_rpg.redb")?;

    // 1. CRÉATION DE NOTRE PERSONNAGE
    let hero: Personnage = Personnage {
        nom: "Aragorn".to_string(),
        classe: "Rôdeur".to_string(),
        points_de_vie: 100,
        niveau: 5,
        inventaire: vec!["Épée".to_string(), "Arc".to_string(), "Herbes".to_string()],
    };
    println!("Personnage original : {:?}", hero);

    // 2. SÉRIALISATION & STOCKAGE
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(PERSONNAGES_TABLE)?;
        
        // On convertit notre objet `hero` en bytes
        let hero_bytes = serde_json::to_vec(&hero)?;
        // On stocke les bytes dans la DB
        table.insert(hero.nom.as_str(), hero_bytes.as_slice())?;
        println!("\n'{}' a été sérialisé et sauvegardé dans la base de données.", hero.nom);
    }
    write_txn.commit()?;


    println!("Set new name:");
    let name: String = read!();
    println!("You entered: {}", name);

    // update personname nom from name
    let mut write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(PERSONNAGES_TABLE)?;
        
        let mut newHero = hero.clone();
         // On change le nom du personnage
         newHero.nom = name.clone();
         // On reconvertit en bytes
         let hero_bytes = serde_json::to_vec(&newHero)?;
          // On met à jour la D
        table.insert(hero.nom.as_str(), hero_bytes.as_slice())?;
        println!("\nLe nom du personnage a été mis à jour dans la base de données.");

    }
    write_txn.commit()?;

    // 3. RÉCUPÉRATION & DÉSÉRIALISATION
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(PERSONNAGES_TABLE)?;

    println!("\nRécupération depuis la base de données...");
    if let Some(personnage_data) = table.get("Aragorn")? {
        let personnage_bytes = personnage_data.value();
        
        // On reconvertit les bytes en notre objet Personnage
        let personnage_recupere: Personnage = serde_json::from_slice(personnage_bytes)?;

        println!("Personnage récupéré : {:?}", personnage_recupere);
        
        // On vérifie que les données sont identiques !
        assert_eq!(hero, personnage_recupere);
        println!("\nSuccès ! Les données sont identiques.");
    }

    Ok(())



}
