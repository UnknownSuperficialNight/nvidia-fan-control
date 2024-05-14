use std::process::{Command, Stdio};

use crate::{GPU_NUMBER, SPEED};

// Determine the appropriate speed based on the input temperature,
// with a potential adjustment if the temperature exceeds a certain threshold i.e 80
pub fn diff_func(temp: u8) -> u8 {
    let (mut speed_output, _) = SPEED.iter().fold((0, u8::MAX), |(speed, min_diff), &x| {
        let diff = if x > temp { x - temp } else { temp - x };
        if diff < min_diff {
            (x, diff)
        } else {
            (speed, min_diff)
        }
    });

    if temp > 80 {
        speed_output = speed_output.saturating_add(20);
    }

    speed_output.min(100)
}

// Use NVML C-based library through nvsmi to get current gpu temp
pub fn get_current_nvidia_temp() -> u8 {
    let temp = Command::new("nvidia-smi").arg("--query-gpu=temperature.gpu").arg("--format=csv,noheader").stdout(Stdio::piped()).output().unwrap();
    let temp_str = String::from_utf8(temp.stdout).unwrap();
    // Remove any newline characters from the string
    let temp_cleaned = temp_str.trim().to_string();
    let temp_u8: u8 = temp_cleaned.parse().unwrap();
    temp_u8
}

// Set cpu to auto mode apon script exit using SIGINT/(Ctrl + C)
pub fn cleanup_nvidia() {
    //
    //
    //
    //
    //
    //                              ┌─────────────────┐
    //                              │ Set Gpu to auto │
    //                              └─────────────────┘
    Command::new("nvidia-settings").arg("-a").arg(&format!("[gpu:{}]/GPUFanControlState=0", GPU_NUMBER)).output().expect("nvidia-settings command failed to execute");
    print!("\x1B[?25h");
}
