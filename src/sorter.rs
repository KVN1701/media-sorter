use anyhow::anyhow;
use walkdir::WalkDir;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::path::{Path,PathBuf};
use std::fs;
use rayon::prelude::*;
use crate::file::MediaFile;


pub fn get_files(path: &Path, recursive: &bool, skipdirs: &[String]) -> HashSet<MediaFile> {
    println!("[i] Gathering filenames ...");

    let mut walkdir = WalkDir::new(path);
    if !recursive {
        walkdir = walkdir.max_depth(1);
    }

    let files: HashSet<MediaFile> = walkdir
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !path_contains_any_skip(e.path(), skipdirs))
        .filter_map(|entry| {
            if let Ok(mf) = MediaFile::new(&entry.into_path()) {
                Some(mf)
            } else {
                None
            }
        }
    ).collect();

    println!("[i] Files gathered successfully!");
    files
}


pub fn get_file_hashes(path: &Path, skipdirs: &[String], ignore: &HashSet<MediaFile>, recursive: &bool) -> HashSet<MediaFile> {
    let files = get_files(path, recursive, skipdirs);

    let pb = ProgressBar::new(files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[i] Gathering file hashes: [{wide_bar}] {pos}/{len} - ETA: {eta} "
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let files_with_hashes : Vec<MediaFile> = files
        .par_iter()
        .progress_with(pb)
        .filter_map(|mf| {
            if mf.is_media_file() && !ignore.contains(mf) {
                let mut media_file = mf.clone();
                media_file.load_hash().ok()?;
                Some(media_file)
            } else {
                None
            }
        }).collect();

    let mut seen: HashSet<u64> = HashSet::new();
    let mut unique_files: HashSet<MediaFile> = HashSet::new();
    
    // delete files found in the same directory if they have the same hash value
    for mf in files_with_hashes {
        if let Some(hash) = &mf.hash {
            if !seen.insert(hash.clone()) {
                if let Err(e) = fs::remove_file(&mf.file_loc) {
                    eprintln!("[!] Unable to delete {}:\n    {}", mf, e)
                }
                continue;
            }
        }
        unique_files.insert(mf);
    }

    println!("[i] File hashes gathered successfully!");
    unique_files
}


pub fn move_file(file: &MediaFile, destination_folder: &Option<PathBuf>, used_filenames: &mut HashSet<String>, rename: bool, create_subfolders: bool) -> anyhow::Result<()> {
    let dest = match destination_folder {
        Some(d) => d,
        None => file.parent()
    };

    let new_filename = match file.new_filename(dest, used_filenames, rename, create_subfolders) {
        Some(name) => name,
        None => return Err(anyhow!("Failed to gather new filename for {}", file))
    };

    // create all subdirs needed
    if let Some(parent) = new_filename.parent() {
        fs::create_dir_all(parent)?;
    }
    
    match fs::rename(&file.file_loc, &new_filename) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("[!] Failed moving the file:\n    {}", e))
    }
}

fn path_contains_any_skip(path: &Path, skips: &[String]) -> bool {
    if skips.is_empty() { return false; }
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        skips.iter().any(|skip| skip == &s)
    })
}
