use std::fs::{File, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header;

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

    // Compare with the compiled version
    let is_older = {
        let compiled_parts: Vec<_> = compiled_version.split('.').collect();
        let repo_parts: Vec<_> = repo_version.split('.').collect();

        if let Some((repo, compiled)) = repo_parts.iter().zip(compiled_parts.iter()).next() {
            let repo_num: u64 = repo.parse()?;
            let compiled_num: u64 = compiled.parse()?;

            if repo_num > compiled_num {
                return Ok((true, repo_version.to_string()));
            } else {
                return Ok((false, repo_version.to_string()));
            }
        }

        false
    };

    Ok((is_older, repo_version.to_string()))
}

pub fn update_func(binary_name: String, file_path: &Path) {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let release_url = "https://api.github.com/repos/UnknownSuperficialNight/nvidia-fan-control/releases/latest";

        let client = reqwest::Client::new();

        let response = client.get(release_url).header(header::USER_AGENT, "User-Agent: Nvidia Fanctrl").send().await.unwrap();

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

        if let Some(asset) = asset {
            let download_url = asset["browser_download_url"].as_str().unwrap();
            if let Err(e) = std::fs::remove_file(file_path) {
                eprintln!("Failed to delete old file: {}", e);
            }
            let mut file = File::create(file_path).expect("Failed to create file");

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
    });
}
