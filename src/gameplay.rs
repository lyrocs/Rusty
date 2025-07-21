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
                apply_damage(context, 1.0);
            },
            Action::SkillList => {
                context.activity = Activity::ManualCombat(ManualCombatState::SelectingSkill);
            },
            Action::Skill => {
                apply_damage(context, 1.2);
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


fn apply_damage(context: &mut Context, modifier: f32) {
    let (hero_atk, hero_def) = calculate_hero_stats(context);
    let damage = calculate_damage(context.hero.base_level as i32, hero_atk as i32, context.enemy.level as i32, context.enemy.defense as i32);
    let damage = (damage as f32 * modifier).round() as u32;
    if context.enemy.hp > damage as u32 {
        context.enemy.hp -= damage as u32;
        context.battle.turn = "enemy".to_string();
        
        context.battle.message = format!("You deal {} damage !", damage.to_string());
        context.last_action_time = Utc::now();
        if matches!(context.activity, Activity::ManualCombat(_)) {
            context.activity = Activity::ManualCombat(ManualCombatState::Overview);
        }
    } else {
        context.enemy.hp = 0;
        context.battle.status = "victory".to_string();
        context.battle.message = "You won !".to_string();
        context.battle.turn = "hero".to_string();
       
        // add reward into inventory
        // handle_victory
        handle_victory(context);
    }
}

fn apply_hero_damage(context: &mut Context) {
    let (hero_atk, hero_def) = calculate_hero_stats(context);
    let damage = calculate_damage(context.hero.base_level as i32, hero_atk as i32, context.hero.base_level as i32, hero_def as i32);
    if context.hero.hp > damage as u32 {
        context.hero.hp -= damage as u32;
        context.battle.turn = "hero".to_string();
        context.battle.message = format!("Enemy deals {} damage !", damage.to_string());
        context.needs_redraw = true;
    } else {
        context.hero.hp = context.hero.max_hp;
        context.battle.status = "dead".to_string();
        context.battle.message = "You're dead !".to_string();
        if matches!(context.activity, Activity::ManualCombat(_)) { 
            context.activity = Activity::BrowseLocation;
        } else if matches!(context.activity, Activity::AutoCombat(_)) { 
            context.activity = Activity::AutoCombat(AutoCombatState::Dead { end_time:  Utc::now() + Duration::seconds(60) });
        }
    }
}

pub fn handle_victory(
    context: &mut Context,
) {
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
    if matches!(context.activity, Activity::ManualCombat(_)) { 
        context.activity = Activity::ManualCombat(ManualCombatState::Result { rewards });
    } else if matches!(context.activity, Activity::AutoCombat(_)) { 
        context.activity = Activity::AutoCombat(AutoCombatState::Searching { end_time:  Utc::now() + Duration::seconds(5) });
    }

    context.hero.base_exp += context.enemy.base_exp;
    if context.hero.base_exp >= context.hero.base_exp_next {
        context.hero.base_exp -= context.hero.base_exp_next;
        context.hero.base_level += 1;
        let base_exp_table = game_data::get_base_exp().unwrap();
        let exp = base_exp_table.iter().find(|exp| exp.level == context.hero.base_level).unwrap();
        context.hero.base_exp_next = exp.exp;
        // Earn HP
        const BASE_HP: f32 = 40.0;
        const GAIN_HP_PER_LEVEL: f32 = 25.0;
        let total_hp = BASE_HP + ((context.hero.base_level - 1) as f32 * GAIN_HP_PER_LEVEL);
        context.hero.max_hp = total_hp.floor() as u32;
        context.hero.hp = context.hero.max_hp;
    }
    if context.hero.job_level < 10 {
        context.hero.job_exp += context.enemy.job_exp;
        if context.hero.job_exp >= context.hero.job_exp_next {
            context.hero.job_exp -= context.hero.job_exp_next;
            context.hero.job_level += 1;
            let job_exp_table = game_data::get_novice_exp().unwrap();
            let exp = job_exp_table.iter().find(|exp| exp.level == context.hero.job_level).unwrap();
            context.hero.job_exp_next = exp.exp;
        }
    }

}

fn calculate_hero_stats(context: &Context) -> (f32, f32) {
    let level = context.hero.base_level as f32;
    let base_atk = 35.0;
    let base_def = 50.0;
    const GAIN_ATK_PER_LEVEL: f32 = 2.0;
    const GAIN_DEF_PER_LEVEL: f32 = 2.0;
    let total_atk = (level * GAIN_ATK_PER_LEVEL + base_atk + (level / 5.0)).floor();
    let total_def = (level * GAIN_DEF_PER_LEVEL + base_def + (level / 5.0)).floor();
    (total_atk, total_def)
}

// La fonction de calcul de dégâts
fn calculate_damage(attacker_level: i32, attacker_base_atk: i32, defender_level: i32, defender_base_def: i32) -> i32 {
    // Calcul de l'ATK totale de l'attaquant
    // let total_atk = (attacker_level as f32 * 2.5 + attacker_base_atk as f32).floor();

    let damage_brut = attacker_base_atk as f32;

    // Calcul de la DEF totale du défenseur
    // let total_def = (defender_level as f32 * 1.5 + defender_base_def as f32).floor();

    // La constante d'équilibrage. Vous pouvez l'ajuster.
    // Une valeur plus élevée rend la défense moins efficace.
    const K: f32 = 50.0;

    let reduction = defender_base_def as f32 / (defender_base_def as f32 + K);
    
    let damage = damage_brut * (1.0 - reduction);

    // Les dégâts ne peuvent pas être inférieurs à 1
    if damage < 1.0 {
        1
    } else {
        damage as i32
    }
}


pub fn handle_action_routine(
    context: &mut Context,
) {
    match &context.activity {
        Activity::AutoCombat(AutoCombatState::Searching { end_time }) => {
            if Utc::now() > *end_time {
                context.activity = Activity::AutoCombat(AutoCombatState::Fighting);
                context.battle = create_battle();
                context.enemy = create_enemy(&context.location.enemies).unwrap();
                context.needs_redraw = true;
            }
        },
        Activity::AutoCombat(AutoCombatState::Fighting) => {
            if context.battle.turn == "enemy" {
                apply_hero_damage(context);
            } else if context.battle.turn == "hero" {
                const DAMAGE: u32 = 20;
                apply_damage(context, 1.0);
            }
            context.needs_redraw = true;
        },
        Activity::AutoCombat(AutoCombatState::Dead { end_time }) => {
            if Utc::now() > *end_time {
                context.activity = Activity::AutoCombat(AutoCombatState::Searching { end_time:  Utc::now() + Duration::seconds(5) });
                context.needs_redraw = true;
            }
        },
        Activity::ManualCombat(ManualCombatState::Overview) => {
            if context.battle.turn == "enemy" {
                apply_hero_damage(context);
            }
            context.needs_redraw = true;
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