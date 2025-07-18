
mod gameplay;
use gameplay::handle_action;
use gameplay::handle_action_routine;
mod game_data;
mod models;
use models::context::Context;
use models::context::CTA;
use models::eink::Eink;
mod database;
use database::get_context;
use database::init_db;
use database::update_context;
mod eink;
use eink::GTDev;
use eink::GTOld;
use eink::gt_scan;
use eink::init_eink;
use anyhow::Result;
use std::{thread, time};
use chrono::Duration;
use chrono::prelude::*;
use redb::Database;
mod rendering;
mod ui;
use ui::generate_cta;

fn main() -> Result<()> {
    let db = Database::create("mon_rpg.redb")?;
    init_db(&db)?;
    let mut context = get_context(&db)?;
    let mut eink: Eink = init_eink();
    let mut gt_dev = GTDev {
        touchpoint_flag: 0,
        touch_count: 0,
        x: [0; 5],
        y: [0; 5],
        s: [0; 5],
        touchkeytrackid: [0; 5],
    };
    let mut gt_old = GTOld {
        x: [0; 5],
        y: [0; 5],
        s: [0; 5],
    };
    loop {
        let (x, y) = gt_scan(&mut eink.i2c, &mut gt_dev, &mut gt_old)?;
        if x != 0 && y != 0 {
            let cta = handle_touch(122 - x as i32, 250 - y as i32, &mut context);
            if cta.is_ok() {
                handle_action(cta.unwrap(), &mut context);
                rendering::render(
                    &mut eink,
                    &mut context,
                );
            }
        }
        let two_seconds_from_now: DateTime<Utc> = Utc::now() - Duration::seconds(2);
        if two_seconds_from_now > context.last_action_time {
            context.last_action_time = Utc::now();
            let _ = update_context(&db, context.clone());
            handle_action_routine(&mut context);
            rendering::render(
                &mut eink,
                &mut context
            );
        }
        thread::sleep(time::Duration::from_millis(200));
    }
}

fn handle_touch(x: i32, y: i32, context: &mut Context) -> Result<CTA> {
    let cta = generate_cta(&context);
    for cta in cta.iter() {
        if x >= cta.x && x <= cta.x + cta.width && y >= cta.y && y <= cta.y + cta.height {
            context.needs_redraw = true;
            return Ok(cta.clone());
        }
    }
    Err(anyhow::anyhow!("Action non trouvée"))
}

    // match &context.activity {
    //     Activity::ManualCombat(ManualCombatState::SelectingSkill) => {
    //         if y > 220 {
    //             Ok("Back".to_string())
    //         } else if y > 190 {
    //             Ok("skill_4".to_string())
    //         } else if y > 160 {
    //             Ok("skill_3".to_string())
    //         } else if y > 130 {
    //             Ok("skill_2".to_string())
    //         } else if y > 100 {
    //             Ok("skill_1".to_string())
    //         } else {
    //             Err(anyhow::anyhow!("Action non trouvée"))
    //         }
    //     }
    //     Activity::ManualCombat(ManualCombatState::Overview) => {
    //         if x < 60 && y > 200 {
    //             Ok("action_1".to_string())
    //         } else if x > 60 && y > 200 {
    //             Ok("action_2".to_string())
    //         } else {
    //             Err(anyhow::anyhow!("Action non trouvée"))
    //         }
    //     }
    //     Activity::ManualCombat(ManualCombatState::Result { rewards }) => {
    //         if y > 220 {
    //             Ok("Back".to_string())
    //         } else {
    //             Err(anyhow::anyhow!("Action non trouvée"))
    //         }
    //     }
    //     Activity::BrowseLocation => {
    //         if y > 220 {
    //             if x < 60 {
    //                 Ok("Fight".to_string())
    //             } else {
    //                 Ok("Menu".to_string())
    //             }
    //         } else if y > 190 {
    //             Ok("action_1".to_string())
    //         } else if y > 160 {
    //             Ok("action_2".to_string())
    //         } else if y > 130 {
    //             Ok("action_3".to_string())
    //         } else if y > 100 {
    //             Ok("action_4".to_string())
    //         } else {
    //             Err(anyhow::anyhow!("Action non trouvée"))
    //         }
    //     }
    //     Activity::HeroOverview => {
    //         if x < 60 && y > 200 {
    //             Ok("action_1".to_string())
    //         } else if x > 60 && y > 200 {
    //             Ok("action_2".to_string())
    //         } else {
    //             Err(anyhow::anyhow!("Action non trouvée"))
    //         }
    //     }
    //     _ => {
    //         Err(anyhow::anyhow!("Action non trouvée"))
    //     }
    // }
// }
