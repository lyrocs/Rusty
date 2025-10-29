use time::{Date, Month, PrimitiveDateTime, Time};

/// Simple blocking driver for PCF85063 RTC
pub struct Pcf85063<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Pcf85063<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    const DEFAULT_ADDRESS: u8 = 0x51;

    // Register addresses
    const REG_SECONDS: u8 = 0x04;
    #[allow(dead_code)]
    const REG_MINUTES: u8 = 0x05;
    #[allow(dead_code)]
    const REG_HOURS: u8 = 0x06;
    #[allow(dead_code)]
    const REG_DAYS: u8 = 0x07;
    #[allow(dead_code)]
    const REG_MONTHS: u8 = 0x09;
    #[allow(dead_code)]
    const REG_YEARS: u8 = 0x0A;

    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: Self::DEFAULT_ADDRESS,
        }
    }

    /// Read current date and time from RTC
    pub fn get_datetime(&mut self) -> Result<PrimitiveDateTime, I2C::Error> {
        let mut buf = [0u8; 7];
        self.i2c
            .write_read(self.address, &[Self::REG_SECONDS], &mut buf)?;

        let seconds = bcd_to_decimal(buf[0] & 0x7F);
        let minutes = bcd_to_decimal(buf[1] & 0x7F);
        let hours = bcd_to_decimal(buf[2] & 0x3F);
        let days = bcd_to_decimal(buf[3] & 0x3F);
        let months = bcd_to_decimal(buf[5] & 0x1F);
        let years = 2000 + bcd_to_decimal(buf[6]) as i32;

        let month = match months {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => Month::January,
        };

        let date = Date::from_calendar_date(years, month, days)
            .unwrap_or_else(|_| Date::from_calendar_date(2024, Month::January, 1).unwrap());
        let time = Time::from_hms(hours, minutes, seconds).unwrap_or(Time::MIDNIGHT);

        Ok(PrimitiveDateTime::new(date, time))
    }

    /// Set date and time on RTC
    pub fn set_datetime(&mut self, dt: &PrimitiveDateTime) -> Result<(), I2C::Error> {
        let buf = [
            Self::REG_SECONDS,
            decimal_to_bcd(dt.time().second()),
            decimal_to_bcd(dt.time().minute()),
            decimal_to_bcd(dt.time().hour()),
            decimal_to_bcd(dt.date().day()),
            0, // weekday (not used)
            decimal_to_bcd(dt.date().month() as u8),
            decimal_to_bcd((dt.date().year() - 2000) as u8),
        ];
        self.i2c.write(self.address, &buf)
    }
}

/// Convert BCD (Binary-Coded Decimal) to normal decimal
pub fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}

/// Convert normal decimal to BCD
pub fn decimal_to_bcd(decimal: u8) -> u8 {
    ((decimal / 10) << 4) | (decimal % 10)
}
