use std::process::{Command, Stdio};

use crate::{GPU_NUMBER, SPEED};

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

pub fn get_current_tmp() -> u8 {
    let temp = Command::new("nvidia-smi").arg("--query-gpu=temperature.gpu").arg("--format=csv,noheader").stdout(Stdio::piped()).output().unwrap();
    let temp_str = String::from_utf8(temp.stdout).unwrap();
    // Remove any newline characters from the string
    let temp_cleaned = temp_str.trim().to_string();
    let temp_u8: u8 = temp_cleaned.parse().unwrap();
    temp_u8
}

pub fn cleanup() {
    //
    //
    //
    //
    //
    //                              ┌─────────────────┐
    //                              │ Set Gpu to auto │
    //                              └─────────────────┘
    Command::new("nvidia-settings").arg("-a").arg(&format!("[gpu:{}]/GPUFanControlState=0", GPU_NUMBER)).output().expect("nvidia-settings command failed to execute");
}
