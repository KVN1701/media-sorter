use xxhash_rust::xxh3::Xxh3;
use walkdir::WalkDir;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs::{self, metadata};
use std::io::{BufReader, Read};
use rayon::prelude::*;

const IMAGE_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "gif", "bmp"];
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "avi", "mkv", "mov", "flv"];


fn main() {
    let mut renamed_files: HashSet<String>;

    let mut source_dir = PathBuf::from("/home/kvn/Pictures/");
    let mut destination_dir = PathBuf::from("./sorted_images");

    // `push` fügt bei Bedarf einen Pfadtrenner hinzu
    source_dir.push(""); 
    destination_dir.push("");

    get_new_name("/home/kvn/Pictures/Privat/2019/07-12_Grundausbildung/IMG-20191214-WA0081.jpg", ".");

    //let source_files = get_file_hashes(&source_dir);
    //let destination_files = get_file_hashes(&destination_dir);

    
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

fn get_new_name(filepath: &str, destination_folder: &str) -> String {
    if let Ok(metadata) = std::fs::metadata(filepath) {
        if let Ok(modification_date) = metadata.modified() {
            println!("{:?}", modification_date); // TODO: Change to another crate function
            return filepath.to_string();
        }
    }
    filepath.to_string()
}

fn get_file_hashes(path: &PathBuf) -> HashMap<u64, String> {
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
                Some((hash, filepath))
            } else {
                None
            }
        })
        .collect()
}
