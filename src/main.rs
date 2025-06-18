use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    image::Image as EgImage,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Triangle, Rectangle, PrimitiveStyleBuilder},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_framebuf::FrameBuf;
// use embedded_hal::delay::DelayNs;
use epd_waveshare::{
    color::*,
    epd2in13_v2::{Display2in13, Epd2in13},
    graphics::DisplayRotation,
    prelude::*,
};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    Delay, SPIError, SpidevDevice, SysfsPin,
    I2cdev,
};

use rppal::{
    gpio::Gpio,
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use anyhow::Result;
use std::{error, thread, time};
use gt911::Gt911Blocking;
// use image::io::Reader as ImageReader; // <--- NOUVEAU: Pour lire le fichier image

use embedded_graphics::{prelude::*, image::Image};

// use embedded_hal::i2c::{I2c, Error};
use rppal::i2c::I2c;

use byteorder::{ByteOrder, LittleEndian};

use i2cdev::linux::*;

const CS_PIN: u64 = 512+ 8;
const BUSY_PIN: u64 = 512 + 24;
const DC_PIN: u64 = 512 + 25;
const RST_PIN: u64 = 512 + 17;


// --- Configuration ---
// Broche GPIO (BCM) connectée à la broche INT du contrôleur tactile
// On garde le décalage de 512 si vous êtes sur un Raspberry Pi 5
const INT_PIN: u64 = 512 + 4;

// Adresse I2C du GT911. 0x5D est une valeur courante.
const I2C_TOUCH_ADDR: u8 = 0x14;

struct GTDev {
    touch: u8,
    touchpoint_flag: u8,
    touch_count: u8,
    x: [u16; 5],
    y: [u16; 5],
    s: [u16; 5],
    touchkeytrackid: [u8; 5],
}

struct GTOld {
    x: [u16; 5],
    y: [u16; 5],
    s: [u16; 5],
}


fn main() -> Result<()> {
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

    let mut epd2in13 =
        Epd2in13::new(&mut spi, busy, dc, rst, &mut delay, None).expect("eink initalize error");
    epd2in13.set_refresh(&mut spi, &mut delay, RefreshLut::Full).expect("set refresh");

    let mut display = Display2in13::default();

    display.set_rotation(DisplayRotation::Rotate0);
    display.clear(Color::White).ok();

    // const SPLASH: &[u8] = include_bytes!("./image(2).bmp");
    // let splash_bmp = tinybmp::Bmp::<BinaryColor>::from_slice(SPLASH).unwrap();
    // Image::new(&splash_bmp, Point::zero()).draw(&mut display.color_converted()).unwrap();


    // let img = ImageReader::open("attack.bmp")?.decode()?.to_luma8();
    // let eg_img = EgImage::new(&img, Point::zero());
    // eg_img.draw(&mut display)?;


    draw_line(&mut display, 0, 50, 121, 50);
    draw_text(&mut display, "YOUHOUUUU !", 5, 50);
    draw_text(&mut display, "Over ", 100, 50);
    draw_line(&mut display, 0, 57, 121, 57);




    draw_line(&mut display, 0, 249, 121, 249);
    draw_line(&mut display, 0, 200, 121, 200);
    draw_line(&mut display, 0, 200, 0, 249);
    draw_line(&mut display, 121, 200, 121, 249);

    // epd2in13.set_background_color(Color::White);
    // display.clear(Color::White).ok();

    epd2in13
    .update_and_display_frame(&mut spi, display.buffer(), &mut delay)
    .expect("display frame new graphics");
    // epd2in13.update_color_frame(&mut spi, &mut delay, display.buffer(), display.chromatic_buffer())?;

    epd2in13
    .display_frame(&mut spi, &mut delay)
    .expect("display frame new graphics");


    //wait 5000ms
    // thread::sleep(time::Duration::from_millis(2000));





    let mut i2c = I2c::new()?;
    // let mut reg = [0u8; 6];
    // let data_reg: u8 = 0x814E;
    i2c.set_slave_address(0x14)?;
    // i2c.write_read(&[data_reg], &mut reg)?;

    let mut gt_dev = GTDev {
        touch: 1,
        touchpoint_flag: 0,
        touch_count: 0,
        x: [0; 5],
        y: [0; 5],
        s: [0; 5],
        touchkeytrackid: [0; 5],
    };
    let mut gt_old = GTOld { x: [0; 5], y: [0; 5], s: [0; 5] };

    loop {
        let (x, y, s) = gt_scan(&mut i2c, &mut gt_dev, &mut gt_old)?;
        if x != 0 && y != 0 && s != 0 {
            // println!("X: {}, Y: {}, S: {}", x, y, s);
            // display.clear(Color::White).ok();
            epd2in13.set_refresh(&mut spi, &mut delay, RefreshLut::Quick).expect("set refresh");


            // let style = PrimitiveStyleBuilder::new()
            //     .stroke_color(Color::Black)
            //     .stroke_width(1)
            //     .fill_color(Color::Black)
            //     .build();
            // Rectangle::new(Point::new(122 - x as i32, 250 - y as i32), Size::new(250,10 ))
            // .into_styled(style)
            // .draw(&mut display)
            // .unwrap();

            // epd2in13
            // .update_and_display_frame(&mut spi, display.buffer(), &mut delay)
            // .expect("display frame new graphics");

            // thread::sleep(time::Duration::from_millis(20));

            // let style = PrimitiveStyleBuilder::new()
            // .stroke_color(Color::White)
            // .stroke_width(1)
            // .fill_color(Color::White)
            // .build();
            // Rectangle::new(Point::new(122 - x as i32, 250 - y as i32), Size::new(250, 10))
            // .into_styled(style)
            // .draw(&mut display)
            // .unwrap();

            draw_text(&mut display, "Blablabla", 122 - x as i32, 250 - y as i32);

            epd2in13
            .update_and_display_frame(&mut spi, display.buffer(), &mut delay)
            .expect("display frame new graphics");

            

           
            
            // epd2in13
            // .update_and_display_frame(&mut spi, display.buffer(), &mut delay)
            // .expect("display frame new graphics");


            // let mut small_buffer = [Color::White; 512]; //16x16 16*256
            // // println!("buffer before {:?}", small_buffer);

            // let mut fbuf = FrameBuf::new(&mut small_buffer, 16, 32);

            // let style = MonoTextStyleBuilder::new()
            // .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
            // .text_color(Color::Black)
            // .background_color(Color::White)
            // .build();
    
            // let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
        
            // let _ = Text::with_text_style("XD", Point::zero(), style, text_style).draw(&mut fbuf).unwrap();

            // println!("buffer after {:?}", small_buffer);

            // let mut cut_buffer = [0u8; 32];

            // for row in 0..16 {
            //     for col in 0..16 {
            //         // Index in the original 16x256 buffer
            //         let src_idx = row * 256 + col;
            //         // Index in the new 16x16 buffer
            //         let dst_idx = row * 16 + col;
            //         cut_buffer[dst_idx] = small_buffer[src_idx].get_byte_value();
            //     }
            // }

            // let mut byte_buffer = [0u8; 512];
            // for (i, color) in small_buffer.iter().enumerate() {
            //     println!("color {:?}", color);
            //     if color == &Color::Black {
            //         byte_buffer[i] = Color::Black as u8;
            //     } else {
            //         byte_buffer[i] = Color::White as u8;
            //     }
            // }

            // let width = 8;
            // let height = 16;
            // let mut packed_buffer = [0u8; 16]; // 16*32 = 512 pixels / 8 = 64 bytes

            // for row in 0..height {
            //     for col_byte in 0..(width / 8) {
            //         let mut byte = 0u8;
            //         for bit in 0..8 {
            //             let col = col_byte * 8 + bit;
            //             let idx = row * width + col;
            //             if small_buffer[idx] == Color::Black {
            //                 byte |= 1 << (7 - bit); // MSB first (common for e-paper)
            //             }
            //         }
            //         packed_buffer[row * (width / 8) + col_byte] = byte;
            //     }
            // }

            // epd2in13
            // .update_partial_frame(&mut spi, &mut delay,  &packed_buffer, 122 - x as u32, 250 - y as u32, width as u32, height as u32)
            // .expect("display frame new graphics");

            // epd2in13
            // .display_frame(&mut spi, &mut delay)
            // .expect("display frame new graphics");
        }
        thread::sleep(time::Duration::from_millis(200));
    }

    Ok(())
}



fn draw_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

fn draw_line(display: &mut Display2in13, x1: i32, y1: i32, x2: i32, y2: i32) {
    let _ = Line::new(Point::new(x1, y1), Point::new(x2, y2))
    .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
    .draw(display);
}

fn gt_scan(i2c: &mut rppal::i2c::I2c, gt_dev: &mut GTDev, gt_old: &mut GTOld) -> Result<((u16,u16,u16)), rppal::i2c::Error> {
    let mask = 0x00u8;

    // if gt_dev.touch == 1 {
        // gt_dev.touch = 0;

        // Read 1 byte from 0x814E
        let reg_addr = [0x81, 0x4E];
        let mut buf = [0u8; 1];
        i2c.write_read(&reg_addr, &mut buf)?;

        if buf[0] & 0x80 == 0x00 {
            // Write mask to 0x814E
            i2c.write(&[0x81, 0x4E, mask])?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        } else {
            gt_dev.touchpoint_flag = buf[0] & 0x80;
            gt_dev.touch_count = buf[0] & 0x0f;

            if gt_dev.touch_count > 5 || gt_dev.touch_count < 1 {
                i2c.write(&[0x81, 0x4E, mask])?;
                return Ok((0,0,0));
            }

            // Read touch data
            let count = gt_dev.touch_count as usize;
            let reg_addr = [0x81, 0x4F];
            let mut buf = vec![0u8; count * 8];
            i2c.write_read(&reg_addr, &mut buf)?;

            // Write mask to 0x814E
            i2c.write(&[0x81, 0x4E, mask])?;

            // Save old values
            gt_old.x[0] = gt_dev.x[0];
            gt_old.y[0] = gt_dev.y[0];
            gt_old.s[0] = gt_dev.s[0];

            for i in 0..count {
                gt_dev.touchkeytrackid[i] = buf[0 + 8 * i];
                gt_dev.x[i] = ((buf[2 + 8 * i] as u16) << 8) | (buf[1 + 8 * i] as u16);
                gt_dev.y[i] = ((buf[4 + 8 * i] as u16) << 8) | (buf[3 + 8 * i] as u16);
                gt_dev.s[i] = ((buf[6 + 8 * i] as u16) << 8) | (buf[5 + 8 * i] as u16);
            }

            println!("X: {}, Y: {}, S: {}", gt_dev.x[0], gt_dev.y[0], gt_dev.s[0]);
            return Ok((gt_dev.x[0], gt_dev.y[0], gt_dev.s[0]));
        // }
    }
    Ok((0,0,0))
}