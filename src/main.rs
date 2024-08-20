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
use colour_math::{rgb_temp, rgb_temp_f32, RgbColor};

mod checksum_func;
use checksum_func::compute_file_sha256;

mod amdgpu;
use amdgpu::get_amdgpu;

mod compile_flag_helper;
use compile_flag_helper::{CAPITALIZED_BINARY_NAME, FAN_AMOUNT};

// Defines interval to refresh screen and screen boundries calculations
fn define_refresh_time(gpu_manufacturer: u8) -> f32 {
    if gpu_manufacturer == 0 {
        5.0 // Refresh rate on nvidia gpu here format in seconds
    } else if gpu_manufacturer == 1 {
        0.1 // Refresh rate on amdgpu here format in seconds
    } else {
        eprintln!("Error: Unknown GPU or no GPU found");
        exit(1)
    }
}

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
    } else if output_str.contains("amdgpu") {
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
fn sleep(input_sec: f32) {
    thread::sleep(Duration::from_secs_f32(input_sec));
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

    let refresh_time = define_refresh_time(gpu_manufacturer);

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
        } else if gpu_manufacturer == 1 {
            print!("\x1B[?25h");
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
                sleep(10.0);
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
                    let cleaned_value = checksum.value.trim_matches('\"');
                    repo_bin_sha256_result = cleaned_value.to_string();
                }
            }

            // Get a sha256sum of the updated binary
            let updated_bin_sha256 = compute_file_sha256(file_path_tmp);

            if repo_bin_sha256_result == updated_bin_sha256 {
                update_func_commit(Path::new(current_exe_dir_path), Path::new(file_path_tmp));
                println!("Checksums Match the repositorys.");
            } else {
                println!("{}", "Issue during download process checksums do not match the repositorys.".red());
                if metadata(file_path_tmp).is_ok() {
                    if let Err(err) = remove_file(file_path_tmp) {
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
                sleep(1.0);
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

    // Define amd specific variables
    let mut amd_current_rpm: f32 = 0.0;
    let mut amd_fan_speed_percentage: u8 = 0;
    let mut amd_junction_temp: f32 = 0.0;
    let mut amd_memory_temp: f32 = 0.0;
    let mut amd_min_temp: f32 = 0.0;
    let mut amd_max_temp: f32 = 0.0;

    let rgb_array: RgbColor = RgbColor::new();
    loop {
        let temp: u8;

        // Define amd specific variables
        if gpu_manufacturer == 0 {
            temp = get_current_nvidia_temp();
        } else if gpu_manufacturer == 1 {
            // Define amd specific variables
            let amdgpu_info = get_amdgpu().unwrap();

            amd_current_rpm = amdgpu_info.get("Current RPM").map_or_else(
                || {
                    println!("Error getting Current RPM info for amd");
                    exit(1);
                },
                |&value| value,
            );

            amd_fan_speed_percentage = amdgpu_info.get("Fan Speed Percentage").map_or_else(
                || {
                    println!("Error getting Fan Speed Percentage info for amd");
                    exit(1);
                },
                |&value| value as u8,
            );

            temp = amdgpu_info.get("Edge Temp").map_or_else(
                || {
                    println!("Error getting temp info for amd");
                    exit(1);
                },
                |&value| value as u8,
            );

            amd_junction_temp = amdgpu_info.get("Junction Temp").map_or_else(
                || {
                    println!("Error getting Junction Temp info for amd");
                    exit(1);
                },
                |&value| value,
            );

            amd_memory_temp = amdgpu_info.get("Memory Temp").map_or_else(
                || {
                    println!("Error getting Memory Temp info for amd");
                    exit(1);
                },
                |&value| value,
            );

            amd_min_temp = amdgpu_info.get("Min RPM").map_or_else(
                || {
                    println!("Error getting Min RPM info for amd");
                    exit(1);
                },
                |&value| value,
            );

            amd_max_temp = amdgpu_info.get("Max RPM").map_or_else(
                || {
                    println!("Error getting Max RPM info for amd");
                    exit(1);
                },
                |&value| value,
            );
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
            let rgb_value_temp = rgb_temp(&rgb_array, temp);
            let rgb_value_speed_output = rgb_temp(&rgb_array, speed_output);
            let gpu_temp_str: String = if args.get_flag("fahrenheit-id") { format!("Gpu temp: {}°F", celcius_to_fahrenheit(temp)) } else { format!("Gpu temp: {}°C", temp) };
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

                if gpu_manufacturer == 1 {
                    // Calculate junction pos
                    let amd_junction_temp_colour = rgb_temp_f32(50.0, 95.0, &rgb_array, amd_junction_temp);
                    let amd_junction_temp_str: String = if args.get_flag("fahrenheit-id") {
                        format!("Junction/hotspot: {}°F", celcius_to_fahrenheit(amd_junction_temp as u8))
                    } else {
                        format!("Junction/hotspot: {}°C", amd_junction_temp)
                    };
                    let amd_junction_temp_center = (width - amd_junction_temp_str.len()) / 2;
                    println!(
                        "{: >width$}",
                        amd_junction_temp_str.truecolor(amd_junction_temp_colour.0, amd_junction_temp_colour.1, amd_junction_temp_colour.2),
                        width = amd_junction_temp_center + amd_junction_temp_str.len()
                    );

                    // Calculate Vram/Memory pos
                    let amd_memory_temp_colour = rgb_temp_f32(60.0, 90.0, &rgb_array, amd_memory_temp);
                    let amd_memory_temp_str: String = if args.get_flag("fahrenheit-id") {
                        format!("Memory/vram temp: {}°F", celcius_to_fahrenheit(amd_memory_temp as u8))
                    } else {
                        format!("Memory/vram temp: {}°C", amd_memory_temp)
                    };
                    let amd_memory_temp_center = (width - amd_memory_temp_str.len()) / 2;
                    println!(
                        "{: >width$}",
                        amd_memory_temp_str.truecolor(amd_memory_temp_colour.0, amd_memory_temp_colour.1, amd_memory_temp_colour.2),
                        width = amd_memory_temp_center + amd_memory_temp_str.len()
                    );

                    // Calculate rpm pos
                    let amd_current_rpm_colour = rgb_temp_f32(amd_min_temp, amd_max_temp, &rgb_array, amd_current_rpm);
                    let amd_current_rpm_str: String = format!("Current fan RPM: {}", amd_current_rpm);
                    let amd_current_rpm_center = (width - amd_current_rpm_str.len()) / 2;
                    println!(
                        "{: >width$}",
                        amd_current_rpm_str.truecolor(amd_current_rpm_colour.0, amd_current_rpm_colour.1, amd_current_rpm_colour.2),
                        width = amd_current_rpm_center + amd_current_rpm_str.len()
                    );

                    // Calculate fanspeed pos
                    let amd_fan_speed_percentage_colour = rgb_temp(&rgb_array, amd_fan_speed_percentage);
                    let amd_fan_speed_percentage_str: String = format!("Current fan speed: {}%", amd_fan_speed_percentage);
                    let amd_fan_speed_percentage_center = (width - amd_fan_speed_percentage_str.len()) / 2;
                    println!(
                        "{: >width$}",
                        amd_fan_speed_percentage_str.truecolor(amd_fan_speed_percentage_colour.0, amd_fan_speed_percentage_colour.1, amd_fan_speed_percentage_colour.2),
                        width = amd_fan_speed_percentage_center + amd_fan_speed_percentage_str.len()
                    );
                } else {
                    println!(
                        "{: >width$}",
                        fan_speed_output_str.truecolor(rgb_value_speed_output.0, rgb_value_speed_output.1, rgb_value_speed_output.2),
                        width = speed_output_center + fan_speed_output_str.len()
                    );
                }
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
                } else if gpu_manufacturer == 1 {
                    // TODO: Here for possible fan setting
                } else {
                    eprintln!("Error: Unknown GPU or no GPU found");
                    exit(1);
                }
            }
        }
        sleep(refresh_time);
    }
}
