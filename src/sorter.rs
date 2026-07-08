use walkdir::WalkDir;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::path::{Path,PathBuf};
use std::fs;
use rayon::prelude::*;

use crate::file::MediaFile;


pub fn get_files(path: &Path, skipdirs: &[String]) -> HashSet<MediaFile> {
    println!("[i] Gathering filenames ...");

    let files: HashSet<MediaFile> = WalkDir::new(path)
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


pub fn get_file_hashes(path: &Path, skipdirs: &[String], ignore: HashSet<MediaFile>) -> HashSet<MediaFile> {
    let files = get_files(path, skipdirs);

    let pb = ProgressBar::new(files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[i] Gathering file hashes: [{wide_bar}] {pos}/{len} - ETA: {eta} "
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let result : HashSet<MediaFile> = files
        .par_iter()
        .progress_with(pb)
        .filter_map(|mf| {
            if mf.is_media_file() && !ignore.contains(mf) {
                let mut media_file = mf.clone();
                media_file.load_hash();
                Some(media_file)
            } else {
                None
            }
        }).collect();

    println!("[i] File hashes gathered successfully!");
    result
}


pub fn rename_file(file: &MediaFile, destination_folder: &PathBuf, renamed_files: &mut HashSet<MediaFile>, rename:bool, create_subfolders: bool) -> anyhow::Result<()> {
    let new_filename = file.new_filename(destination_folder, renamed_files, rename, create_subfolders).unwrap();

    // create all subdirs needed
    if let Some(parent) = new_filename.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::rename(&file.file_loc, &new_filename)?;
    Ok(())
}

fn path_contains_any_skip(path: &Path, skips: &[String]) -> bool {
    if skips.is_empty() { return false; }
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        skips.iter().any(|skip| skip == &s)
    })
}
