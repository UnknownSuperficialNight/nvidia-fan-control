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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    // Normalize input values to required ranges
    let h = h / 360.0; // Hue needs to be in [0, 1] for the calculations
    let s = s / 100.0; // Saturation is converted from [0, 100] to [0, 1]
    let v = v / 100.0; // Value is converted from [0, 100] to [0, 1]

    // Check for achromatic (grey) color
    if s == 0.0 {
        // Achromatic (grey)
        let value = (v * 255.0).round() as u8;
        return (value, value, value); // Return grey color tuple
    }

    // Calculate intermediate values for RGB conversion
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    // Determine RGB values based on intermediate values
    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (0.0, 0.0, 0.0), // Should not happen due to the modulo operation
    };

    // Scale and round RGB values to u8 range and return as tuple
    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}
