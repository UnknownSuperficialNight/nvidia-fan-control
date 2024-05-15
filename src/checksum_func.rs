use std::fmt::Write;
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
    if file.read_to_end(&mut buffer).is_err() {
        return None; // Return None if reading fails
    }
    hasher.update(&buffer);

    // Finalize the hash computation
    let result = hasher.finalize();

    // Convert the hash result to hexadecimal string
    let mut hash_string = String::new();
    for byte in result {
        write!(&mut hash_string, "{:02x}", byte).expect("Failed to write to String");
    }

    Some(hash_string)
}
