use std::path::Path;
use std::collections::HashSet;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::Write;

use crate::file::MediaFile;
use crate::sorter::*;


pub fn list_files(source: &Path, skip_dirs: &[String], output: Option<String>) {
    let files = get_files(&source, &skip_dirs);
    println!("[i] Found {} files in {}", files.len(), source.display());

    // put the filenames into a file if the output option is set
    if output.is_some() {
        let mut output_file = File::create(output.unwrap()).unwrap();
        let mut contents = String::new();
        for f in &files {
            contents.push_str(&format!("{}\n", f));
            println!("[i] File found: {}", f);
        }
        output_file.write_all(contents.as_bytes()).unwrap();
        return;
    }
    files.iter().for_each(|file| println!("[i] File found: {}", file));
}

pub fn move_with_hashing(source: &Path, dest: &Path, skip_dirs: &[String], mut used_filenames: &mut HashSet<String>, dont_create_subdirs: bool) {
    if source == dest {
        println!("[!] The source and destination folder are equal.");
        println!("    If you want to sort your images in this folder run media-sorter with the '--quick' option.");
        return;
    }

    println!("[i] Gathering file hashes in source folder {:?}", source);
    let source_files = get_file_hashes(&source, skip_dirs, HashSet::new());

    println!("[i] Gathering file hashes in destination folder {:?}", &dest);
    let dest_files = get_file_hashes(&dest, skip_dirs, source_files.clone());

    // setting ub the pregress bar
    let pb = ProgressBar::new(source_files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[i] Processing files: [{wide_bar}] {pos}/{len} - ETA: {eta} "
        )
        .unwrap()
        .progress_chars("=> "),
    );

    for file in &source_files {
        if !dest_files.contains(file) {
            rename_file(file, &dest.to_path_buf(), &mut used_filenames, true,!dont_create_subdirs).unwrap();
        }
        pb.inc(1);
    }
    pb.finish();
}


pub fn rename_in_place(source: &Path, skip_dirs: &[String], mut used_filenames: &mut HashSet<String>) {
    let source_files = get_files(&source, &skip_dirs);
    let pb = ProgressBar::new(source_files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[i] Processing files: [{wide_bar}] {pos}/{len} - ETA: {eta} "
        )
        .unwrap()
        .progress_chars("=> "),
    );

    for file in &source_files {
        let dest_folder = match file.parent() {
            Some(p) => p.to_path_buf(),
            None => source.to_path_buf() // if not possible default to moving into the source folder
        };

        rename_file(file, &dest_folder, &mut used_filenames,true, false).unwrap();
        pb.inc(1);
    }
    pb.finish();
}


pub fn quick_mode(source: &Path, dest: &Path, skip_dirs: &[String], mut used_filenames: &mut HashSet<String>, dont_create_subdirs: bool) {
    let source_files = get_files(&source, &skip_dirs);
    let pb = ProgressBar::new(source_files.len() as u64);

    // customizing the progress bar
    pb.set_style(
        ProgressStyle::with_template(
            "[i] Processing files: [{wide_bar}] {pos}/{len} - ETA: {eta} "
        )
        .unwrap()
        .progress_chars("=> "),
    );

    // renaming and moving files
    for file in source_files {
        rename_file(&file, &dest.to_path_buf(), &mut used_filenames, true, !dont_create_subdirs).unwrap();
        pb.inc(1);
    }
    pb.finish();
}