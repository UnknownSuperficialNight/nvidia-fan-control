use crate::{GPU_NUMBER, SPEED};
use std::io::Write;
use std::process::{Command, Stdio};

/// Determine the appropriate fan speed based on the input temperature.
///
/// This function performs two main tasks:
/// 1. It finds the closest matching speed from a predefined set (SPEED) based on the input temperature.
/// 2. It applies additional increments to the speed if the temperature is 70°C or higher.
///
/// The function works as follows:
/// - It iterates through the SPEED array to find the speed value closest to the input temperature.
/// - If the temperature is 70°C or above, it applies an additional increment to the speed.
///   The increment varies based on specific temperature ranges:
///   - 70-71°C: +2, 72-73°C: +4, 74-77°C: +6, 78-79°C: +3, 80°C: +5,
///   - 81°C: +10, 82°C: +12, 83°C: +14, 84°C: +16, 85°C and above: +15
/// - Finally, it ensures the output speed doesn't exceed 100 (maximum fan speed).
///
/// This approach allows for fine-tuned fan speed control, with more aggressive cooling at higher temperatures.
pub fn diff_func(temp: u8) -> u8 {
    let (mut speed_output, _) = SPEED.iter().fold((0, u8::MAX), |(speed, min_diff), &x| {
        let diff = if x > temp { x - temp } else { temp - x };
        if diff < min_diff {
            (x, diff)
        } else {
            (speed, min_diff)
        }
    });

    if temp >= 70 {
        let increment = match temp {
            70..=71 => 2,
            72..=73 => 4,
            74..=77 => 6,
            78..=79 => 3,
            80 => 5,
            81 => 10,
            82 => 12,
            83 => 14,
            84 => 16,
            85.. => 15,
            _ => 0,
        };

        speed_output = speed_output.saturating_add(increment);
    }

    speed_output.min(100)
}

/// Utilize NVIDIA Management Library (NVML) via nvidia-smi command-line interface
/// to retrieve the current GPU temperature. This approach leverages the C-based
/// NVML library indirectly through the nvidia-smi tool, providing a reliable
/// method to access GPU temperature data without direct NVML integration.
pub fn get_current_nvidia_temp() -> u8 {
    match Command::new("nvidia-smi").args(["--query-gpu=temperature.gpu", "--format=csv,noheader"]).output() {
        Ok(output) => {
            if let Ok(temp_str) = String::from_utf8(output.stdout) {
                if let Ok(temp) = temp_str.trim().parse() {
                    return temp;
                }
            }
            eprintln!("Error: Failed to parse temperature");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: Failed to execute nvidia-smi: {}", e);
            std::process::exit(1);
        }
    }
}

/// Resets the GPU fan control to automatic mode upon programmatic exit.
///
/// This function is designed to be called when the script exits, typically in response
/// to a SIGINT signal (Ctrl + C). It performs the following actions:
/// 1. Sets the GPU fan control state back to automatic mode.
/// 2. Restores the cursor visibility in the terminal.
///
/// # Panics
///
/// This function will panic if the `nvidia-settings` command fails to execute.
pub fn cleanup_nvidia() {
    // Set GPU fan control to automatic mode
    Command::new("nvidia-settings")
        .arg("-a")
        .arg(format!("[gpu:{}]/GPUFanControlState=0", GPU_NUMBER))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to execute nvidia-settings command");

    // Restore cursor visibility
    print!("\x1B[?25h");
    let _ = std::io::stdout().flush();
}
