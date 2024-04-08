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

fn temp_func() -> String {
    let temp = Command::new("nvidia-smi")
        .arg("--query-gpu=temperature.gpu")
        .arg("--format=csv,noheader")
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    String::from_utf8(temp.stdout).unwrap()
}

fn sleep() {
    thread::sleep(Duration::from_secs(REFRESH_TIME.into()));
}

fn temp_loop() -> i32 {
    let mut speed_output = 0;
    let temp = temp_func();
    let target: i32 = temp.trim().parse().expect("Wanted a number");
    let mut speed_output_diff = 999;
    let speed: [i32; 10] = [10, 20, 30, 40, 59, 70, 80, 90, 95, 100];
    for x in speed {
        let diff = (x - target).abs();

        if diff < speed_output_diff {
            speed_output = x;

            speed_output_diff = diff;
        }
    }
    if temp > 80.to_string() {
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
        sleep();

        let gpu_temp_str = format!("gpu_temp {}", temp);
        let fan_speed_output_str = format!("fan_speed_output {}", speed_output);
        let skip = format!("Skip command as speed has not changed");

        // Get the terminal size
        if let Some(size) = tsize::get() {
            let width = size.cols as usize; // Convert cols to usize

            // Calculate the center position
            let temp_center = (width - gpu_temp_str.len()) / 2;
            let speed_output_center = (width - fan_speed_output_str.len()) / 2;
            let skip_center = (width - skip.len()) / 2;

            // Print the formatted output at the calculated center positions
            println!(
                "{: >width$}",
                gpu_temp_str,
                width = temp_center + gpu_temp_str.len()
            );
            println!(
                "{: >width$}",
                fan_speed_output_str,
                width = speed_output_center + fan_speed_output_str.len()
            );
            println!("{: >width$}", skip, width = skip_center + skip.len());

            if speed_output == temp_capture_call.into() {
            } else {
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
