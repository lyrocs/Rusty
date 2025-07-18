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
use crate::models::context::Action;
use crate::models::context::CTA;
use chrono::prelude::*;
use anyhow::Result;
use chrono::Duration;
use crate::game_data;
use crate::ui::generate_cta;

pub fn handle_action(cta: CTA,
    context: &mut Context) {
     
        match cta.action {
            Action::Map => {
                context.activity = Activity::BrowseLocation;
            },
            Action::HeroOverview => {
                context.activity = Activity::HeroOverview;
            },
            Action::FightManual => {
                context.activity = Activity::ManualCombat(ManualCombatState::Overview);
                context.battle = create_battle();
                context.enemy = create_enemy(&context.location.enemies).unwrap();
            },
            Action::FightAuto => {
                context.activity = Activity::AutoCombat(AutoCombatState::Searching { end_time:  Utc::now() + Duration::seconds(5) });
            },
            Action::BackMap => {
                context.activity = Activity::BrowseLocation;
            },
            Action::BackManualFight => {
                context.activity = Activity::ManualCombat(ManualCombatState::Overview);
            },
            Action::Wrap => {
                let maps = game_data::get_locations().unwrap();
                let location = context.location.connections.iter().find(|connection| connection.target_id == cta.id.unwrap() as u32).unwrap();
                context.location = maps.iter().find(|map| map.id == location.target_id).unwrap().clone();
                context.activity = Activity::BrowseLocation;
            },
            Action::Attack => {
                apply_damage(context, 5);
            },
            Action::SkillList => {
                context.activity = Activity::ManualCombat(ManualCombatState::SelectingSkill);
            },
            Action::Skill => {
                apply_damage(context, 40);
            },
            _ => println!("Action non trouvée"),
        }
}

pub fn handle_touch(x: i32, y: i32, context: &mut Context) -> Result<CTA> {
    let cta = generate_cta(&context);
    for cta in cta.iter() {
        if x >= cta.x && x <= cta.x + cta.width && y >= cta.y && y <= cta.y + cta.height {
            context.needs_redraw = true;
            return Ok(cta.clone());
        }
    }
    Err(anyhow::anyhow!("Action non trouvée"))
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
}