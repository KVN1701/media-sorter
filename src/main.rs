use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use hex;


fn main() {
    println!("File hash: {}", get_file_hashes("./")["main.rs"]);
}

fn get_file_hashes(path: &str) -> HashMap<String, String> {
    let mut file_hashes: HashMap<String, String> = HashMap::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let mut file = File::open(entry.path()).unwrap();
            let mut hasher = Sha256::new();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();

            hasher.update(&mut buffer);
            let hash = hex::encode(hasher.finalize());

            if let Some(filename) = entry.file_name().to_str() {
                file_hashes.insert(filename.to_string(), hash);
            } 
        }
    }
    file_hashes
}
