use std::process::{self, Command, Stdio};
use std::thread;
use std::time::Duration;
use termsize as tsize;
//use std::io;
const REFRESH_TIME: u8 = 5;
static FAN_AMOUNT: i32 = 1;

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
    let mut speed_output_diff = 999; // some high value
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

fn main() {
    check_sudo();
    let mut temp_capture_call = temp_capture();
    loop {
        let temp_capture = temp_capture();
        let speed_output = temp_loop();
        let temp = temp_func();
        sleep();

        let gpu_temp_str = format!("gpu_temp {}", temp);
        let fan_speed_output_str = format!("fan_speed_output {}", speed_output);
        // let temp_capture_str = format!("temp_capture {}", temp_capture_call);

        // Get the terminal size
        if let Some(size) = tsize::get() {
            let width = size.cols as usize; // Convert cols to usize

            // Calculate the center position
            let temp_center = (width - gpu_temp_str.len()) / 2;
            let speed_output_center = (width - fan_speed_output_str.len()) / 2;
            // let temp_capture_center = (width - temp_capture_str.len()) / 2;

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
            // println!(
            //     "{: >width$}",
            //     temp_capture_str,
            //     width = temp_capture_center + temp_capture_str.len()
            // );

            if speed_output == temp_capture_call.into() {
                println!("Skip command as speed has not changed");
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
