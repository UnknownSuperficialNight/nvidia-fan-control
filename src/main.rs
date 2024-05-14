use std::fs::{metadata, remove_file};
use std::path::Path;
use std::process::{exit, Command};
use std::sync::mpsc;
use std::time::Duration;
use std::{env, thread};

use clap::{command, Arg, ArgAction};
use owo_colors::OwoColorize;
use termion::terminal_size;

mod update_func_logic;
use update_func_logic::*;

mod calculations;
use calculations::*;

mod colour_math;
use colour_math::rgb_temp;

mod checksum_func;
use checksum_func::compute_file_sha256;

// Defines interval to refresh screen and screen boundries calculations
const REFRESH_TIME: u8 = 5;

//  ╔═══════════════════════════════════════════════════════════════════╗
//  ║   Define build flags for quick compilation of different fan_args  ║
//  ╠═══════════════════════════════════════════════════════════════════╣
#[cfg(feature = "fan_amount_2")]
const FAN_AMOUNT: u8 = 2;
#[cfg(feature = "fan_amount_2")]
const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-2-fans-static"
    } else {
        "Rust-gpu-fan-control-2-fans"
    }
};

#[cfg(feature = "fan_amount_3")]
const FAN_AMOUNT: u8 = 3;
#[cfg(feature = "fan_amount_3")]
const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-3-fans-static"
    } else {
        "Rust-gpu-fan-control-3-fans"
    }
};

#[cfg(feature = "fan_amount_4")]
const FAN_AMOUNT: u8 = 4;
#[cfg(feature = "fan_amount_4")]
const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-4-fans-static"
    } else {
        "Rust-gpu-fan-control-4-fans"
    }
};

// Input your gpu fan amount here
#[cfg(not(any(feature = "fan_amount_2", feature = "fan_amount_3", feature = "fan_amount_4")))]
const FAN_AMOUNT: u8 = 1; // Default value when none of the other build options are specified
#[cfg(not(any(feature = "fan_amount_2", feature = "fan_amount_3", feature = "fan_amount_4")))]
const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-static"
    } else {
        "Rust-gpu-fan-control"
    }
};

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
            exit(1);
        }
    } else {
        println!("Unable to retrieve user information.");
        exit(1);
    }
}

fn find_gpu_manufacturer() -> u8 {
    let output = match Command::new("lspci").arg("-nnk").output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Error: Failed to execute lspci command");
            exit(1);
        }
    };

    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("NVIDIA") {
        0
    } else if output_str.contains("AMD") {
        1
    } else {
        255
    }
}

// Convert celcius to fahrenheit for the americans
fn celcius_to_fahrenheit(input_celcius: u8) -> u8 {
    (input_celcius as f32 * 1.8 + 32.0) as u8
}

// Sleep calling thread for x seconds
fn sleep(input_sec: u8) {
    thread::sleep(Duration::from_secs(input_sec.into()));
}

// Get path of the current binary at runtime
fn get_binary_path() -> String {
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
    binary_path
}

fn get_current_exe_dir() -> String {
    let current_exe_dir = match env::current_exe() {
        Ok(path) => {
            if let Some(parent_dir) = path.parent() {
                parent_dir.to_string_lossy().into_owned()
            } else {
                panic!("Failed to obtain parent directory of the current executable.");
            }
        }
        Err(e) => {
            eprintln!("Error get_current_exe_dir: {}", e);
            exit(1);
        }
    };
    current_exe_dir
}

fn main() {
    check_sudo();
    // Added if statement here for later amd gpu intergration
    let gpu_manufacturer = find_gpu_manufacturer();

    // If find_gpu_manufacturer can't find a supported gpu exit the program
    if gpu_manufacturer == 255 {
        eprintln!("Error: Unknown GPU or no GPU found");
        exit(1)
    }

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

        // Added if statement here for later amd gpu intergration
        if gpu_manufacturer == 0 {
            // Execute the cleanup function for nvidia
            cleanup_nvidia();
        } else {
            eprintln!("Error: Unknown GPU or no GPU found");
            exit(1);
        }

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
        .arg(Arg::new("fahrenheit-id").short('f').long("fahrenheit").help("Use fahrenheit instead of celsius").action(ArgAction::SetTrue))
        .get_matches();
    {
        // Standard update check on boot up prints a message if there is a newer version
        if !args.get_flag("skip-update-check") && !args.get_flag("update-now") {
            let checking_repo_version = is_current_version_older("https://github.com/UnknownSuperficialNight/nvidia-fan-control", VERSION);
            let (is_older, repo_version) = match checking_repo_version {
                Ok((is_older, version)) => (is_older, version),
                _ => (false, String::from("0.0.0")), // Default values in case of an error
            };
            if is_older {
                let binary_path = get_binary_path();
                println!("Current Version: \"{VERSION}\" is behind repo: \"{}\"", repo_version);
                println!("Please update to the new version using \"sudo ./{} -u\"", binary_path);
                println!("Resuming normal operation in 10 seconds");
                sleep(10);
            }
        }

        // Update to the new version on git i.e (Remove current binary download a new replacement binary in
        // its place)
        if args.get_flag("update-now") {
            let checking_repo_version = is_current_version_older("https://github.com/UnknownSuperficialNight/nvidia-fan-control", VERSION);
            let repo_version = match checking_repo_version {
                Ok((_, version)) => version,
                Err(_) => String::from("0.0.0"), // Default value in case of an error
            };

            let binary_path = get_binary_path();
            let current_exe_dir = get_current_exe_dir();
            let current_exe_dir_path = &format!("{}/{}", current_exe_dir, binary_path);
            let file_path_tmp = &format!("{}-dl_tmp", current_exe_dir_path);

            let checksums_vec = update_func(CAPITALIZED_BINARY_NAME, Path::new(file_path_tmp));

            let mut repo_bin_sha256_result: String = "Error".to_string();
            for checksum in &checksums_vec {
                if checksum.key == CAPITALIZED_BINARY_NAME {
                    repo_bin_sha256_result = checksum.value.clone();
                }
            }

            // Get a sha256sum of the updated binary
            let updated_bin_sha256 = compute_file_sha256(file_path_tmp);
            let updated_bin_sha256_result = match updated_bin_sha256 {
                Some(hash) => format!("{}", hash),
                None => "Failed to compute SHA-256 hash of the file.".to_string(),
            };

            // println!("repo_bin_sha256_result: {}", repo_bin_sha256_result);
            // println!("updated_bin_sha256: {}", updated_bin_sha256_result);

            if repo_bin_sha256_result == updated_bin_sha256_result {
                update_func_commit(Path::new(current_exe_dir_path), Path::new(file_path_tmp));
                println!("Checksums Match the repositorys.");
            } else {
                println!("{}", "Issue during download process checksums do not match the repositorys.".red());
                if metadata(&file_path_tmp).is_ok() {
                    if let Err(err) = remove_file(&file_path_tmp) {
                        eprintln!("Error: {}", err);
                    } else {
                        println!("Tmp_file '{}' successfully deleted", file_path_tmp);
                    }
                } else {
                    println!("File '{}' does not exist", file_path_tmp);
                }
            }

            println!("Downloaded version: {}", repo_version);
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

            // Added if statement here for later amd gpu intergration
            if gpu_manufacturer == 0 {
                for faninc in 0..FAN_AMOUNT {
                    Command::new("nvidia-settings").arg("-a").arg(&format!("GPUTargetFanSpeed[fan:{}]=100", faninc)).output().expect("nvidia-settings command failed to execute");
                }
            } else {
                eprintln!("Error: Unknown GPU or no GPU found");
                exit(1);
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

    // Define width and height variables
    let mut width: usize = 0;
    let mut height: usize = 0;

    // Define variables to save current line calculations
    let mut temp_center: usize = 0;
    let mut speed_output_center: usize = 0;
    let mut skip_center: usize = 0;
    let mut skip_changed_center: usize = 0;
    let mut vertical_center: usize = 0;
    loop {
        let temp: u8;
        // Added if statement here for later amd gpu intergration
        if gpu_manufacturer == 0 {
            temp = get_current_nvidia_temp();
        } else {
            eprintln!("Error: Unknown GPU or no GPU found");
            exit(1);
        }
        let speed_output = diff_func(temp);
        if args.get_flag("no-tui") {
            if speed_output != Into::<u8>::into(temp_capture_call) {
                // Added if statement here for later amd gpu intergration
                if gpu_manufacturer == 0 {
                    for faninc in 0..FAN_AMOUNT {
                        Command::new("nvidia-settings").arg("-a").arg(&format!("GPUTargetFanSpeed[fan:{}]={}", faninc, speed_output)).output().expect("nvidia-settings command failed to execute");
                    }
                } else {
                    eprintln!("Error: Unknown GPU or no GPU found");
                    exit(1);
                }
            }
            temp_capture_call = speed_output;
        } else {
            let rgb_value_temp = rgb_temp(temp);
            let rgb_value_speed_output = rgb_temp(speed_output);
            let gpu_temp_str: String;
            if args.get_flag("fahrenheit-id") {
                gpu_temp_str = format!("gpu temp: {}°F", celcius_to_fahrenheit(temp));
            } else {
                gpu_temp_str = format!("gpu temp: {}°C", temp);
            };
            let fan_speed_output_str = format!("Current fan speed: {}%", speed_output);
            let skip = format!("Skipped execution as speed has not changed from {}", speed_output);
            let skip_changed = format!("Changed Speed to {}", speed_output);

            // Hide the cursor
            print!("\x1B[?25l");

            // Get the terminal size
            if let Ok(size) = terminal_size() {
                // Optimize CPU cycles by caching width, height, and other calculations for future iterations.
                if size.0 as usize != width || size.1 as usize != height {
                    // Define width and height of current program terminal window
                    width = size.0 as usize;
                    height = size.1 as usize;

                    // Calculate the center position
                    temp_center = (width - gpu_temp_str.len()) / 2;
                    speed_output_center = (width - fan_speed_output_str.len()) / 2;
                    skip_center = (width - skip.len()) / 2;
                    skip_changed_center = (width - skip_changed.len()) / 2;

                    // Calculate vertical centering
                    vertical_center = height / 2;
                }

                // Clear the terminal before printing (optional)
                print!("\x1B[2J\x1B[1;1H");

                // Debug print statements
                // println!("gpu_temp_str: {}", gpu_temp_str.len());
                // println!("Width: {}", width);
                // println!("Height: {}", height);
                // println!("Temperature Center: {}", temp_center);
                // println!("Speed Output Center: {}", speed_output_center);
                // println!("Skip Center: {}", skip_center);
                // println!("Skip Changed Center: {}", skip_changed_center);
                // println!("Vertical Center: {}", vertical_center);

                // Move the cursor to the desired position
                print!("\x1B[{};H", vertical_center);

                // Print the formatted output at the calculated center positions
                println!("{: >width$}", gpu_temp_str.truecolor(rgb_value_temp.0, rgb_value_temp.1, rgb_value_temp.2), width = temp_center + gpu_temp_str.len());
                println!(
                    "{: >width$}",
                    fan_speed_output_str.truecolor(rgb_value_speed_output.0, rgb_value_speed_output.1, rgb_value_speed_output.2),
                    width = speed_output_center + fan_speed_output_str.len()
                );

                // Added if statement here for later amd gpu intergration
                if gpu_manufacturer == 0 {
                    if speed_output == Into::<u8>::into(temp_capture_call) {
                        println!("{: >width$}", skip, width = skip_center + skip.len());
                    } else {
                        println!("{: >width$}", skip_changed, width = skip_changed_center + skip_changed.len());
                        for faninc in 0..FAN_AMOUNT {
                            Command::new("nvidia-settings")
                                .arg("-a")
                                .arg(&format!(
                                    "
                GPUTargetFanSpeed[fan:{}]={}",
                                    faninc, speed_output
                                ))
                                .output()
                                .expect("nvidia-settings command failed to execute");
                        }
                    }
                    temp_capture_call = speed_output;
                } else {
                    eprintln!("Error: Unknown GPU or no GPU found");
                    exit(1);
                }
            }
        }
        sleep(REFRESH_TIME);
    }
}
