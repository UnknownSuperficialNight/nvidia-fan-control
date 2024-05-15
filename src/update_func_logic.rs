use std::fs::{rename, File, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header;

#[derive(Debug)]
pub struct Checksum {
    pub key: String,
    pub value: String,
}

pub fn is_current_version_older(repo_url: &str, compiled_version: &str) -> Result<(bool, String), Box<dyn std::error::Error>> {
    // Fetch the Cargo.toml file from the repository
    let cargo_toml_url = format!("{}/raw/main/Cargo.toml", repo_url);
    let cargo_toml_content = reqwest::blocking::get(cargo_toml_url)?.text()?;

    // Extract the version from Cargo.toml
    let repo_version = cargo_toml_content
        .lines()
        .find(|line| line.starts_with("version"))
        .and_then(|line| line.split('=').nth(1))
        .and_then(|version| version.trim().strip_prefix('"').and_then(|v| v.strip_suffix('"')))
        .ok_or("Version not found in Cargo.toml")?;

    // Compare the versions
    let is_older = compare_versions(compiled_version, repo_version)?;

    Ok((is_older, repo_version.to_string()))
}

fn compare_versions(compiled_version: &str, repo_version: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let compiled_parts: Vec<u64> = compiled_version.split('.').map(|part| part.parse::<u64>()).collect::<Result<_, _>>()?;
    let repo_parts: Vec<u64> = repo_version.split('.').map(|part| part.parse::<u64>()).collect::<Result<_, _>>()?;

    let max_length = std::cmp::max(compiled_parts.len(), repo_parts.len());

    for i in 0..max_length {
        let compiled_part = *compiled_parts.get(i).unwrap_or(&0);
        let repo_part = *repo_parts.get(i).unwrap_or(&0);

        if repo_part > compiled_part {
            return Ok(true);
        } else if compiled_part > repo_part {
            return Ok(false);
        }
    }

    // If all parts are equal, the version is not older
    Ok(false)
}

pub fn update_func_commit(file_path: &Path, file_path_tmp: &Path) {
    if let Err(e) = std::fs::remove_file(file_path) {
        eprintln!("Failed to delete old file: {}", e);
    } else {
        match rename(file_path_tmp, file_path) {
            Ok(_) => {}
            Err(e) => eprintln!("Failed to rename file: {}", e),
        }
    }
}

pub fn update_func(binary_name: &str, file_path_tmp: &Path) -> Vec<Checksum> {
    let checksums_result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let release_url = "https://api.github.com/repos/UnknownSuperficialNight/nvidia-fan-control/releases/latest";

        let client = reqwest::Client::new();

        let response = client.get(release_url).header(header::USER_AGENT, "Nvidia Fanctrl").send().await.unwrap();

        let response_text = response.text().await.unwrap();

        let json_result = serde_json::from_str::<serde_json::Value>(&response_text).unwrap();

        let assets = json_result["assets"].as_array().unwrap();
        let asset = assets
            .iter()
            .find(|a| {
                let name = a["name"].as_str().unwrap();
                name == binary_name
            })
            .or_else(|| {
                assets.iter().find(|a| {
                    let name = a["name"].as_str().unwrap();
                    name == "Rust-gpu-fan-control-static"
                })
            });

        // Find the checksums.json asset
        let checksums_asset = assets.iter().find(|a| {
            let name = a["name"].as_str().unwrap_or_default();
            name == "checksums.json"
        });

        let mut checksums: Vec<Checksum> = Vec::new();
        if let Some(asset) = checksums_asset {
            let checksums_url = asset["browser_download_url"].as_str().unwrap_or_default();

            let response = client.get(checksums_url).header(header::USER_AGENT, "Nvidia Fanctrl").send().await.unwrap();

            // Extract the response body as bytes
            let response_bytes = response.bytes().await.unwrap();

            // Parse checksums.json as JSON
            let checksums_json: serde_json::Value = serde_json::from_slice(&response_bytes).unwrap();

            if let Some(object) = checksums_json.as_object() {
                for (key, value) in object {
                    let checksum = Checksum { key: key.clone(), value: value.to_string() };
                    checksums.push(checksum);
                }
            }
        }

        if let Some(asset) = asset {
            let download_url = asset["browser_download_url"].as_str().unwrap();

            let mut file = File::create(file_path_tmp).expect("Failed to create file");

            let mut response = client.get(download_url).send().await.unwrap();
            let content_length = response.content_length().unwrap();
            let mut downloaded = 0u64;
            let pb = ProgressBar::new(content_length);
            pb.set_style(
                ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} MiB ({eta})").expect("Failed to create ProgressStyle object").progress_chars("##-"),
            );

            while let Some(chunk) = response.chunk().await.unwrap() {
                file.write_all(&chunk).unwrap();
                downloaded += chunk.len() as u64;
                pb.set_position(downloaded);
            }
            let permissions = Permissions::from_mode(0o755); // Sets the permission to rwxr-xr-x
            file.set_permissions(permissions).unwrap();
        }

        checksums
    });
    checksums_result
}
