use owo_colors::OwoColorize;
use std::process::{self, exit, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termsize as tsize;

// Defines interval to refresh
const REFRESH_TIME: u8 = 5;

#[cfg(feature = "fan_amount_2")]
const FAN_AMOUNT: u8 = 2;

#[cfg(feature = "fan_amount_3")]
const FAN_AMOUNT: u8 = 3;

#[cfg(feature = "fan_amount_4")]
const FAN_AMOUNT: u8 = 4;

// Input your gpu fan amount here
#[cfg(not(any(
    feature = "fan_amount_2",
    feature = "fan_amount_3",
    feature = "fan_amount_4"
)))]
const FAN_AMOUNT: u8 = 1; // Default value when none of the other options are specified

// Input your gpu number here (if you have 1 gpu its normally nought so just leave it)
const GPU_NUMBER: u8 = 0;

// Determine the RGB value based on the temperature
fn rgb_temp(temp: u8) -> (u8, u8, u8) {
    let blue = if temp < 38 {
        255 - ((38 - temp) * 5)
    } else if temp < 70 {
        0
    } else {
        (temp - 70) * 3
    };
    let green = if temp < 38 {
        temp * 2
    } else if temp < 70 {
        0
    } else {
        (temp - 70) * 2
    };
    let red = if temp < 38 {
        0
    } else if temp < 70 {
        (temp - 38) * 2
    } else {
        255
    };
    (red, green, blue)
}

fn check_sudo() {
    if let Ok(user) = std::env::var("USER") {
        if user != "root" {
            println!("This script must be run with sudo privileges.");
            process::exit(1);
        }
    } else {
        println!("Unable to retrieve user information.");
        process::exit(1);
    }
}

fn temp_func() -> u8 {
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

fn sleep() {
    thread::sleep(Duration::from_secs(REFRESH_TIME.into()));
}

fn temp_loop() -> u8 {
    let mut speed_output: u8 = 0;
    let temp: u8 = temp_func();
    let mut speed_output_diff: u8 = 255;
    let speed: [u8; 10] = [10, 20, 30, 40, 59, 70, 80, 90, 95, 100];
    for &x in speed.iter() {
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
    return speed_output;
}

fn temp_capture() -> u8 {
    let temp_capture2 = temp_loop();
    return temp_capture2.try_into().unwrap();
}

fn cleanup() {
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

fn main() {
    check_sudo();

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel();

    // Set up the Ctrl+C handler
    ctrlc::set_handler(move || {
        // Send a signal on the channel when Ctrl+C is pressed
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl+C handler");

    // Spawn a new thread to execute the cleanup function
    thread::spawn(move || {
        // Wait for the signal from the main thread
        rx.recv().expect("Could not receive from channel.");

        // Execute the cleanup function
        cleanup();

        // Exit the program
        exit(0);
    });

    let mut temp_capture_call = temp_capture();
    loop {
        let temp_capture = temp_capture();
        let speed_output = temp_loop();
        let temp = temp_func();
        let rgb_value_temp = rgb_temp(temp.into());
        let rgb_value_speed_output = rgb_temp(speed_output.into());
        sleep();

        let gpu_temp_str = format!("gpu temp: {}°C", temp);
        let fan_speed_output_str = format!("Current fan speed: {}%", speed_output);
        let skip = format!(
            "Skipped execution as speed has not changed from {}",
            speed_output
        );
        let skip_changed = format!("Changed Speed to {}", speed_output);

        // Get the terminal size
        if let Some(size) = tsize::get() {
            let width = size.cols as usize; // Convert cols to usize
            let height = size.rows as usize; // Convert rows to usize

            // Calculate the center position
            let temp_center = (width - gpu_temp_str.len()) / 2;
            let speed_output_center = (width - fan_speed_output_str.len()) / 2;
            let skip_center = (width - skip.len()) / 2;
            let skip_changed_center = (width - skip_changed.len()) / 2;

            // Calculate vertical centering
            let vertical_center = height / 2;

            // Clear the terminal before printing (optional)
            print!("\x1B[2J\x1B[1;1H");

            // Use the vertical center to print the strings in the middle of the screen
            for _ in 0..vertical_center - 1 {
                println!();
            }
            // Print the formatted output at the calculated center positions
            println!(
                "{: >width$}",
                gpu_temp_str.truecolor(rgb_value_temp.0, rgb_value_temp.1, rgb_value_temp.2),
                width = temp_center + gpu_temp_str.len()
            );
            println!(
                "{: >width$}",
                fan_speed_output_str.truecolor(
                    rgb_value_speed_output.0,
                    rgb_value_speed_output.1,
                    rgb_value_speed_output.2
                ),
                width = speed_output_center + fan_speed_output_str.len()
            );

            if speed_output == temp_capture_call.into() {
                println!("{: >width$}", skip, width = skip_center + skip.len());
            } else {
                println!(
                    "{: >width$}",
                    skip_changed,
                    width = skip_changed_center + skip_changed.len()
                );
                for faninc in 0..FAN_AMOUNT {
                    Command::new("nvidia-settings")
                        .arg("-a")
                        .arg(&format!(
                            "GPUTargetFanSpeed[fan:{}]={}",
                            faninc, speed_output
                        ))
                        .output()
                        .expect("nvidia-settings command failed to execute");
                }
            }
            temp_capture_call = temp_capture;
        }
    }
}
