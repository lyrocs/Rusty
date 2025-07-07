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
            println!("Same values");
            return Ok((0, 0));
        }

        println!("X: {}, Y: {}, S: {}", gt_dev.x[0], gt_dev.y[0], gt_dev.s[0]);
        return Ok((gt_dev.x[0], gt_dev.y[0]));
        // }
    }
    Ok((0, 0))
}
