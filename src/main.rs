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

/// Defines the interval for refreshing the screen and recalculating screen boundaries.
/// This function determines the appropriate refresh rate based on the GPU manufacturer.
fn define_refresh_time(gpu_manufacturer: u8) -> f32 {
    if gpu_manufacturer == 0 {
        0.3 // Refresh rate for NVIDIA GPUs (in seconds)
    } else if gpu_manufacturer == 1 {
        0.1 // Refresh rate for AMD GPUs (in seconds)
    } else {
        eprintln!("Error: Unknown GPU or no GPU found");
        exit(1)
    }
}

/// Specify the GPU number (0 for the first GPU, 1 for the second, etc.)
/// If you have only one GPU, leave this as 0
pub const GPU_NUMBER: u8 = 0;

/// This array defines fan speeds (in percentages) corresponding to different temperature thresholds.
/// The index of each speed value represents a temperature range.
/// The program uses this array to determine the appropriate fan speed based on the current GPU temperature.
/// It finds the nearest matching temperature and sets the fan speed to the corresponding value in this array.
pub const SPEED: [u8; 10] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

/// Used for checking for updates
const fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

const VERSION: &str = get_version();

/// Check if user is root
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

/// Used to find if the user has a supported gpu
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
        eprintln!("Error: Unknown GPU or no GPU found");
        exit(1)
    }
}

/// Convert celcius to fahrenheit for the americans
fn celcius_to_fahrenheit(input_celcius: u8) -> u8 {
    (input_celcius as f32 * 1.8 + 32.0) as u8
}

/// Sleep calling thread for x seconds
fn sleep(input_sec: f32) {
    thread::sleep(Duration::from_secs_f32(input_sec));
}

/// Retrieves the path of the current executable binary at runtime.
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

fn setup_ctrl_c_handler(gpu_manufacturer: u8) {
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

        // Added if statement here for later AMD GPU integration
        if gpu_manufacturer == 0 {
            // Execute the cleanup function for NVIDIA
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

/// Prints a colored string centered in the terminal
///
/// # Arguments
/// * `width` - The width of the terminal
/// * `format_str` - The primary format string for the output
/// * `value` - The value to be inserted into the format string
/// * `min_temp` - The minimum temperature for color scaling
/// * `max_temp` - The maximum temperature for color scaling
/// * `rgb_array` - The RgbColor struct for color calculations
/// * `use_alt_str` - Boolean flag to use the alternate string
/// * `alt_format_str` - The alternate format string (used if use_alt_str is true)
fn print_centered_colored_string(
    width: usize,
    format_str: &str,
    value: Option<f32>,
    min_temp: Option<f32>,
    max_temp: Option<f32>,
    rgb_array: &RgbColor,
    use_alt_str: bool,
    alt_format_str: Option<&str>,
) {
    if let (Some(value), Some(min_temp), Some(max_temp)) = (value, min_temp, max_temp) {
        // Calculate the color based on the temperature range
        let color = rgb_temp_f32(min_temp, max_temp, rgb_array, value);
        // Choose the appropriate format string
        let chosen_format_str = if use_alt_str { alt_format_str.unwrap_or(format_str) } else { format_str };
        // Format the string with the provided value
        let formatted_value = if use_alt_str { celcius_to_fahrenheit(value as u8) as f32 } else { value };
        let formatted_str = format!("{}", chosen_format_str.replace("{}", &formatted_value.to_string()));
        // Calculate the center position for the string
        let center = (width - formatted_str.len()) / 2;
        // Print the formatted string with calculated color and centering
        println!("{: >width$}", formatted_str.truecolor(color.0, color.1, color.2), width = center + formatted_str.len());
    }
}

fn main() {
    // Make sure the executing user is sudo
    check_sudo();

    // Set flags/arguments
    let args = command!()
        .disable_version_flag(true)
        .arg(Arg::new("skip-update-check").short('s').long("skip-update").help("Skip the update check").action(ArgAction::SetTrue))
        .arg(Arg::new("update-now").short('u').long("update").help("update the binary to a new version if available").action(ArgAction::SetTrue))
        .arg(Arg::new("no-tui").short('n').long("no_tui_output").help("no text user interface output (useful for running in the background)").action(ArgAction::SetTrue))
        .arg(Arg::new("version-num").short('v').long("version").help("Display the current version").action(ArgAction::SetTrue))
        .arg(Arg::new("test-true").short('t').long("test_fan").help("Test by setting gpu fan to 100% so you know it has control over the gpu (nvidia only)").action(ArgAction::SetTrue))
        .arg(Arg::new("fahrenheit-id").short('f').long("fahrenheit").help("Use fahrenheit instead of celsius").action(ArgAction::SetTrue))
        .arg(Arg::new("force-nvidia").long("nvidia").help("Force the detected gpu to be nvidia").action(ArgAction::SetTrue))
        .arg(Arg::new("force-amd").long("amd").help("Force the detected gpu to be amd").action(ArgAction::SetTrue))
        .get_matches();

    // Auto detects gpu to target unless overridden with --amd or --nvidia
    let gpu_manufacturer = if args.get_flag("force-amd") {
        1
    } else if args.get_flag("force-nvidia") {
        0
    } else {
        find_gpu_manufacturer()
    };

    // Defines what second interval amd or nvidia ui it refreshed at
    let refresh_time = define_refresh_time(gpu_manufacturer);

    setup_ctrl_c_handler(gpu_manufacturer);

    {
        // Performs a standard version check at startup and notifies if an update is available
        if !args.get_flag("skip-update-check") && !args.get_flag("update-now") {
            let checking_repo_version = is_current_version_older("https://github.com/UnknownSuperficialNight/nvidia-fan-control", VERSION);
            let (is_older, repo_version) = match checking_repo_version {
                Ok((is_older, version)) => (is_older, version),
                Err(_) => (false, String::from("0.0.0")), // Default values in the event of an error
            };
            if is_older {
                let binary_path = get_binary_path();
                println!("Current Version: \"{VERSION}\" is behind repo: \"{}\"", repo_version);
                println!("Please update to the new version using \"sudo ./{} -u\"", binary_path);
                println!("Resuming normal operation in 10 seconds");
                sleep(10.0);
            }
        }

        // Update the binary to the latest version from the repository
        // This process involves removing the current binary and downloading a new replacement
        if args.get_flag("update-now") {
            let checking_repo_version = is_current_version_older("https://github.com/UnknownSuperficialNight/nvidia-fan-control", VERSION);
            let repo_version = match checking_repo_version {
                Ok((_, version)) => version,
                Err(_) => String::from("0.0.0"), // Default to initial version if unable to retrieve current version
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

            // Calculate the SHA-256 checksum of the updated binary
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

        // Display the current version number of the compiled binary
        if args.get_flag("version-num") {
            println!("Version: {}", VERSION);
            exit(0);
        }

        // Test GPU responsiveness by setting fan speed to 100%
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
            // Pause execution and instruct the user to terminate the program using Ctrl+C
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
    let mut amd_current_rpm: Option<f32> = None;
    let mut amd_fan_speed_percentage: Option<u8> = None;
    let mut amd_junction_temp: Option<f32> = None;
    let mut amd_memory_temp: Option<f32> = None;
    let mut amd_min_temp: Option<f32> = None;
    let mut amd_max_temp: Option<f32> = None;

    let rgb_array: RgbColor = RgbColor::new();
    loop {
        let temp: u8;

        // Get info depending on gpu
        if gpu_manufacturer == 0 {
            temp = get_current_nvidia_temp();
        } else if gpu_manufacturer == 1 {
            let amdgpu_info = get_amdgpu().unwrap();

            amd_current_rpm = amdgpu_info.get("Current RPM").map_or_else(
                || {
                    println!("Error getting Current RPM info for amd. This could be due to a missing sensor for your GPU model.");
                    None
                },
                |&value| Some(value),
            );

            amd_fan_speed_percentage = amdgpu_info.get("Fan Speed Percentage").map_or_else(
                || {
                    println!("Error getting Fan Speed Percentage info for amd. This could be due to a missing sensor for your GPU model.");
                    None
                },
                |&value| Some(value as u8),
            );

            temp = amdgpu_info
                .get("Edge Temp")
                .map_or_else(
                    || {
                        println!("Error getting temp info for amd. This could be due to a missing sensor for your GPU model.");
                        None
                    },
                    |&value| Some(value as u8),
                )
                .unwrap_or(0);

            amd_junction_temp = amdgpu_info.get("Junction Temp").map_or_else(
                || {
                    println!("Error getting Junction Temp info for amd. This could be due to a missing sensor for your GPU model.");
                    None
                },
                |&value| Some(value),
            );

            amd_memory_temp = amdgpu_info.get("Memory Temp").map_or_else(
                || {
                    println!("Error getting Memory Temp info for amd. This could be due to a missing sensor for your GPU model.");
                    None
                },
                |&value| Some(value),
            );

            amd_min_temp = amdgpu_info.get("Min RPM").map_or_else(
                || {
                    println!("Error getting Min RPM info for amd. This could be due to a missing sensor for your GPU model.");
                    None
                },
                |&value| Some(value),
            );

            amd_max_temp = amdgpu_info.get("Max RPM").map_or_else(
                || {
                    println!("Error getting Max RPM info for amd. This could be due to a missing sensor for your GPU model.");
                    None
                },
                |&value| Some(value),
            );

            if !args.get_flag("force-amd")
                && (amd_current_rpm.is_none() || amd_fan_speed_percentage.is_none() || amd_junction_temp.is_none() || amd_memory_temp.is_none() || amd_min_temp.is_none() || amd_max_temp.is_none())
            {
                sleep(5.0);
            }
        } else {
            eprintln!("Error: Unknown GPU or no GPU found");
            exit(1);
        }
        let speed_output = diff_func(temp);
        if args.get_flag("no-tui") {
            if speed_output != Into::<u8>::into(temp_capture_call) {
                //TODO Added if statement here for later amd gpu intergration
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
                    print_centered_colored_string(
                        width,
                        "Junction/hotspot: {}°C",
                        amd_junction_temp,
                        Some(50.0),
                        Some(95.0),
                        &rgb_array,
                        args.get_flag("fahrenheit-id"),
                        Some("Junction/hotspot: {}°F"),
                    );

                    // Calculate Vram/Memory pos
                    print_centered_colored_string(width, "Memory/vram temp: {}°C", amd_memory_temp, Some(60.0), Some(90.0), &rgb_array, args.get_flag("fahrenheit-id"), Some("Memory/vram temp: {}°F"));

                    // Calculate rpm pos
                    print_centered_colored_string(width, "Current fan RPM: {}", amd_current_rpm, amd_min_temp, amd_max_temp, &rgb_array, false, Some(""));

                    // Calculate fanspeed pos
                    if let Some(fan_speed_percentage) = amd_fan_speed_percentage {
                        print_centered_colored_string(width, "Current fan speed: {}%", Some(fan_speed_percentage as f32), Some(30.0), Some(85.0), &rgb_array, false, Some(""));
                    }
                } else {
                    println!(
                        "{: >width$}",
                        fan_speed_output_str.truecolor(rgb_value_speed_output.0, rgb_value_speed_output.1, rgb_value_speed_output.2),
                        width = speed_output_center + fan_speed_output_str.len()
                    );
                }
                //TODO Added if statement here for later amd gpu intergration
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
