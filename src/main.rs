mod render;
use render::render;

mod hero;
use hero::Personnage;

mod battle;
use battle::Battle;

mod enemy;
use enemy::Enemy;

mod context;
use context::Context;

mod database;
use database::get_context;
use database::get_hero;
use database::init_db;
use database::update_context;

mod eink;
use eink::GTDev;
use eink::GTOld;
use eink::gt_scan;

use epd_waveshare::{
    epd2in13_v2::{Display2in13, Epd2in13},
    graphics::DisplayRotation,
    prelude::*,
};
use linux_embedded_hal::{
    Delay, SpidevDevice, SysfsPin,
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
};

use anyhow::Result;

use std::{thread, time};
// use image::io::Reader as ImageReader; // <--- NOUVEAU: Pour lire le fichier image

// use embedded_graphics::{prelude::*, image::Image};

// use embedded_hal::i2c::{I2c, Error};
use chrono::Duration;
use chrono::prelude::*;
use redb::Database;
use rppal::i2c::I2c;

const BUSY_PIN: u64 = 512 + 24;
const DC_PIN: u64 = 512 + 25;
const RST_PIN: u64 = 512 + 17;

fn main() -> Result<()> {
    let db = Database::create("mon_rpg.redb")?;

    init_db(&db)?;
    println!("Database initialized");
    let mut context = get_context(&db)?;

    let mut spi = SpidevDevice::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(10_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    // let cs = SysfsPin::new(CS_PIN); //BCM7 CE0
    // cs.export().expect("cs export");
    // while !cs.is_exported() {}
    // cs.set_direction(Direction::Out).expect("CS Direction");
    // cs.set_value(1).expect("CS Value set to 1");

    let busy = SysfsPin::new(BUSY_PIN); // GPIO 24, board J-18
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");
    //busy.set_value(1).expect("busy Value set to 1");

    let dc = SysfsPin::new(DC_PIN); // GPIO 25, board J-22
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    let rst = SysfsPin::new(RST_PIN); // GPIO 17, board J-11
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    rst.set_value(1).expect("rst Value set to 1");

    let mut delay = Delay {};

    let mut epd2in13: Epd2in13<SpidevDevice, SysfsPin, SysfsPin, SysfsPin, Delay> =
        Epd2in13::new(&mut spi, busy, dc, rst, &mut delay, None).expect("eink initalize error");
    epd2in13
        .set_refresh(&mut spi, &mut delay, RefreshLut::Full)
        .expect("set refresh");

    let mut display = Display2in13::default();

    display.set_rotation(DisplayRotation::Rotate0);

    // const SPLASH: &[u8] = include_bytes!("./image(2).bmp");
    // let splash_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(SPLASH).unwrap();
    // Image::new(&splash_bmp, Point::zero()).draw(&mut display.color_converted()).unwrap();

    // let img = ImageReader::open("attack.bmp")?.decode()?.to_luma8();
    // let eg_img = EgImage::new(&img, Point::zero());
    // eg_img.draw(&mut display)?;

    // draw_line(&mut display, 0, 50, 121, 50);
    // draw_text(&mut display, "YOUHOUUUU !", 5, 50);
    // draw_text(&mut display, "Over ", 100, 50);
    // draw_line(&mut display, 0, 57, 121, 57);

    // draw_line(&mut display, 0, 249, 121, 249);
    // draw_line(&mut display, 0, 200, 121, 200);
    // draw_line(&mut display, 0, 200, 0, 249);
    // draw_line(&mut display, 121, 200, 121, 249);

    // epd2in13.set_background_color(Color::White);
    // display.clear(Color::White).ok();

    // epd2in13
    // .update_and_display_frame(&mut spi, display.buffer(), &mut delay)
    // .expect("display frame new graphics");
    // epd2in13.update_color_frame(&mut spi, &mut delay, display.buffer(), display.chromatic_buffer())?;

    // epd2in13
    // .display_frame(&mut spi, &mut delay)
    // .expect("display frame new graphics");

    //wait 5000ms
    // thread::sleep(time::Duration::from_millis(2000));

    // draw_body(&mut display, &context);
    // draw_footer(&mut display);
    // epd2in13
    // .update_and_display_frame(&mut spi, display.buffer(), &mut delay)
    // .expect("display frame new graphics");

    let mut hero: hero::Personnage = get_hero(&db)?;

    let mut battle: battle::Battle = Battle {
        turn: "".to_string(),
        status: "".to_string(),
        message: "".to_string(),
    };

    let mut enemy: enemy::Enemy = Enemy {
        name: "".to_string(),
        hp: 0,
        max_hp: 0,
        mp: 0,
        max_mp: 0,
    };

    render(
        &mut epd2in13,
        &mut display,
        &mut spi,
        &mut delay,
        &context,
        &hero,
        &battle,
        &enemy,
    );

    let mut i2c = I2c::new()?;
    // let mut reg = [0u8; 6];
    // let data_reg: u8 = 0x814E;
    i2c.set_slave_address(0x14)?;
    // i2c.write_read(&[data_reg], &mut reg)?;

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
        let (x, y) = gt_scan(&mut i2c, &mut gt_dev, &mut gt_old)?;
        if x != 0 && y != 0 {
            let _ = handle_touch(122 - x, 250 - y, &mut context, &db);
            render(
                &mut epd2in13,
                &mut display,
                &mut spi,
                &mut delay,
                &context,
                &hero,
                &battle,
                &enemy,
            );
            // println!("X: {}, Y: {}, S: {}", x, y, s);
            // display.clear(Color::White).ok();

            // draw_text(&mut display, "Blablabla", 122 - x as i32, 250 - y as i32);

            // epd2in13
            // .update_and_display_frame(&mut spi, display.buffer(), &mut delay)
            // .expect("display frame new graphics");
        }
        let ten_seconds_from_now: DateTime<Utc> = Utc::now() - Duration::seconds(10);

        if ten_seconds_from_now > context.last_action_time {
            context.last_action_time = Utc::now();
            let _ = update_context(&db, context.clone());
            handle_action_routine(&mut context, &mut hero, &mut battle, &mut enemy);
            render(
                &mut epd2in13,
                &mut display,
                &mut spi,
                &mut delay,
                &context,
                &hero,
                &battle,
                &enemy,
            );
        }
        thread::sleep(time::Duration::from_millis(200));
    }
}

fn handle_action_routine(
    context: &mut Context,
    hero: &mut Personnage,
    battle: &mut Battle,
    enemy: &mut Enemy,
) {
    if context.action == "battle" {
        if battle.turn == "" {
            *battle = create_battle();
            *enemy = create_enemy();
        } else if battle.turn == "hero" {
            enemy.hp -= 5;
            battle.turn = "enemy".to_string();
            battle.message = "Tu es attaqué !".to_string();
        } else if battle.turn == "enemy" {
            hero.hp -= 2;
            battle.turn = "hero".to_string();
            battle.message = "L'ennemi attaque !".to_string();
        }

        // TODO
        println!("Loop battle");
    }
}

fn create_battle() -> Battle {
    Battle {
        turn: "hero".to_string(),
        status: "ongoing".to_string(),
        message: "L'ennemi apparaît !".to_string(),
    }
}

fn create_enemy() -> Enemy {
    Enemy {
        name: "Poring".to_string(),
        hp: 50,
        max_hp: 50,
        mp: 50,
        max_mp: 50,
    }
}

fn handle_touch(x: u16, y: u16, context: &mut Context, db: &Database) -> Result<()> {
    // On Action 1 (bottom left)
    if x < 60 && y > 200 {
        println!("Action 1");
        context.action = "battle".to_string();
        update_context(db, context.clone())?;
        Ok(())
    } else if x > 60 && y > 200 {
        println!("Action 2");
        context.action = "overview".to_string();
        update_context(db, context.clone())?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Action non trouvée"))
    }
}
