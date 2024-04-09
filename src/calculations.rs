use std::process::{Command, Stdio};

use crate::GPU_NUMBER;
use crate::SPEED;

pub fn diff_func() -> u8 {
    let mut speed_output: u8 = 0;
    let temp: u8 = get_current_tmp();
    let mut speed_output_diff: u8 = 255;
    for &x in SPEED.iter() {
        let diff = if x > temp { x - temp } else { temp - x };

        if diff < speed_output_diff {
            speed_output = x;

            speed_output_diff = diff;
        }
    }
    if temp > 80 {
        speed_output += 20;
    }
    if speed_output > 100 {
        speed_output = 100
    }
    speed_output
}

pub fn get_current_tmp() -> u8 {
    let temp = Command::new("nvidia-smi")
        .arg("--query-gpu=temperature.gpu")
        .arg("--format=csv,noheader")
        .stdout(Stdio::piped())
        .output()
        .unwrap();
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
    Command::new("nvidia-settings")
        .arg("-a")
        .arg(&format!("[gpu:{}]/GPUFanControlState=0", GPU_NUMBER))
        .output()
        .expect("nvidia-settings command failed to execute");
}
