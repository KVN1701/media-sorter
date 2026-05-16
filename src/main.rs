use xxhash_rust::xxh3::Xxh3;
use walkdir::WalkDir;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::io::{BufReader, Read};
use rayon::prelude::*;

const IMAGE_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "gif", "bmp"];
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "avi", "mkv", "mov", "flv"];


fn main() {
    let mut source_dir = PathBuf::from("/home/kvn/Pictures/");
    let mut destination_dir = PathBuf::from("./sorted_images");

    // `push` fügt bei Bedarf einen Pfadtrenner hinzu
    source_dir.push(""); 
    destination_dir.push("");

    let source_files = get_file_hashes(&source_dir);
    let destination_files = get_file_hashes(&destination_dir);

    println!("{}", source_files.len());
    
    for (filepath, hash) in &source_files {
        println!("filepath: {}", filepath);
    }
}

fn is_image_file(filename: &str) -> bool {
    IMAGE_EXTENSIONS.iter().any(|ext| filename.to_lowercase().ends_with(ext))
}

fn is_video_file(filename: &str) -> bool {
    VIDEO_EXTENSIONS.iter().any(|ext| filename.to_lowercase().ends_with(ext))
}

fn is_media_file(filename: &str) -> bool {
    is_image_file(filename) || is_video_file(filename)
}

fn get_new_name(path: &str) -> String {

}

fn get_file_hashes(path: &PathBuf) -> HashMap<String, u64> {
    WalkDir::new(path)
        .into_iter()
        .par_bridge() // change iterator to parallel one
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().is_file() && is_media_file(entry.path().to_str().unwrap_or_default()) {
                let file = fs::File::open(entry.path()).ok()?;
                let mut reader = BufReader::new(file);
                let mut buffer = [0; 8192];
                let mut hasher = Xxh3::new();
                
                loop {
                    let count = reader.read(&mut buffer).ok()?;
                    if count == 0 { break; }
                    hasher.update(&buffer[..count]);
                }
                
                let hash = hasher.digest();
                let filepath = entry.path().to_str().unwrap().to_string();
                Some((filepath, hash))
            } else {
                None
            }
        })
        .collect()
}
