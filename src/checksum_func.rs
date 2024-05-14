use std::fs::File;
use std::io::Read;

use sha2::{Digest, Sha256};

pub fn compute_file_sha256(file_path: &str) -> Option<String> {
    // Open the file
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return None, // Return None if file opening fails
    };

    // Initialize SHA-256 hasher
    let mut hasher = Sha256::new();

    // Read the entire file and update hasher
    let mut buffer = Vec::new();
    if let Err(_) = file.read_to_end(&mut buffer) {
        return None; // Return None if reading fails
    }
    hasher.update(&buffer);

    // Finalize the hash computation
    let result = hasher.finalize();

    // Convert the hash result to hexadecimal string
    let hash_string = result.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();

    Some(hash_string)
}
