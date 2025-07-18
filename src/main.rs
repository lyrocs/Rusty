
mod gameplay;
use gameplay::handle_action;
use gameplay::handle_action_routine;
use gameplay::handle_touch;
mod game_data;
mod models;
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



