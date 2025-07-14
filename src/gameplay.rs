use redb::Database;
use crate::models::context::Context;
use crate::models::hero::Personnage;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;
use crate::database::update_context;
use serde::Deserialize;
use std::fs;
use chrono::prelude::*;
use anyhow::Result;

pub fn handle_action(action_name: String,
    db: &Database,
    context: &mut Context) {
     
        
        if context.action == "overview" {
            match action_name.as_str() {
                "action_1" => {
                    context.action = "battle".to_string();
                    context.battle = Some(create_battle());
                    context.enemy = Some(create_enemy().unwrap());
                    // update_context(db, context.clone()).unwrap();
                },
                "action_2" => println!("Action 2"),
                _ => println!("Action non trouvée"),
            }
            return;
        }
        let battle = match context.battle.as_mut() {
            Some(b) => b,
            None => return,
        };
        let enemy = match context.enemy.as_mut() {
            Some(e) => e,
            None => return,
        };
        // let mut battle;
        // if let Some(existing_battle) = context.battle.as_mut() {
        //    battle = existing_battle;
        // } else {
        //     return;
        // }
        // let mut enemy;
        // if let Some(existing_enemy) = context.enemy.as_mut() {
        //   enemy = existing_enemy;
        // } else {
        //     return;
        // }
        if context.action == "battle" && battle.turn == "hero" { 
            // let battle = context.battle.as_mut().unwrap();
            // let enemy = context.enemy.as_mut().unwrap();
            match action_name.as_str() {
                "action_1" => {
                     enemy.hp -= 5;
                     battle.turn = "enemy".to_string();
                     battle.message = "You're attacking !".to_string();
                     context.last_action_time = Utc::now();
                },
                "action_2" => {
                    context.action = "battle_spell".to_string();
                    
                    // enemy.hp -= 25;
                    // battle.turn = "enemy".to_string();
                    // battle.message = "You're spelling !".to_string();
                    // context.last_action_time = Utc::now();
                },
                _ => println!("Action non trouvée"),
            }
        }
        if context.action == "battle_spell" {
            match action_name.as_str() {
                "Back" => {
                    context.action = "battle".to_string();
                    // update_context(db, context).unwrap();
                },
                "skill_1" => {
                    enemy.hp -= 5;
                    battle.turn = "enemy".to_string();
                    battle.message = "Using skill 1 !".to_string();
                    context.last_action_time = Utc::now();
                    context.action = "battle".to_string();
                },
                "skill_2" => {
                    enemy.hp -= 10;
                    battle.turn = "enemy".to_string();
                    battle.message = "Using skill 2 !".to_string();
                    context.last_action_time = Utc::now();
                    context.action = "battle".to_string();
                },
                "skill_3" => {
                    enemy.hp -= 15;
                    battle.turn = "enemy".to_string();
                    battle.message = "Using skill 3 !".to_string();
                    context.last_action_time = Utc::now();
                    context.action = "battle".to_string();
                },
                "skill_4" => {
                    enemy.hp -= 20;
                    battle.turn = "enemy".to_string();
                    battle.message = "Using skill 4 !".to_string();
                    context.last_action_time = Utc::now();
                    context.action = "battle".to_string();
                },
                _ => println!("Action non trouvée"),
            }
        }
}

pub fn handle_action_routine(
    context: &mut Context,
) {
    let battle = match context.battle.as_mut() {
        Some(b) => b,
        None => return,
    };
    let enemy = match context.enemy.as_mut() {
        Some(e) => e,
        None => return,
    };
    if context.action == "battle" {
        if battle.turn == "" {
            *battle = create_battle();
            *enemy = create_enemy().unwrap();
        } else if battle.turn == "hero" {
            // enemy.hp -= 5;
            // battle.turn = "enemy".to_string();
            // battle.message = "You're attacking !".to_string();
        } else if battle.turn == "enemy" {
            context.hero.hp -= 2;
            battle.turn = "hero".to_string();
            battle.message = "Enemy is attacking !".to_string();
        }

        // TODO
        println!("Loop battle");
    }
}

fn create_battle() -> Battle {
    Battle {
        turn: "hero".to_string(),
        status: "ongoing".to_string(),
        message: "Enemy is appearing !".to_string(),
    }
}


#[derive(Deserialize, Debug)]
pub struct EnemyJSON {
    pub name: String,
    // On pourrait ajouter les objets, etc.
}

fn create_enemy() -> Result<Enemy> {
    let json_contenu = fs::read_to_string("data/enemies.json")?;
    let game_data: EnemyJSON = serde_json::from_str(&json_contenu)?;

    Ok(Enemy {
        name: game_data.name,
        hp: 50,
        max_hp: 50,
        mp: 50,
        max_mp: 50,
    })
    // Enemy {
    //     name: "Poring".to_string(),
    //     hp: 50,
    //     max_hp: 50,
    //     mp: 50,
    //     max_mp: 50,
    // }
}