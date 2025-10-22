/// Convert battery voltage (in millivolts) to percentage
/// LiPo voltage range: 4.2V (100%) to 3.0V (0%)
pub fn voltage_to_battery_percent(voltage_mv: u16) -> u8 {
    if voltage_mv >= 4200 {
        100
    } else if voltage_mv <= 3000 {
        0
    } else {
        let range = 4200 - 3000;
        let offset = voltage_mv.saturating_sub(3000);
        ((offset as u32 * 100) / range as u32).min(100) as u8
    }
}
