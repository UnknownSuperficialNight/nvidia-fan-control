// Determine the RGB value based on the temperature
// Yes this is bad code its a stand in until i get the gradient mathematics worked out
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

// Added to be used when adding gradient calculations to convert from a human-readable colour format
// like CMYK to RGB
fn cmyk_to_rgb(c: u8, m: u8, y: u8, k: u8) -> (u8, u8, u8) {
    let r = 255.0 * (1.0 - c as f32 / 100.0) * (1.0 - k as f32 / 100.0);
    let g = 255.0 * (1.0 - m as f32 / 100.0) * (1.0 - k as f32 / 100.0);
    let b = 255.0 * (1.0 - y as f32 / 100.0) * (1.0 - k as f32 / 100.0);

    (r.round() as u8, g.round() as u8, b.round() as u8)
}
