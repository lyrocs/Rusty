use crate::models::context::Context;
use crate::models::context::ManualCombatState;
use crate::models::context::Activity;
use crate::models::battle::Battle;
use crate::models::enemy::Enemy;
use crate::models::context::AutoCombatState;
use serde::Deserialize;
use std::fs;
use chrono::prelude::*;
use anyhow::Result;
use chrono::Duration;

pub fn handle_action(action_name: String,
    context: &mut Context) {
     
        match &context.activity {
            Activity::HeroOverview => {
                match action_name.as_str() {
                    "action_1" => {
                        context.activity = Activity::BrowseLocation;
                        // update_context(db, context.clone()).unwrap();
                    },
                    "action_2" => {
                        // context.action = Action::BattleAuto;
                        context.activity = Activity::AutoCombat(AutoCombatState::Searching { end_time:  Utc::now() + Duration::seconds(5) });
                        context.battle = create_battle();
                        context.enemy = create_enemy().unwrap();
                    },
                    _ => println!("Action non trouvée"),
                }
                return;
            },
            Activity::ManualCombat(ManualCombatState::Overview) => {
                match action_name.as_str() {
                    "action_1" => {
                        apply_damage(context, 5);
                    },
                    "action_2" => {
                        context.activity = Activity::ManualCombat(ManualCombatState::SelectingSkill);
                    },
                    _ => println!("Action non trouvée"),
                }
                return;
            },
            Activity::ManualCombat(ManualCombatState::SelectingSkill) => {
                match action_name.as_str() {
                    "Back" => {
                        context.activity = Activity::ManualCombat(ManualCombatState::Overview);
                    },
                    "skill_1" => {
                        apply_damage(context, 5);
                    },
                    "skill_2" => {
                        apply_damage(context, 10);
                    },
                    "skill_3" => {
                        apply_damage(context, 15);
                    },
                    "skill_4" => {
                        apply_damage(context, 100);
                    },
                    _ => println!("Action non trouvée"),
                }
                return;
            },
            Activity::ManualCombat(ManualCombatState::Result { rewards: _ }) => {
                match action_name.as_str() {
                    "Back" => {
                        context.activity = Activity::BrowseLocation;
                    },
                    _ => println!("Action non trouvée"),
                }
                return;
            },
            Activity::BrowseLocation => {
                match action_name.as_str() {
                    "Fight" => {
                        context.activity = Activity::ManualCombat(ManualCombatState::Overview);
                        context.battle = create_battle();
                        context.enemy = create_enemy().unwrap();
                    },
                    _ => println!("Action non trouvée"),
                }
                return;
            },
            _ => println!("Activity non trouvée"),
        }
        
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
        context.activity = Activity::ManualCombat(ManualCombatState::Result { rewards: Vec::new() });
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



fn create_enemy() -> Result<Enemy> {
    let json_contenu = fs::read_to_string("data/enemies.json")?;
    let game_data: Enemy = serde_json::from_str(&json_contenu)?;

    Ok(game_data)
    // Enemy {
    //     name: "Poring".to_string(),
    //     hp: 50,
    //     max_hp: 50,
    //     mp: 50,
    //     max_mp: 50,
    // }
}