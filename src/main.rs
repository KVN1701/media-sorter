use xxhash_rust::xxh3::Xxh3;
use walkdir::WalkDir;
use std::collections::{HashMap, HashSet};
use std::path::{Path,PathBuf};
use std::fs;
use std::io::{BufReader, Read};
use rayon::prelude::*;
use std::process::Command;
use chrono::{Datelike, NaiveDateTime, Timelike, DateTime};
use indicatif::{ProgressBar, ParallelProgressIterator, ProgressStyle};
use clap::Parser;

const IMAGE_EXTENSIONS: [&str; 5] = ["jpg", "jpeg", "png", "gif", "bmp"];
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "avi", "mkv", "mov", "flv"];

#[derive(Parser)]
#[command(name = "image_sorter")]
#[command(version = "1.0")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Define the source folder
    source:PathBuf,

    #[arg(short, long, conflicts_with = "list")]
    /// Define the destination folder. Defaults to the value of source
    destination: Option<PathBuf>,

    ///
    
    #[arg(short, long, conflicts_with = "destination")]
    /// List the files in the source folder. Does not move or rename files.
    list:bool,

    #[arg(short, long, conflicts_with = "destination", conflicts_with = "list", conflicts_with = "quick")]
    /// Renames the files in the current directory without moving them.
    rename:bool,

    #[arg(short, long, conflicts_with = "list")]
    /// Greately improves speed, but does not check for duplicates. Does not override!
    quick:bool,

    #[arg(long, num_args = 0.., value_delimiter = ',')]
    /// Skips the directories, allows multiple entries separated by ','
    skip_dirs:Vec<String>,

    #[arg(long, conflicts_with = "list")]
    /// Does not automatically create subdirectories for every year (2000, 2001, ...)
    dont_create_subdirs:bool,
}


fn main() {
    let mut renamed_files: HashSet<String> = HashSet::new();

    // Parser
    let cli = Cli::parse();
    let source_dir = cli.source.clone();
    let destination_dir = cli.destination.clone().unwrap_or(source_dir.clone());

    // get the absolute paths
    let abs_source = match source_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap().join(&source_dir),
    };
    
    let abs_dest = match destination_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap().join(&source_dir),
    };

    // list option
    if cli.list {
        let files = get_files(&abs_source, &cli.skip_dirs);
        println!("[i] Found {} files in {}", files.len(), abs_source.display());
        get_files(&abs_source, &cli.skip_dirs).iter().for_each(|file| println!("[+] File found: {}", file));
        return;
    }

    // quick-mode
    if cli.quick {
        let source_files = get_files(&abs_source, &cli.skip_dirs);
        for file in &source_files {
            println!("[i] Renaming{}{}", if abs_source == abs_dest { " " } else { " and moving " }, file);
            let new_file = rename_file(&file, &abs_dest, &mut renamed_files, !cli.dont_create_subdirs).unwrap();
            if new_file != PathBuf::new() {
                println!("[+] {} file to {}", if abs_source == abs_dest { "Renamed" } else { "Moved" } , new_file.display())
            }
        }
        println!("[+] Finished {} {} files in {}", if abs_source == abs_dest { "Renaming" } else { "Moving" }, source_files.len(), abs_source.display());
        return;
    }

    // rename-mode
    if cli.rename {
        let source_files = get_files(&abs_source, &cli.skip_dirs);
        for file in &source_files {
            println!("[i] Renaming file '{}'", file);
            rename_file(file, &abs_source, &mut renamed_files, !cli.dont_create_subdirs).unwrap();
        }
        return;
    }

    // base case
    println!("[i] Gathering file hashes in source folder {}", abs_source.display());
    let source_files = get_file_hashes(&abs_source, &cli.skip_dirs);
    println!("[i] Gathering file hashes in destination folder {}", abs_dest.display());
    let dest_files = get_file_hashes(&abs_dest, &cli.skip_dirs);

    for (hash, filepath) in &source_files {
        println!("[i] Moving and renaming file {}", filepath);
        if !dest_files.contains_key(hash) {
            rename_file(filepath, &abs_dest, &mut renamed_files, !cli.dont_create_subdirs).unwrap();
            continue;
        }
        println!("[i] Duplicate detected for {}. Skipping file", filepath);
    }
        
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

fn get_date_taken(filepath: &str) -> Option<NaiveDateTime> {
    let output = Command::new("exiftool")
        .args([
            "-DateTimeOriginal",
            "-CreateDate",
            "-MediaCreateDate",
            "-TrackCreateDate",
            "-CreationTime",
            "-FileModifyDate",  // last resort fallback
            "-s3",
            "-f",
            filepath,
        ])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && *l != "-")
        .find_map(|l| {
            // "%Y:%m:%d %H:%M:%S" — standard EXIF
            NaiveDateTime::parse_from_str(l, "%Y:%m:%d %H:%M:%S")
                // "%Y:%m:%d %H:%M:%S%z" — with timezone offset (e.g. FileModifyDate)
                .or_else(|_| {
                    DateTime::parse_from_str(l, "%Y:%m:%d %H:%M:%S%z")
                        .map(|dt| dt.naive_local())
                })
                // "Sat 02 May 2026 16:23:10 CEST" — CreationTime format
                .or_else(|_| {
                    DateTime::parse_from_str(l, "%a %d %b %Y %H:%M:%S %Z")
                        .map(|dt| dt.naive_local())
                })
                .ok()
        })
}

fn rename_file(filepath: &str, destination_folder: &PathBuf, renamed_files: &mut HashSet<String>, create_subfolders: bool) -> Result<PathBuf, std::io::Error> {
    let filename = filepath.split("/").last().unwrap().to_string();

    if let Some(dt) = get_date_taken(filepath) {
        let mut dest_path = destination_folder.clone();
        let base_filename = format!(
            "{}-{}{:02}{:02}-{:02}{:02}{:02}.{}",
            if is_video_file(&filename) { "VID" } else { "IMG" },
            dt.year(), dt.month(), dt.day(),
            dt.hour(), dt.minute(), dt.second(),
            get_file_extension(&filename)
        );

        if create_subfolders {
            dest_path.push(dt.year().to_string());
        }
        fs::create_dir_all(&dest_path)?;

        // Deduplicate filename
        let mut new_filename = base_filename.clone();
        let mut counter: u32 = 1; // BUG FIX: was u8 and counter += counter (always 0)
        while renamed_files.contains(&new_filename) {
            let ext = get_file_extension(&base_filename);
            let stem_end = base_filename.len() - ext.len() - 1;
            new_filename = format!("{}_{:04}.{}", &base_filename[..stem_end], counter, ext);
            counter += 1;
        }

        renamed_files.insert(new_filename.clone());
        dest_path.push(new_filename);
        fs::rename(filepath, &dest_path)?;
        println!("[+] Successfully renamed to {}", dest_path.display());
        return Ok(dest_path);
    }

    // Fallback: move without renaming
    eprintln!("[-] No EXIF date found, moving as-is: {}", filepath);
    let mut fallback = destination_folder.clone();
    fallback.push(&filename);
    fs::rename(filepath, &fallback)?;
    println!("[-] Moved to {}", fallback.display());
    Ok(fallback)
}

fn path_contains_any_skip(path: &Path, skips: &[String]) -> bool {
    if skips.is_empty() { return false; }
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        skips.iter().any(|skip| skip == &s)
    })
}


fn get_file_hashes(path: &PathBuf, skipdirs: &[String]) -> HashMap<u64, String> {
    let files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !path_contains_any_skip(e.path(), skipdirs))
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


fn get_files(path: &PathBuf, skipdirs: &[String]) -> HashSet<String> {
    println!("[+] Gathering filenames ...");

    let files: HashSet<String> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !path_contains_any_skip(e.path(), skipdirs))
        .filter_map(|entry| {
            let filepath = entry.path().to_str().unwrap_or_default();
            if entry.file_type().is_file() && is_media_file(entry.path().to_str().unwrap_or_default()) {
                Some(String::from(filepath))
            }
            else {
                None
            }
        }
    ).collect();

    println!("[+] Files gathered successfully!");
    files
}
