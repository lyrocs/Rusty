

use crate::models::context::Context;
use crate::models::context::Activity;
use crate::models::context::ManualCombatState;
use crate::models::context::CTA;
use crate::models::context::Action;

const SCREEN_WIDTH: i32 = 122;
const HALF_SCREEN_WIDTH: i32 = SCREEN_WIDTH / 2;
const CTA_HEIGHT: i32 = 30;
const CTA_ZONE_1: i32 = 220;
const CTA_ZONE_2: i32 = CTA_ZONE_1 - CTA_HEIGHT;
const CTA_ZONE_3: i32 = CTA_ZONE_2 - CTA_HEIGHT;


pub fn generate_cta(context: &Context) -> Vec<CTA> {
    let mut cta = Vec::new();
    match &context.activity {
        Activity::BrowseLocation => {
            let mut y = CTA_ZONE_1;
            if context.location.enemies.len() > 0 {
                cta.push(CTA {
                    label: "Fight".to_string(),
                    action: Action::FightManual,
                    id: None,
                    x: 0,
                    y: y,
                    width: HALF_SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
                cta.push(CTA {
                    label: "Auto".to_string(),
                    action: Action::FightAuto,
                    id: None,
                    x: HALF_SCREEN_WIDTH,
                    y: y,
                    width: HALF_SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
                y -= CTA_HEIGHT;
                cta.push(CTA {
                    label: "Menu".to_string(),
                    action: Action::HeroOverview,
                    id: None,
                    x: 0,
                    y: y,
                    width: SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
                y -= CTA_HEIGHT;
            } else {
                cta.push(CTA {
                    label: "Menu".to_string(),
                    action: Action::HeroOverview,
                    id: None,
                    x: 0,
                    y: y,
                    width: SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
                y -= CTA_HEIGHT;
            }
            for connection in context.location.connections.iter() {
                cta.push(CTA {
                    label: connection.label.clone(),
                    action: Action::Wrap,
                    id: Some(connection.target_id.clone() as i32),
                    x: 0,
                    y: y,
                    width: SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
                y -= CTA_HEIGHT;
            }
        }
        Activity::HeroOverview => {
            cta.push(CTA {
                label: "Map".to_string(),
                action: Action::Map,
                id: None,
                x: 0,
                y: CTA_ZONE_1,
                width: HALF_SCREEN_WIDTH,
                height: CTA_HEIGHT,
            });
            cta.push(CTA {
                label: "Inventory".to_string(),
                action: Action::Inventory,
                id: None,
                x: HALF_SCREEN_WIDTH,
                y: CTA_ZONE_1,
                width: HALF_SCREEN_WIDTH,
                height: CTA_HEIGHT,
            });
        }
        Activity::ManualCombat(ManualCombatState::Overview) => {
            if context.battle.turn == "hero" {
                cta.push(CTA {
                    label: "Fight".to_string(),
                    action: Action::Attack,
                    id: None,
                    x: 0,
                    y: CTA_ZONE_1,
                    width: HALF_SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
                cta.push(CTA {
                    label: "Spell".to_string(),
                    action: Action::SkillList,
                    id: None,
                    x: HALF_SCREEN_WIDTH,
                    y: CTA_ZONE_1,
                    width: HALF_SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
            }
        }
        Activity::ManualCombat(ManualCombatState::SelectingSkill) => {
            cta.push(CTA {
                label: "Back".to_string(),
                action: Action::BackManualFight,
                id: None,
                x: 0,
                y: CTA_ZONE_1,
                width: SCREEN_WIDTH,
                height: CTA_HEIGHT,
            });
            let mut y = CTA_ZONE_2;
            for skill in context.hero.skills.iter() {
                cta.push(CTA {
                    label: skill.name.clone(),
                    action: Action::Skill,
                    id: Some(skill.id.clone() as i32),
                    x: 0,
                    y: y,
                    width: SCREEN_WIDTH,
                    height: CTA_HEIGHT,
                });
                y -= CTA_HEIGHT;
            }
        }
        Activity::ManualCombat(ManualCombatState::Result { rewards }) => {
            cta.push(CTA {
                label: "Back to map".to_string(),
                action: Action::BackMap,
                id: None,
                x: 0,
                y: CTA_ZONE_1,
                width: SCREEN_WIDTH,
                height: CTA_HEIGHT,
            });
        }
        Activity::AutoCombat(_) => {
            cta.push(CTA {
                label: "Stop".to_string(),
                action: Action::BackMap,
                id: None,
                x: 0,
                y: CTA_ZONE_1,
                width: SCREEN_WIDTH,
                height: CTA_HEIGHT,
            });
        }
        _ => {}
    }
    cta
}