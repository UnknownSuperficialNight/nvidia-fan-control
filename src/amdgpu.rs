use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::io::{BufReader, Read};
use std::path::PathBuf;

const HWMON_PATH: &str = "/sys/class/hwmon";

// Used as a starting point to amdgpu functions
pub fn get_amdgpu() -> Option<HashMap<String, f32>> {
    let hwmon = find_amdgpu_hwmon().expect("Failed to find amdgpu hwmon");
    let amdgpu_paths = get_amdgpu_info_paths(hwmon);
    amdgpu_calc(amdgpu_paths)
}

// Used to find the amdgpu directory
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

// Used to get the correct Paths of target files from the amdgpu directory
fn get_amdgpu_info_paths(hwmon_path: PathBuf) -> Option<HashMap<String, DirEntry>> {
    let mut file_paths = HashMap::new();

    for file in fs::read_dir(hwmon_path).unwrap() {
        let file = file.unwrap();
        let file_name = file.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Fan RPM logic
        if file_name_str.contains("fan") {
            if file_name_str.contains("min") {
                file_paths.insert("Min RPM".to_string(), file);
            } else if file_name_str.contains("max") {
                file_paths.insert("Max RPM".to_string(), file);
            } else if file_name_str.contains("input") {
                file_paths.insert("Current RPM".to_string(), file);
            }
        }
        // Temperature logic
        else if file_name_str.contains("temp") {
            if file_name_str.contains("temp1_input") {
                file_paths.insert("Edge Temp".to_string(), file);
            } else if file_name_str.contains("temp2_input") {
                file_paths.insert("Junction Temp".to_string(), file);
            } else if file_name_str.contains("temp3_input") {
                file_paths.insert("Memory Temp".to_string(), file);
            }
        }
    }

    Some(file_paths)
}

// Used to calculate percentage and take the input amdgpu paths and return their values in a human
// readable format
fn amdgpu_calc(amdgpu_paths: Option<HashMap<String, DirEntry>>) -> Option<HashMap<String, f32>> {
    let mut result = HashMap::new();

    let mut min_rpm = None;
    let mut max_rpm = None;
    let mut current_rpm = None;

    // Read data from target files and add them to the hashmap
    for (file_name, dir_entry) in amdgpu_paths.unwrap() {
        let file = fs::File::open(dir_entry.path()).unwrap();
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();

        if file_name.contains("RPM") {
            let value: i32 = content.trim().parse().unwrap_or(0);

            if file_name.contains("Min RPM") {
                min_rpm = Some(value);
            } else if file_name.contains("Max RPM") {
                max_rpm = Some(value);
            } else if file_name.contains("Current RPM") {
                current_rpm = Some(value);
            }
        } else if file_name.contains("Temp") {
            let temp_value: f32 = content.trim().parse().unwrap_or(0.0) / 1000.0; // Convert from millidegree Celsius to Celsius

            if file_name.contains("Edge Temp") {
                result.insert("Edge Temp".to_string(), temp_value);
            } else if file_name.contains("Junction Temp") {
                result.insert("Junction Temp".to_string(), temp_value);
            } else if file_name.contains("Memory Temp") {
                result.insert("Memory Temp".to_string(), temp_value);
            }
        }
    }

    // Calculate Fan Percentage based off (min_rpm, max_rpm, current_rpm)
    if let (Some(min), Some(max), Some(current)) = (min_rpm, max_rpm, current_rpm) {
        let percentage = ((current - min) as f32 / (max - min) as f32) * 100.0;
        result.insert("Min RPM".to_string(), min as f32);
        result.insert("Max RPM".to_string(), max as f32);
        result.insert("Current RPM".to_string(), current as f32);
        result.insert("Fan Speed Percentage".to_string(), percentage);
        return Some(result);
    }

    None
}
