// Determine the RGB value based on the temperature
pub fn rgb_temp(temp: u8) -> (u8, u8, u8) {
    let blue = match temp {
        0..=34 => 206 - ((35 - temp) * 4),
        35..=44 => 206,
        45..=59 => 206 - ((temp - 45) * 4),
        60 => 197,
        61..=69 => 197 + ((temp - 60) * 3),
        _ => 255,
    };
    let green = match temp {
        0..=34 => 64 + (temp * 3),
        35..=44 => 255,
        45..=59 => 56,
        60 => 0,
        61..=69 => 0,
        _ => 0,
    };
    let red = match temp {
        0..=34 => 64,
        35..=44 => 201,
        45..=59 => 206,
        60 => 206,
        61..=69 => 206,
        _ => 255,
    };
    (red, green, blue)
}
