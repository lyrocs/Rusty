use linux_embedded_hal::{
    Delay, SpidevDevice, SysfsPin,
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
};

use epd_waveshare::{
    epd2in13_v2::{Display2in13, Epd2in13},
    graphics::DisplayRotation,
    prelude::*,
};

use rppal::i2c::I2c;

use crate::models::eink::Eink;

pub struct GTDev {
    pub touchpoint_flag: u8,
    pub touch_count: u8,
    pub x: [u16; 5],
    pub y: [u16; 5],
    pub s: [u16; 5],
    pub touchkeytrackid: [u8; 5],
}

pub struct GTOld {
    pub x: [u16; 5],
    pub y: [u16; 5],
    pub s: [u16; 5],
}

const BUSY_PIN: u64 = 512 + 24;
const DC_PIN: u64 = 512 + 25;
const RST_PIN: u64 = 512 + 17;

pub fn init_eink() -> Eink {
    let mut spi = SpidevDevice::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(10_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

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


    let mut i2c = I2c::new().unwrap();
    i2c.set_slave_address(0x14).unwrap();

    return Eink {
        i2c,
        display,
        epd2in13,
        spi,
        delay,
    };

}

pub fn gt_scan(
    i2c: &mut rppal::i2c::I2c,
    gt_dev: &mut GTDev,
    gt_old: &mut GTOld,
) -> Result<(u16, u16), rppal::i2c::Error> {
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
            return Ok((0, 0));
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

        if gt_old.x[0] == gt_dev.x[0]
            && gt_old.y[0] == gt_dev.y[0]
            && gt_old.s[0] == gt_dev.s[0]
            && (gt_old.x[0] != 0 && gt_old.y[0] != 0 && gt_old.s[0] != 0)
        {
            return Ok((0, 0));
        }

        return Ok((gt_dev.x[0], gt_dev.y[0]));
    }
    Ok((0, 0))
}
