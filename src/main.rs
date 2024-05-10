use std::path::Path;
use std::process::{self, exit, Command};
use std::sync::mpsc;
use std::time::Duration;
use std::{env, thread};

use clap::{command, Arg, ArgAction};
use owo_colors::OwoColorize;
use termsize as tsize;

mod update_func_logic;
use update_func_logic::*;

mod calculations;
use calculations::*;

mod colour_math;
use colour_math::rgb_temp;

// Defines interval to refresh screen and screen boundries calculations
const REFRESH_TIME: u8 = 5;

//  ╔═══════════════════════════════════════════════════════════════════╗
//  ║   Define build flags for quick compilation of different fan_args  ║
//  ╠═══════════════════════════════════════════════════════════════════╣
#[cfg(feature = "fan_amount_2")] //                                     ║
const FAN_AMOUNT: u8 = 2; //                                            ║
                          //                                            ║
#[cfg(feature = "fan_amount_3")] //                                     ║
const FAN_AMOUNT: u8 = 3; //                                            ║
                          //                                            ║
#[cfg(feature = "fan_amount_4")] //                                     ║
const FAN_AMOUNT: u8 = 4; //                                            ║

// Input your gpu fan amount here
#[cfg(not(any(feature = "fan_amount_2", feature = "fan_amount_3", feature = "fan_amount_4")))]
const FAN_AMOUNT: u8 = 1; // Default value when none of the other build options are specified

// Input your gpu number here (if you have 1 gpu its normally nought so just leave it)
pub const GPU_NUMBER: u8 = 0;

// Finding the nearest neighbor to the current temperature and setting the speed accordingly using
// this array.
pub const SPEED: [u8; 10] = [10, 20, 30, 40, 59, 70, 80, 90, 95, 100];

// Used for checking for updates
const fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

const VERSION: &str = get_version();

// Check if user is root
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

// Sleep calling thread for x seconds
fn sleep(input_sec: u8) {
    thread::sleep(Duration::from_secs(input_sec.into()));
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

    // Set flags/arguments
    let args = command!()
        .disable_version_flag(true)
        .arg(Arg::new("skip-update-check").short('s').long("skip-update").help("Skip the update check").action(ArgAction::SetTrue))
        .arg(Arg::new("update-now").short('u').long("update").help("update the binary to a new version if available").action(ArgAction::SetTrue))
        .arg(Arg::new("no-tui").short('n').long("no_tui_output").help("no text user interface output (useful for running in the background)").action(ArgAction::SetTrue))
        .arg(Arg::new("version-num").short('v').long("version").help("Display the current version").action(ArgAction::SetTrue))
        .arg(Arg::new("test-true").short('t').long("test_fan").help("Test by setting gpu fan to 100% so you know it has control over the gpu").action(ArgAction::SetTrue))
        .get_matches();
    {
        // Get path of the current binary at runtime
        let binary_path = if let Ok(path) = env::current_exe() {
            if let Some(file_name) = path.file_name() {
                if let Some(name) = file_name.to_str() {
                    name.to_string()
                } else {
                    println!("Can't find binary_path exiting make a issue on github");
                    exit(0);
                }
            } else {
                println!("Can't find binary_path (path.file_name) exiting make a issue on github");
                exit(0);
            }
        } else {
            println!("Can't find binary_path (env::current_exe()) exiting make a issue on github");
            exit(0);
        };

        let current_exe_dir = match env::current_exe() {
            Ok(path) => {
                if let Some(parent_dir) = path.parent() {
                    parent_dir.to_string_lossy().into_owned()
                } else {
                    panic!("Failed to obtain parent directory of the current executable.");
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        };

        // Standard update check on boot up prints a message if there is a newer version
        if !args.get_flag("skip-update-check") && !args.get_flag("update-now") {
            let checking_repo_version = is_current_version_older("https://github.com/UnknownSuperficialNight/nvidia-fan-control", VERSION);
            let (is_older, repo_version) = match checking_repo_version {
                Ok((is_older, version)) => (is_older, version),
                _ => (false, String::from("0.0.0")), // Default values in case of an error
            };
            if is_older {
                println!("Current Version: \"{VERSION}\" is behind repo: \"{}\"", repo_version);
                println!("Please update to the new version using \"sudo ./{} -u\"", binary_path);
                println!("Resuming normal operation in 10 seconds");
                sleep(10);
            }
        }

        // Update to the new version on git i.e (Remove current binary download a new replacement binary in
        // its place)
        if args.get_flag("update-now") {
            let binary_name_capitalized = format!("{}{}", &binary_path[..1].to_uppercase(), &binary_path[1..]);
            let current_exe_dir_path = &format!("{}/{}", current_exe_dir, binary_path);

            update_func(binary_name_capitalized, Path::new(current_exe_dir_path));
            exit(0);
        }

        // Print current compiled version number
        if args.get_flag("version-num") {
            println!("Version: {}", VERSION);
            exit(0);
        }

        // Test Gpu but forcing it to 100% to make sure that the users gpu is responding to commands
        if args.get_flag("test-true") {
            println!("Test starting");
            for faninc in 0..FAN_AMOUNT {
                Command::new("nvidia-settings").arg("-a").arg(&format!("GPUTargetFanSpeed[fan:{}]=100", faninc)).output().expect("nvidia-settings command failed to execute");
            }
            // Wait and prompt the user to press Ctrl+C to exit
            println!("Press Ctrl+C to exit");
            loop {
                sleep(1);
            }
        }
    }

    // Define a variable to hold a starting speed value and to hold the current used/selected speed
    let mut temp_capture_call: u8 = 8;

    loop {
        let temp = get_current_tmp();
        let speed_output = diff_func(temp);
        if args.get_flag("no-tui") {
            if speed_output != Into::<u8>::into(temp_capture_call) {
                for faninc in 0..FAN_AMOUNT {
                    Command::new("nvidia-settings").arg("-a").arg(&format!("GPUTargetFanSpeed[fan:{}]={}", faninc, speed_output)).output().expect("nvidia-settings command failed to execute");
                }
            }
            temp_capture_call = speed_output;
        } else {
            let rgb_value_temp = rgb_temp(temp);
            let rgb_value_speed_output = rgb_temp(speed_output);

            let gpu_temp_str = format!("gpu temp: {}°C", temp);
            let fan_speed_output_str = format!("Current fan speed: {}%", speed_output);
            let skip = format!("Skipped execution as speed has not changed from {}", speed_output);
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
                println!("{: >width$}", gpu_temp_str.truecolor(rgb_value_temp.0, rgb_value_temp.1, rgb_value_temp.2), width = temp_center + gpu_temp_str.len());
                println!(
                    "{: >width$}",
                    fan_speed_output_str.truecolor(rgb_value_speed_output.0, rgb_value_speed_output.1, rgb_value_speed_output.2),
                    width = speed_output_center + fan_speed_output_str.len()
                );

                if speed_output == Into::<u8>::into(temp_capture_call) {
                    println!("{: >width$}", skip, width = skip_center + skip.len());
                } else {
                    println!("{: >width$}", skip_changed, width = skip_changed_center + skip_changed.len());
                    for faninc in 0..FAN_AMOUNT {
                        Command::new("nvidia-settings").arg("-a").arg(&format!("GPUTargetFanSpeed[fan:{}]={}", faninc, speed_output)).output().expect("nvidia-settings command failed to execute");
                    }
                }
                temp_capture_call = speed_output;
            }
        }
        sleep(REFRESH_TIME);
    }
}
