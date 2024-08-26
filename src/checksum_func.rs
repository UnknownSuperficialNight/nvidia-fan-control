use std::fs::File;
use std::io::Read;
use std::process::exit;

use sha2::{Digest, Sha256};

pub fn compute_file_sha256(file_path: &str) -> String {
    // Open the file
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening the file: {}", err);
            exit(1);
        }
    };

    // Initialize SHA-256 hasher
    let mut hasher = Sha256::new();

    // Read and update hasher in chunks
    let mut buffer = [0; 8192];
    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buffer[..n]),
            Err(err) => {
                eprintln!("Error reading the file: {}", err);
                exit(1);
            }
        }
    }

    // Finalize the hash computation
    let result = hasher.finalize();

    // Convert the hash result to hexadecimal string
    let hash_string: String = result.iter().map(|byte| format!("{:02x}", byte)).collect();

    hash_string
}
