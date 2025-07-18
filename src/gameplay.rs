use crate::models::context::Context;
use crate::models::context::ManualCombatState;
use crate::models::context::Activity;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;
use rand::seq::SliceRandom;
use rand::Rng;
use crate::models::context::AutoCombatState;
use crate::models::context::EnemyShort;
use crate::models::context::LootItem;
use serde::Deserialize;
use std::fs;
use crate::models::context::CTA;
use chrono::prelude::*;
use anyhow::Result;
use chrono::Duration;
use crate::game_data;

pub fn handle_action(cta: CTA,
    context: &mut Context) {
     
        match cta.action.as_str() {
            "map" => {
                context.activity = Activity::BrowseLocation;
            },
            "inventory" => {
                context.activity = Activity::HeroOverview;
            },
            "fight_manual" => {
                context.activity = Activity::ManualCombat(ManualCombatState::Overview);
                context.battle = create_battle();
                context.enemy = create_enemy(&context.location.enemies).unwrap();
            },
            "fight_auto" => {
                context.activity = Activity::AutoCombat(AutoCombatState::Searching { end_time:  Utc::now() + Duration::seconds(5) });
            },
            "back" => {
                context.activity = Activity::BrowseLocation;
            },
            "back_manual_overview" => {
                context.activity = Activity::ManualCombat(ManualCombatState::Overview);
            },
            "connection" => {
                let maps = game_data::get_locations().unwrap();
                let location = context.location.connections.iter().find(|connection| connection.target_id == cta.id.unwrap() as u32).unwrap();
                context.location = maps.iter().find(|map| map.id == location.target_id).unwrap().clone();
                context.activity = Activity::BrowseLocation;
            },
            "attack" => {
                apply_damage(context, 5);
            },
            "spell" => {
                context.activity = Activity::ManualCombat(ManualCombatState::SelectingSkill);
            },
            "skill" => {
                apply_damage(context, 5);
            },
            _ => println!("Action non trouvée"),
        }
        // match &context.activity {
        //     Activity::HeroOverview => {
        //         match action_name.as_str() {
        //             "action_1" => {
        //                 context.activity = Activity::BrowseLocation;
        //                 // update_context(db, context.clone()).unwrap();
        //             },
        //             "action_2" => {
        //                 // context.action = Action::BattleAuto;
        //                 // context.activity = Activity::AutoCombat(AutoCombatState::Searching { end_time:  Utc::now() + Duration::seconds(5) });
        //                 // context.battle = create_battle();
        //                 // context.enemy = create_enemy().unwrap();
        //             },
        //             _ => println!("Action non trouvée"),
        //         }
        //         return;
        //     },
        //     Activity::ManualCombat(ManualCombatState::Overview) => {
        //         match action_name.as_str() {
        //             "action_1" => {
        //                 apply_damage(context, 5);
        //             },
        //             "action_2" => {
        //                 context.activity = Activity::ManualCombat(ManualCombatState::SelectingSkill);
        //             },
        //             _ => println!("Action non trouvée"),
        //         }
        //         return;
        //     },
        //     Activity::ManualCombat(ManualCombatState::SelectingSkill) => {
        //         match action_name.as_str() {
        //             "Back" => {
        //                 context.activity = Activity::ManualCombat(ManualCombatState::Overview);
        //             },
        //             "skill_1" => {
        //                 apply_damage(context, 5);
        //             },
        //             "skill_2" => {
        //                 apply_damage(context, 10);
        //             },
        //             "skill_3" => {
        //                 apply_damage(context, 15);
        //             },
        //             "skill_4" => {
        //                 apply_damage(context, 100);
        //             },
        //             _ => println!("Action non trouvée"),
        //         }
        //         return;
        //     },
        //     Activity::ManualCombat(ManualCombatState::Result { rewards: _ }) => {
        //         match action_name.as_str() {
        //             "Back" => {
        //                 context.activity = Activity::BrowseLocation;
        //             },
        //             _ => println!("Action non trouvée"),
        //         }
        //         return;
        //     },
        //     Activity::BrowseLocation => {
        //         match action_name.as_str() {
        //             "action_1" => {
        //                 if context.location.connections.len() > 0 {
        //                     let maps = game_data::get_locations().unwrap();
        //                     let target_location = maps.iter().find(|map| map.id == context.location.connections[0].target_id).unwrap();
        //                     context.location = target_location.clone();
        //                 }
        //             },
        //             "Menu" => {
        //                 context.activity = Activity::HeroOverview;
        //             },
        //             "Fight" => {
        //                 context.activity = Activity::ManualCombat(ManualCombatState::Overview);
        //                 context.battle = create_battle();
        //                 context.enemy = create_enemy(&context.location.enemies).unwrap();
        //             },
        //             _ => println!("Action non trouvée"),
        //         }
        //         return;
        //     },
        //     _ => println!("Activity non trouvée"),
        // }
        
        // if context.action == Action::Overview {
        //     match action_name.as_str() {
        //         "action_1" => {
        //             context.action = Action::Battle;
        //             context.battle = create_battle();
        //             context.enemy = create_enemy().unwrap();
        //             // update_context(db, context.clone()).unwrap();
        //         },
        //         "action_2" => {
        //             // context.action = Action::BattleAuto;
        //             context.activity = Activity::AutoCombat(AutoCombatState::Searching { end_time:  Utc::now() + Duration::seconds(5) });
        //         },
        //         _ => println!("Action non trouvée"),
        //     }
        //     return;
        // }

        // if context.action == Action::Battle && context.battle.turn == "hero" { 
        //     match action_name.as_str() {
        //         "action_1" => {
        //              apply_damage(context, 5);
        //         },
        //         "action_2" => {
        //             context.action = Action::BattleSpell;
        //         },
        //         _ => println!("Action non trouvée"),
        //     }
        // }
        // if context.action == Action::BattleSpell {
        //     match action_name.as_str() {
        //         "Back" => {
        //             context.action = Action::Battle;
        //             // update_context(db, context).unwrap();
        //         },
        //         "skill_1" => {
        //             apply_damage(context, 5);
        //         },
        //         "skill_2" => {
        //             apply_damage(context, 10);
        //         },
        //         "skill_3" => {
        //             apply_damage(context, 15);
        //         },
        //         "skill_4" => {
        //             apply_damage(context, 20);
        //         },
        //         _ => println!("Action non trouvée"),
        //     }
        // }
}

fn apply_damage(context: &mut Context, damage: u32) {
    if context.enemy.hp > damage {
        context.enemy.hp -= damage;
        context.battle.turn = "enemy".to_string();
        
        context.battle.message = format!("You deal {} damage !", damage.to_string());
        context.last_action_time = Utc::now();
        context.activity = Activity::ManualCombat(ManualCombatState::Overview);
        // context.action = Action::Battle;
    } else {
        context.enemy.hp = 0;
        context.battle.status = "victory".to_string();
        context.battle.message = "You won !".to_string();
        context.battle.turn = "hero".to_string();
        let mut rewards = Vec::new();
        for drop in context.enemy.drops.iter() {
            // get reward based on drop rate
            let random_number = rand::thread_rng().gen_range(0..1000);
            if random_number < (drop.chance as u32 * 10) {
                rewards.push(LootItem {
                    id: drop.id,
                    name: drop.item.clone(),
                    quantity: 1,
                });
            }  
        }
        // add reward into inventory
        for reward in rewards.iter() {
            // check if item already exists in inventory
            let mut item_exists = false;
            for item in context.hero.inventaire.iter_mut() {
                if item.id == reward.id {
                    item_exists = true;
                    item.quantity += reward.quantity;
                    break;
                }
            }
            if !item_exists {
                context.hero.inventaire.push(reward.clone());
            }
        }
        
        context.activity = Activity::ManualCombat(ManualCombatState::Result { rewards });
    }
}
    


pub fn handle_action_routine(
    context: &mut Context,
) {
    match &context.activity {
        Activity::AutoCombat(AutoCombatState::Searching { end_time }) => {
            if Utc::now() > *end_time {
                context.activity = Activity::AutoCombat(AutoCombatState::Fighting);
                context.needs_redraw = true;
            }
        }
        Activity::ManualCombat(ManualCombatState::Overview) => {
            if context.battle.turn == "enemy" {
                const DAMAGE: u32 = 2;
                context.battle.turn = "hero".to_string();
                context.needs_redraw = true;
                if context.hero.hp > DAMAGE {
                    context.hero.hp -= DAMAGE;
                    context.battle.message = "Enemy is attacking !".to_string();
                } else {
                    context.hero.hp = 0;
                    context.battle.status = "dead".to_string();
                    context.battle.message = "You're dead !".to_string();
                }
            }
        }   
        _ => {}
    }
    // if context.action == Action::Battle {
    //     if context.battle.turn == "" {
    //         context.battle = create_battle();
    //         context.enemy = create_enemy().unwrap();
    //     } else if context.battle.turn == "hero" {
    //         // enemy.hp -= 5;
    //         // battle.turn = "enemy".to_string();
    //         // battle.message = "You're attacking !".to_string();
    //     } else if context.battle.turn == "enemy" {
    //         const DAMAGE: u32 = 2;
    //         if context.hero.hp > DAMAGE {
    //             context.hero.hp -= DAMAGE;
    //             context.battle.turn = "hero".to_string();
    //             context.battle.message = "Enemy is attacking !".to_string();
    //         } else {
    //             context.hero.hp = 0;
    //             context.battle.turn = "hero".to_string();
    //             context.battle.status = "dead".to_string();
    //             context.battle.message = "You're dead !".to_string();
    //         }
    //     }

    //     // TODO
    //     println!("Loop battle");
    // }
}

fn create_battle() -> Battle {
    Battle {
        turn: "hero".to_string(),
        status: "ongoing".to_string(),
        message: "Enemy is appearing !".to_string(),
    }
}



fn create_enemy(enemies: &Vec<EnemyShort>) -> Result<Enemy> {
    let random_enemy = enemies.choose(&mut rand::thread_rng()).unwrap();
    let enemies = game_data::get_enemies().unwrap();
    let enemy = enemies.iter().find(|enemy| enemy.id == random_enemy.id).unwrap();

    Ok(enemy.clone())
    // Enemy {
    //     name: "Poring".to_string(),
    //     hp: 50,
    //     max_hp: 50,
    //     mp: 50,
    //     max_mp: 50,
    // }
}