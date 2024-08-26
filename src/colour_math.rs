/// Calculates the RGB color value corresponding to a given temperature.
pub fn rgb_temp(rgb: &RgbColor, temp: u8) -> (u8, u8, u8) {
    // Invoke the primary calculation function with a predefined temperature range
    rgb_temp_f32(30.0, 85.0, rgb, temp as f32)
}

/// Calculates the RGB color value corresponding to a given temperature.
///
/// This function maps temperatures to RGB values within a predefined range.
pub fn rgb_temp_f32(min: f32, max: f32, rgb: &RgbColor, temp: f32) -> (u8, u8, u8) {
    // Define temperature-to-color mapping ranges
    let min_val = min;
    let max_val = max;
    let total_gradients = RgbColor::total_colors(rgb);

    // Return the selected rgb values
    let returned_temp: Option<(u8, u8, u8)> = if temp <= min_val {
        Some(rgb.colors[0])
    } else if temp >= max_val {
        rgb.colors.last().cloned().or_else(|| Some(rgb.colors[0]))
    } else {
        let selected_gradient_index = calculate_gradient_index(temp, min_val, max_val, total_gradients.into());
        RgbColor::get_color_by_index(rgb, selected_gradient_index, total_gradients)
    };

    match returned_temp {
        Some((r, g, b)) => (r, g, b),
        _none => {
            eprintln!("Error: returned_temp is None");

            (0, 0, 0)
        }
    }
}

pub struct RgbColor {
    pub colors: Vec<(u8, u8, u8)>,
}

impl RgbColor {
    pub fn new() -> RgbColor {
        RgbColor {
            colors: vec![
                (0, 255, 175),
                (0, 255, 215),
                (0, 255, 255),
                (0, 215, 255),
                (0, 175, 255),
                (0, 135, 255),
                (0, 95, 255),
                (0, 0, 255),
                (0, 0, 215),
                (0, 0, 175),
                (0, 0, 135),
                (95, 0, 135),
                (95, 0, 175),
                (95, 0, 215),
                (95, 0, 255),
                (135, 0, 255),
                (135, 0, 215),
                (135, 0, 175),
                (135, 0, 135),
                (135, 0, 95),
                (135, 0, 0),
                (175, 0, 0),
                (215, 0, 0),
                (255, 0, 0),
            ],
        }
    }

    fn get_color_by_index(&self, index: u8, selected_index: u8) -> Option<(u8, u8, u8)> {
        if index < selected_index {
            Some(self.colors[index as usize])
        } else {
            None
        }
    }

    fn total_colors(&self) -> u8 {
        self.colors.len() as u8
    }
}

/// Calculates the appropriate gradient index based on the input temperature mapped to the color array.
fn calculate_gradient_index(temp: f32, min_val: f32, max_val: f32, total_gradients: f32) -> u8 {
    // Validate that the temperature is within the specified range
    if temp < min_val || temp > max_val {
        eprintln!("Error: Temperature is out of the specified range.");
        std::process::exit(1);
    }

    // Adjust temp to start from min_val
    let adjusted_temp = temp - min_val;

    // Calculate the step size for each gradient
    let step_size = (max_val - min_val) / (total_gradients - 1.0);

    // Calculate the gradient index
    let gradient_index = (adjusted_temp - (adjusted_temp % step_size)) / step_size;

    gradient_index as u8
}
