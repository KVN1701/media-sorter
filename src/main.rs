use xxhash_rust::xxh3::Xxh3;
use walkdir::WalkDir;
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::path::PathBuf;
use std::fs::{self, DirEntry};
use std::io::{BufReader, Read};
use rayon::prelude::*;
use rexif::ExifTag;
use colored::Colorize;
use chrono::{Datelike, NaiveDateTime, Timelike};
use indicatif::{ProgressBar, ParallelProgressIterator, ProgressStyle};

const IMAGE_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "gif", "bmp"];
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "avi", "mkv", "mov", "flv"];



fn main() {
    let mut renamed_files: HashSet<String>;

    let mut source_dir = PathBuf::from("/home/kvn/Media/");
    let mut destination_dir = PathBuf::from("./sorted_images");

    // `push` fügt bei Bedarf einen Pfadtrenner hinzu
    source_dir.push(""); 
    destination_dir.push("");

    //rename_file("/home/kvn/Pictures/Privat/2019/07-12_Grundausbildung/IMG-20191214-WA0081.jpg", &destination_dir, true);

    let source_files = get_files(&source_dir);//get_file_hashes(&source_dir);
    //let destination_files = get_file_hashes(&destination_dir);

    
}

fn get_file_extension(filename: &str) -> String {
    filename.split(".").last().unwrap().to_string()
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

fn rename_file(filepath: &str, destination_folder: &PathBuf, renamed_files: &mut HashSet<String>, create_subfolders: bool) -> Result<(), std::io::Error> {
    let filename = filepath.split("/").last().unwrap().to_string();

    match rexif::parse_file(filepath) {
        Ok(exif_data) => {
            if let Some(entry) = exif_data.entries.iter().find(|e| e.tag == ExifTag::DateTimeOriginal){
                let date_taken = entry.value_more_readable.trim();

                match NaiveDateTime::parse_from_str(date_taken, "%Y:%m:%d %H:%M:%S") {
                    Ok(dt) => {
                        let mut dest_path = destination_folder.clone();
                        let base_filename = format!("{}-{}{:02}{:02}-{:02}{:02}{:02}.{}",
                            if is_video_file(&filename) { "VID" } else {"IMG"},
                            dt.year(), 
                            dt.month(),
                            dt.day(), 
                            dt.hour(),
                            dt.minute(),
                            dt.second(),
                            get_file_extension(&filename)
                        );

                        if create_subfolders {
                            dest_path.push(dt.year().to_string());
                        }
                        fs::create_dir_all(&dest_path)?;

                        // Check if a file has the same name
                        let mut new_filename = base_filename.clone();
                        let mut counter: u8 = 0;

                        // append a number to the name
                        while renamed_files.contains(&new_filename) {
                            counter += counter;
                            let end = new_filename.char_indices().nth_back(get_file_extension(&new_filename).len()).map(|(i, _)| i).unwrap_or_default();
                            let prefix = new_filename[..end].to_string();
                            new_filename = format!("{}_{}.{}", prefix, format!("{:04}", counter), get_file_extension(&filename));
                        }
                        renamed_files.insert(new_filename.clone());
                        dest_path.push(new_filename);
                        fs::rename(filepath, &dest_path)?;
                        return Ok(());
                    }
                    Err(e) => eprintln!("[!] An error has occured with file '{}':\n\t{}\n[i] Skipping file {}", filename, e.to_string().red(), filepath)
                }
            }
        }
        Err(e) => eprintln!("[!] An error has occured with file '{}':\n\t{}\n[i] Skipping file {}", filename, e.to_string().red(), filepath)
    }
    fs::rename(filepath, destination_folder)?;
    Ok(())
}


fn get_file_hashes(path: &PathBuf) -> HashMap<u64, String> {
    let files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    let pb = ProgressBar::new(files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[+] Gathering file hashes: [{bar:65}] {pos}/{len} - ETA: {eta}"
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let result : HashMap<u64, String> = files
        .par_iter()
        .progress_with(pb)
        .filter_map(|entry| {
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
        }).collect();

    println!("[+] File hashes gathered successfully!");
    result
}


fn get_files(path: &PathBuf) -> HashSet<String> {
    println!("[+] Gathering filenames ...");

    let files: HashSet<String> = WalkDir::new(path).into_iter().filter_map(|e| e.ok()).filter_map(|entry| {
        let filepath = entry.path().to_str().unwrap_or_default();
        if entry.file_type().is_file() && is_media_file(entry.path().to_str().unwrap_or_default()) {
            Some(String::from(filepath))
        }
        else {
            None
        }
    }).collect();

    println!("[+] Files gathered successfully!");
    files
}
