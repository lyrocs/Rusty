use crate::models::context::Context;
use crate::models::context::ManualCombatState;
use crate::models::context::Activity;
use crate::models::hero::InventoryItem;
use crate::models::context::AutoCombatState;
use crate::models::context::Action;
use crate::models::context::CTA;
use crate::models::context::HeroOverviewState;
use chrono::prelude::*;
use anyhow::Result;
use chrono::Duration;
use crate::game_data;
use crate::ui::generate_cta;
use crate::combat;


pub fn handle_action(cta: CTA,
    context: &mut Context) {
     
        match cta.action {
            Action::Map => {
                context.activity = Activity::BrowseLocation;
            },
            Action::HeroOverview => {
                context.activity = Activity::HeroOverview(HeroOverviewState::Overview);
            },
            Action::FightManual => {
                context.activity = Activity::ManualCombat(ManualCombatState::Overview);
                context.battle = combat::create_battle();
                context.enemy = combat::create_enemy(&context.location.enemies).unwrap();
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
                combat::apply_damage(context, 1.0);
            },
            Action::SkillList => {
                context.activity = Activity::ManualCombat(ManualCombatState::SelectingSkill);
            },
            Action::Skill => {
                combat::apply_damage(context, 1.2);
            },
            Action::Inventory => {
                context.activity = Activity::HeroOverview(HeroOverviewState::Inventory);
            },
            Action::EquipmentPage => {
                context.activity = Activity::HeroOverview(HeroOverviewState::Equipment);
            },
            Action::Equip => {
                let item = context.hero.inventaire.iter().find_map(|item| {
                    match item {
                        InventoryItem::Equipment(item) => {
                            if item.id == cta.id.unwrap() as u32 {
                                Some(item)  
                            } else {
                                None
                            }
                        },
                        _ => None,
                    }
                });
                let item = item.unwrap();
                if context.hero.weapon.is_some() {
                    context.hero.weapon = None;
                } else {
                    context.hero.weapon = Some(item.clone());
                }
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


pub fn handle_action_routine(
    context: &mut Context,
) {
    match &context.activity {
        Activity::AutoCombat(AutoCombatState::Searching { end_time }) => {
            if Utc::now() > *end_time {
                context.activity = Activity::AutoCombat(AutoCombatState::Fighting);
                context.battle = combat::create_battle();
                context.enemy = combat::create_enemy(&context.location.enemies).unwrap();
                context.needs_redraw = true;
            }
        },
        Activity::AutoCombat(AutoCombatState::Fighting) => {
            if context.battle.turn == "enemy" {
                combat::apply_hero_damage(context);
            } else if context.battle.turn == "hero" {
                combat::apply_damage(context, 1.0);
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
                combat::apply_hero_damage(context);
            }
            context.needs_redraw = true;
        }   
        _ => {}
    }
}

