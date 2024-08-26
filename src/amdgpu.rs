use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

const HWMON_PATH: &str = "/sys/class/hwmon";

/// Retrieves AMD GPU information and metrics.
///
/// This function serves as the primary entry point for obtaining AMD GPU data.
/// It coordinates the following processes:
/// 1. Locating the AMD GPU's hwmon directory
/// 2. Gathering relevant file paths for metric collection
/// 3. Calculating various metrics, including temperatures and fan speeds
///
/// # Returns
/// - `Some(HashMap<&'static str, f32>)`: A collection of calculated metrics if successful
/// - `None`: If any step in the process fails
pub fn get_amdgpu() -> Option<HashMap<&'static str, f32>> {
    let hwmon = find_amdgpu_hwmon().expect("Failed to find amdgpu hwmon");
    let amdgpu_paths = get_amdgpu_info_paths(hwmon);
    amdgpu_calc(amdgpu_paths)
}

/// Locates the hwmon directory for the AMD GPU.
///
/// This function scans the HWMON_PATH directory to find the
/// subdirectory associated with the AMD GPU's hardware monitoring.
fn find_amdgpu_hwmon() -> Option<PathBuf> {
    for entry in fs::read_dir(HWMON_PATH).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            let name_file_path = path.join("name");
            if let Ok(name) = fs::read_to_string(name_file_path) {
                if name.trim().contains("amdgpu") {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Retrieves file paths for AMD GPU metrics from the specified hwmon directory.
///
/// This function scans the given hwmon directory for specific files related to
/// AMD GPU metrics, including fan speeds and temperatures. It returns a HashMap
/// containing the relevant file paths if found.
fn get_amdgpu_info_paths(hwmon_path: PathBuf) -> Option<HashMap<&'static str, DirEntry>> {
    let mut file_paths = HashMap::with_capacity(6);

    for entry in fs::read_dir(hwmon_path).unwrap() {
        let file = entry.unwrap();
        let file_name = file.file_name();
        let file_name_str = file_name.to_str().unwrap_or("");

        match file_name_str {
            s if s.starts_with("fan") => {
                if s.ends_with("_min") {
                    file_paths.insert("Min RPM", file);
                } else if s.ends_with("_max") {
                    file_paths.insert("Max RPM", file);
                } else if s.ends_with("_input") {
                    file_paths.insert("Current RPM", file);
                }
            }
            "temp1_input" => {
                file_paths.insert("Edge Temp", file);
            }
            "temp2_input" => {
                file_paths.insert("Junction Temp", file);
            }
            "temp3_input" => {
                file_paths.insert("Memory Temp", file);
            }
            _ => {}
        }

        if file_paths.len() == 6 {
            break;
        }
    }

    Some(file_paths)
}

/// Calculates and processes AMD GPU metrics based on provided file paths.
///
/// This function takes a HashMap of file paths associated with various AMD GPU metrics,
/// reads the contents of these files, and processes the data to produce a set of
/// human-readable metrics. It performs the following operations:
///
/// 1. Reads and parses RPM values (minimum, maximum, and current)
/// 2. Calculates fan speed percentage based on the RPM values
/// 3. Reads and converts temperature values from millidegrees to degrees Celsius
///
/// The function returns a HashMap containing processed metrics if successful, or None if
/// critical data is missing or cannot be parsed.
fn amdgpu_calc(amdgpu_paths: Option<HashMap<&str, DirEntry>>) -> Option<HashMap<&str, f32>> {
    let mut result = HashMap::new();
    let mut rpm_values: [Option<f32>; 3] = [None; 3];

    if let Some(paths) = amdgpu_paths {
        for (file_name, dir_entry) in paths {
            if let Ok(content) = fs::read_to_string(dir_entry.path()) {
                match file_name {
                    "Min RPM" => rpm_values[0] = content.trim().parse().ok(),
                    "Max RPM" => rpm_values[1] = content.trim().parse().ok(),
                    "Current RPM" => rpm_values[2] = content.trim().parse().ok(),
                    "Edge Temp" | "Junction Temp" | "Memory Temp" => {
                        if let Ok(temp_value) = content.trim().parse::<f32>() {
                            result.insert(file_name, temp_value / 1000.0);
                        }
                    }
                    _ => {}
                }
            }
        }
    } else {
        return None;
    }

    if let [Some(min), Some(max), Some(current)] = rpm_values {
        let percentage = ((current - min) / (max - min)) * 100.0;
        result.extend([("Min RPM", min), ("Max RPM", max), ("Current RPM", current), ("Fan Speed Percentage", percentage)]);
        Some(result)
    } else {
        None
    }
}
